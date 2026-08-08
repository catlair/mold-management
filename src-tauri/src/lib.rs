mod agent;
mod attachments;
mod backup;
mod data_package;
mod db;
mod excel;
mod log;
mod secret_store;
mod storage;
mod webdav_sync;

use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};

// ========== 备份记录结构 ==========

#[derive(Serialize, Deserialize, Clone, Debug)]
struct BackupRecord {
    file_path: String,
    backup_time: String,
    backup_reason: String,
    backup_md5: String,
    locked: bool,
}

fn get_backup_index_path(backup_dir: &str) -> PathBuf {
    PathBuf::from(backup_dir).join("backups.json")
}

fn load_backup_index(backup_dir: &str) -> Result<Vec<BackupRecord>, String> {
    let index_path = get_backup_index_path(backup_dir);
    if !index_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&index_path)
        .map_err(|e| format!("读取备份索引失败「{}」: {}", index_path.display(), e))?;
    match serde_json::from_str(&content) {
        Ok(records) => Ok(records),
        Err(error) => {
            let damaged_path = index_path.with_file_name(format!(
                "backups-damaged-{}.json",
                Local::now().format("%Y%m%d_%H%M%S")
            ));
            fs::rename(&index_path, &damaged_path).map_err(|rename_error| {
                format!(
                    "备份索引损坏（{}），且无法隔离旧索引「{}」: {}",
                    error,
                    index_path.display(),
                    rename_error
                )
            })?;
            eprintln!(
                "备份索引损坏，已隔离到「{}」并准备从目录重建: {}",
                damaged_path.display(),
                error
            );
            Ok(Vec::new())
        }
    }
}

fn save_backup_index(backup_dir: &str, records: &[BackupRecord]) -> Result<(), String> {
    fs::create_dir_all(backup_dir)
        .map_err(|e| format!("创建备份目录失败「{}」: {}", backup_dir, e))?;
    storage::atomic_write_json(&get_backup_index_path(backup_dir), records)
        .map_err(|e| format!("保存备份索引失败: {}", e))
}

fn backup_time_from_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let raw = stem.strip_prefix("mold-backup-").unwrap_or_default();
    NaiveDateTime::parse_from_str(raw, "%Y%m%d_%H%M%S_%3f")
        .map(|value| value.format("%Y-%m-%d %H:%M:%S").to_string())
        .or_else(|_| {
            fs::metadata(path)
                .and_then(|metadata| metadata.modified())
                .map(|modified| {
                    chrono::DateTime::<Local>::from(modified)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                })
        })
        .unwrap_or_else(|_| Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
}

fn is_managed_backup_path(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        && path
            .file_stem()
            .and_then(|value| value.to_str())
            .and_then(|name| name.strip_prefix("mold-backup-"))
            .is_some_and(|timestamp| {
                NaiveDateTime::parse_from_str(timestamp, "%Y%m%d_%H%M%S_%3f").is_ok()
            })
}

fn repair_backup_index(backup_dir: &str) -> Result<Vec<BackupRecord>, String> {
    fs::create_dir_all(backup_dir)
        .map_err(|e| format!("创建备份目录失败「{}」: {}", backup_dir, e))?;
    let records = load_backup_index(backup_dir)?;
    let mut actual_files = Vec::new();
    for entry in fs::read_dir(backup_dir)
        .map_err(|e| format!("扫描备份目录失败「{}」: {}", backup_dir, e))?
    {
        let path = entry
            .map_err(|e| format!("读取备份目录项失败: {}", e))?
            .path();
        if is_managed_backup_path(&path) {
            match backup::validate_backup_zip(&path) {
                Ok(()) => actual_files.push(path),
                Err(error) => eprintln!(
                    "发现无法使用的备份 ZIP，已保留文件但不加入索引「{}」: {}",
                    path.display(),
                    error
                ),
            }
        }
    }
    actual_files.sort();
    let mut repaired_records = Vec::with_capacity(actual_files.len());
    for path in actual_files {
        // 索引只保存备份目录实际扫描到的文件，不能通过 JSON 将清理范围扩展到目录外。
        if let Some(existing) = records
            .iter()
            .find(|record| Path::new(&record.file_path) == path)
        {
            if repaired_records
                .iter()
                .any(|record: &BackupRecord| record.file_path == existing.file_path)
            {
                continue;
            }
            repaired_records.push(existing.clone());
        } else {
            repaired_records.push(BackupRecord {
                file_path: path.to_string_lossy().to_string(),
                backup_time: backup_time_from_path(&path),
                backup_reason: "索引自动修复".to_string(),
                backup_md5: String::new(),
                locked: false,
            });
        }
    }
    repaired_records.sort_by(|left, right| left.backup_time.cmp(&right.backup_time));
    save_backup_index(backup_dir, &repaired_records)?;
    Ok(repaired_records)
}

fn attachment_dir_for_data(file_path: &str) -> Result<PathBuf, String> {
    Path::new(file_path)
        .parent()
        .map(|parent| parent.join("attachments"))
        .ok_or_else(|| "无法确定附件数据目录".to_string())
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory).map_err(|e| format!("读取目录失败: {}", e))? {
        let path = entry.map_err(|e| format!("读取目录项失败: {}", e))?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn dataset_md5(file_path: &str) -> Result<String, String> {
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(b"mold-management-dataset-v2\0");
    hasher.update(db::content_sha256(file_path)?.as_bytes());

    let attachment_dir = attachment_dir_for_data(file_path)?;
    let mut files = Vec::new();
    collect_files(&attachment_dir, &mut files)?;
    files.sort();
    for path in files {
        if let Ok(relative) = path.strip_prefix(&attachment_dir) {
            hasher.update(relative.to_string_lossy().replace('\\', "/").as_bytes());
            hasher.update([0]);
        }
        let bytes = fs::read(&path)
            .map_err(|e| format!("读取附件备份内容失败 {}: {}", path.display(), e))?;
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn do_backup(file_path: &str, backup_dir: &str, reason: &str) -> Result<String, String> {
    if !Path::new(file_path).exists() {
        return Err(format!("数据文件不存在: {}", file_path));
    }

    // 确保备份目录存在
    fs::create_dir_all(backup_dir).map_err(|e| format!("创建备份目录失败: {}", e))?;
    if !Path::new(backup_dir).exists() {
        return Err(format!("备份目录创建失败: {}", backup_dir));
    }

    let current_md5 = dataset_md5(file_path)?;

    // 只与最近一次新版备份比较；数据回到早期状态时仍会创建新的当前快照。
    let records = repair_backup_index(backup_dir)?;
    if records
        .last()
        .is_some_and(|record| !record.backup_md5.is_empty() && record.backup_md5 == current_md5)
    {
        return Ok(String::new());
    }

    let now = Local::now();
    let backup_name = format!("mold-backup-{}.zip", now.format("%Y%m%d_%H%M%S_%3f"));
    let backup_file = PathBuf::from(backup_dir).join(&backup_name);
    let created_at = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let bytes = backup::create_backup_zip(file_path, reason, &created_at)?;
    storage::atomic_write(&backup_file, &bytes)?;

    // 备份 ZIP 完整落盘后再写入索引。
    let mut new_records = records;
    new_records.push(BackupRecord {
        file_path: backup_file.to_string_lossy().to_string(),
        backup_time: created_at,
        backup_reason: reason.to_string(),
        backup_md5: current_md5,
        locked: false,
    });
    if let Err(error) = save_backup_index(backup_dir, &new_records) {
        let _ = fs::remove_file(&backup_file);
        return Err(error);
    }

    Ok(backup_file.to_string_lossy().to_string())
}

fn cleanup_old_backups(backup_dir: &str, keep_count: usize) -> Result<(), String> {
    let mut records = repair_backup_index(backup_dir)?;

    // 统计未锁定的记录
    let unlocked: Vec<usize> = records
        .iter()
        .enumerate()
        .filter(|(_, r)| !r.locked)
        .map(|(i, _)| i)
        .collect();

    // 如果未锁定的数量超过保留数，删除最早的
    if unlocked.len() > keep_count {
        let to_remove = unlocked.len() - keep_count;
        let mut remove_indices: Vec<usize> = unlocked.into_iter().take(to_remove).collect();
        remove_indices.sort_by(|a, b| b.cmp(a)); // 从后往前删

        for idx in remove_indices {
            let record = &records[idx];
            let backup_file = Path::new(&record.file_path);
            if let Err(e) = fs::remove_file(backup_file) {
                // 单个过期备份清理失败（如安全软件拒绝访问）不影响主流程：
                // 仍把记录从索引中移除，下次备份时再尝试清理。
                eprintln!(
                    "清理过期备份失败「{}」: {}（将留待后续清理）",
                    backup_file.display(),
                    e
                );
            }
            records.remove(idx);
        }

        save_backup_index(backup_dir, &records)?;
    }
    Ok(())
}

#[tauri::command]
fn list_backups(state: State<AppState>) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir_for_file(&path, &config);
    let records = repair_backup_index(&backup_dir)?;
    serde_json::to_value(records).map_err(|e| format!("序列化备份记录失败: {}", e))
}

#[tauri::command]
fn toggle_backup_lock(state: State<AppState>, index: usize) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir_for_file(&path, &config);
    let mut records = repair_backup_index(&backup_dir)?;
    if index < records.len() {
        records[index].locked = !records[index].locked;
        save_backup_index(&backup_dir, &records)?;
        Ok(json!({ "success": true, "locked": records[index].locked }))
    } else {
        Err("无效的索引".to_string())
    }
}

fn get_backup_dir_for_file(file_path: &str, config: &Config) -> String {
    // 优先使用用户配置的备份路径
    if let Some(ref custom) = config.backup_path {
        if !custom.is_empty() {
            return normalized_string(Path::new(custom));
        }
    }
    // 没配置则用数据文件同级 backups 目录
    Path::new(file_path)
        .parent()
        .map(|p| normalized_string(&p.join("backups")))
        .unwrap_or_else(default_backup_dir)
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
struct WebDavConfig {
    url: String,
    remote_path: String,
    username_encrypted: String,
    password_encrypted: String,
    auto_upload_on_start: bool,
    auto_upload_on_exit: bool,
    last_etag: Option<String>,
    last_uploaded_at: Option<String>,
    last_downloaded_at: Option<String>,
}

/// AI 助手配置：多份配置档案 + 当前启用项。
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
struct AgentConfig {
    /// 全部配置档案（内置或自定义）
    profiles: Vec<AgentProfile>,
    /// 当前启用的档案 id
    active: String,
}

/// 单份 AI 配置档案。
/// kind = "builtin"：provider 为内置服务商（opencode-zen/deepseek/glm/openai/anthropic/qwen/gemini），
///   endpoint 由后端预设，只需提供 API Key，model 可按需选择/修改；
/// kind = "custom"：format 为 "openai"（OpenAI Chat Completions）或 "cc"（Anthropic Messages，Claude Code 兼容），
///   endpoint/model 由用户填写。
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
struct AgentProfile {
    id: String,
    name: String,
    kind: String,
    provider: String,
    format: String,
    endpoint: String,
    model: String,
}

impl Default for AgentProfile {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            kind: "builtin".to_string(),
            provider: AGENT_PROVIDER_OPENCODE_ZEN_FREE.to_string(),
            format: AGENT_PROTOCOL_OPENAI.to_string(),
            endpoint: String::new(),
            model: "deepseek-v4-flash-free".to_string(),
        }
    }
}

const AGENT_PROVIDER_GLM: &str = "glm";
const AGENT_PROVIDER_OPENAI: &str = "openai";
const AGENT_PROVIDER_ANTHROPIC: &str = "anthropic";
const AGENT_PROVIDER_DEEPSEEK: &str = "deepseek";
const AGENT_PROVIDER_QWEN: &str = "qwen";
const AGENT_PROVIDER_GEMINI: &str = "gemini";
const AGENT_PROVIDER_CUSTOM_ANTHROPIC: &str = "custom-anthropic";
/// 内置 opencode Zen 服务商（OpenAI 兼容，同一网关地址）。
/// 拆分为两个选项：需 Key 版（付费模型）与免费版（免费模型，无需 API Key）。
const AGENT_PROVIDER_OPENCODE_ZEN: &str = "opencode-zen";
const AGENT_PROVIDER_OPENCODE_ZEN_FREE: &str = "opencode-zen-free";
/// 自定义配置的协议格式：cc = Anthropic Messages（Claude Code 兼容）。
const AGENT_FORMAT_CC: &str = "cc";
const OPENCODE_ZEN_BASE: &str = "https://opencode.ai/zen/v1";
const OPENCODE_ZEN_ENDPOINT: &str = "https://opencode.ai/zen/v1/chat/completions";
const AGENT_PROTOCOL_OPENAI: &str = "openai";
const AGENT_PROTOCOL_RESPONSES: &str = "responses";
const AGENT_PROTOCOL_ANTHROPIC: &str = "anthropic";
const AGENT_PROTOCOL_GEMINI: &str = "gemini";

#[derive(Clone, Copy)]
struct ZenModelPreset {
    id: &'static str,
    label: &'static str,
    protocol: &'static str,
    free: bool,
}

/// OpenCode Zen 模型注册表（显示名、模型 ID、协议、是否免费）。
/// 请求端点由协议自动推导，不要求用户手填。
const OPENCODE_ZEN_MODELS: &[ZenModelPreset] = &[
    ZenModelPreset {
        id: "gpt-5.6-sol",
        label: "GPT 5.6 Sol",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.6-terra",
        label: "GPT 5.6 Terra",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.6-luna",
        label: "GPT 5.6 Luna",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.5",
        label: "GPT 5.5",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.5-pro",
        label: "GPT 5.5 Pro",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.4",
        label: "GPT 5.4",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.4-pro",
        label: "GPT 5.4 Pro",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.4-mini",
        label: "GPT 5.4 Mini",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.4-nano",
        label: "GPT 5.4 Nano",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.3-codex",
        label: "GPT 5.3 Codex",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.3-codex-spark",
        label: "GPT 5.3 Codex Spark",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.2",
        label: "GPT 5.2",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.2-codex",
        label: "GPT 5.2 Codex",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.1",
        label: "GPT 5.1",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.1-codex",
        label: "GPT 5.1 Codex",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.1-codex-max",
        label: "GPT 5.1 Codex Max",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5.1-codex-mini",
        label: "GPT 5.1 Codex Mini",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5",
        label: "GPT 5",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5-codex",
        label: "GPT 5 Codex",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "gpt-5-nano",
        label: "GPT 5 Nano",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "grok-4.5",
        label: "Grok 4.5",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "grok-build-0.1",
        label: "Grok Build 0.1",
        protocol: AGENT_PROTOCOL_RESPONSES,
        free: false,
    },
    ZenModelPreset {
        id: "claude-fable-5",
        label: "Claude Fable 5",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "claude-opus-5",
        label: "Claude Opus 5",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "claude-opus-4-8",
        label: "Claude Opus 4.8",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "claude-opus-4-7",
        label: "Claude Opus 4.7",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "claude-opus-4-6",
        label: "Claude Opus 4.6",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "claude-opus-4-5",
        label: "Claude Opus 4.5",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "claude-sonnet-5",
        label: "Claude Sonnet 5",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "claude-sonnet-4-6",
        label: "Claude Sonnet 4.6",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "claude-sonnet-4-5",
        label: "Claude Sonnet 4.5",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "claude-haiku-4-5",
        label: "Claude Haiku 4.5",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "qwen3.7-max",
        label: "Qwen3.7 Max",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "qwen3.7-plus",
        label: "Qwen3.7 Plus",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "qwen3.6-plus",
        label: "Qwen3.6 Plus",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "qwen3.5-plus",
        label: "Qwen3.5 Plus",
        protocol: AGENT_PROTOCOL_ANTHROPIC,
        free: false,
    },
    ZenModelPreset {
        id: "gemini-3.6-flash",
        label: "Gemini 3.6 Flash",
        protocol: AGENT_PROTOCOL_GEMINI,
        free: false,
    },
    ZenModelPreset {
        id: "gemini-3.5-flash",
        label: "Gemini 3.5 Flash",
        protocol: AGENT_PROTOCOL_GEMINI,
        free: false,
    },
    ZenModelPreset {
        id: "gemini-3.5-flash-lite",
        label: "Gemini 3.5 Flash Lite",
        protocol: AGENT_PROTOCOL_GEMINI,
        free: false,
    },
    ZenModelPreset {
        id: "gemini-3.1-pro",
        label: "Gemini 3.1 Pro",
        protocol: AGENT_PROTOCOL_GEMINI,
        free: false,
    },
    ZenModelPreset {
        id: "gemini-3-flash",
        label: "Gemini 3 Flash",
        protocol: AGENT_PROTOCOL_GEMINI,
        free: false,
    },
    ZenModelPreset {
        id: "deepseek-v4-pro",
        label: "DeepSeek V4 Pro",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: false,
    },
    ZenModelPreset {
        id: "deepseek-v4-flash",
        label: "DeepSeek V4 Flash",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: false,
    },
    ZenModelPreset {
        id: "minimax-m3",
        label: "MiniMax M3",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: false,
    },
    ZenModelPreset {
        id: "minimax-m2.7",
        label: "MiniMax M2.7",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: false,
    },
    ZenModelPreset {
        id: "minimax-m2.5",
        label: "MiniMax M2.5",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: false,
    },
    ZenModelPreset {
        id: "glm-5.2",
        label: "GLM 5.2",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: false,
    },
    ZenModelPreset {
        id: "glm-5.1",
        label: "GLM 5.1",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: false,
    },
    ZenModelPreset {
        id: "glm-5",
        label: "GLM 5",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: false,
    },
    ZenModelPreset {
        id: "kimi-k2.5",
        label: "Kimi K2.5",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: false,
    },
    ZenModelPreset {
        id: "kimi-k2.6",
        label: "Kimi K2.6",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: false,
    },
    ZenModelPreset {
        id: "kimi-k2.7-code",
        label: "Kimi K2.7 Code",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: false,
    },
    ZenModelPreset {
        id: "kimi-k3",
        label: "Kimi K3",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: false,
    },
    ZenModelPreset {
        id: "big-pickle",
        label: "Big Pickle",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: true,
    },
    ZenModelPreset {
        id: "mimo-v2.5-free",
        label: "MiMo-V2.5 Free",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: true,
    },
    ZenModelPreset {
        id: "laguna-s-2.1-free",
        label: "Laguna S 2.1 Free",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: true,
    },
    ZenModelPreset {
        id: "ling-3.0-tiny-free",
        label: "Ling-3.0-tiny Free",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: true,
    },
    ZenModelPreset {
        id: "longcat-2.0-free",
        label: "LongCat-2.0 Free",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: true,
    },
    ZenModelPreset {
        id: "north-mini-code-free",
        label: "North Mini Code Free",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: true,
    },
    ZenModelPreset {
        id: "nemotron-3-ultra-free",
        label: "Nemotron 3 Ultra Free",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: true,
    },
    ZenModelPreset {
        id: "deepseek-v4-flash-free",
        label: "DeepSeek V4 Flash Free",
        protocol: AGENT_PROTOCOL_OPENAI,
        free: true,
    },
];

fn zen_model(model: &str) -> Option<&'static ZenModelPreset> {
    OPENCODE_ZEN_MODELS
        .iter()
        .find(|preset| preset.id == model.trim())
}

fn zen_endpoint(protocol: &str) -> String {
    match protocol {
        AGENT_PROTOCOL_RESPONSES => format!("{}/responses", OPENCODE_ZEN_BASE),
        AGENT_PROTOCOL_ANTHROPIC => format!("{}/messages", OPENCODE_ZEN_BASE),
        AGENT_PROTOCOL_GEMINI => OPENCODE_ZEN_BASE.to_string(),
        _ => format!("{}/chat/completions", OPENCODE_ZEN_BASE),
    }
}

fn zen_models_json(free_only: bool) -> Value {
    Value::Array(
        OPENCODE_ZEN_MODELS
            .iter()
            .filter(|preset| !free_only || preset.free)
            .map(|preset| {
                json!({
                    "id": preset.id,
                    "label": preset.label,
                    "protocol": preset.protocol,
                    "free": preset.free,
                })
            })
            .collect(),
    )
}

struct AgentPreset {
    endpoint: &'static str,
    model: &'static str,
    protocol: &'static str,
    key_account: &'static str,
    /// 是否需要 API Key（false = 免费服务，如 opencode Zen，无需填写 Key）
    needs_api_key: bool,
}

fn agent_preset(provider: &str) -> Option<AgentPreset> {
    match provider {
        AGENT_PROVIDER_GLM => Some(AgentPreset {
            endpoint: "https://open.bigmodel.cn/api/paas/v4/chat/completions",
            model: "glm-4.7-flash",
            protocol: AGENT_PROTOCOL_OPENAI,
            key_account: secret_store::AGENT_GLM_API_KEY,
            needs_api_key: true,
        }),
        AGENT_PROVIDER_OPENAI => Some(AgentPreset {
            endpoint: "https://api.openai.com/v1",
            model: "gpt-4.1-mini",
            protocol: AGENT_PROTOCOL_OPENAI,
            key_account: secret_store::AGENT_OPENAI_API_KEY,
            needs_api_key: true,
        }),
        AGENT_PROVIDER_ANTHROPIC => Some(AgentPreset {
            endpoint: "https://api.anthropic.com/v1/messages",
            model: "claude-sonnet-4-6",
            protocol: AGENT_PROTOCOL_ANTHROPIC,
            key_account: secret_store::AGENT_ANTHROPIC_API_KEY,
            needs_api_key: true,
        }),
        AGENT_PROVIDER_DEEPSEEK => Some(AgentPreset {
            endpoint: "https://api.deepseek.com",
            model: "deepseek-v4-flash",
            protocol: AGENT_PROTOCOL_OPENAI,
            key_account: secret_store::AGENT_DEEPSEEK_API_KEY,
            needs_api_key: true,
        }),
        AGENT_PROVIDER_QWEN => Some(AgentPreset {
            endpoint: "https://dashscope.aliyuncs.com/compatible-mode/v1",
            model: "qwen-plus",
            protocol: AGENT_PROTOCOL_OPENAI,
            key_account: secret_store::AGENT_QWEN_API_KEY,
            needs_api_key: true,
        }),
        AGENT_PROVIDER_GEMINI => Some(AgentPreset {
            endpoint: "https://generativelanguage.googleapis.com/v1beta",
            model: "gemini-3.6-flash",
            protocol: AGENT_PROTOCOL_GEMINI,
            key_account: secret_store::AGENT_GEMINI_API_KEY,
            needs_api_key: true,
        }),
        AGENT_PROVIDER_OPENCODE_ZEN => Some(AgentPreset {
            endpoint: OPENCODE_ZEN_ENDPOINT,
            model: "deepseek-v4-flash",
            protocol: AGENT_PROTOCOL_OPENAI,
            key_account: secret_store::AGENT_API_KEY,
            needs_api_key: true,
        }),
        AGENT_PROVIDER_OPENCODE_ZEN_FREE => Some(AgentPreset {
            endpoint: OPENCODE_ZEN_ENDPOINT,
            model: "deepseek-v4-flash-free",
            protocol: AGENT_PROTOCOL_OPENAI,
            key_account: secret_store::AGENT_API_KEY,
            needs_api_key: false,
        }),
        _ => None,
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    file_path: Option<String>,
    #[serde(default = "default_backup_count")]
    backup_count: usize,
    #[serde(default)]
    backup_path: Option<String>,
    #[serde(default)]
    allow_delete: bool,
    #[serde(default = "default_die_machine_types")]
    die_machine_types: Vec<String>,
    #[serde(default = "default_punch_specs")]
    punch_specs: Vec<String>,
    #[serde(default)]
    agent: AgentConfig,
    /// AI 跨会话记忆：最近若干轮问答摘要（注入后续请求上下文）
    #[serde(default)]
    agent_memory: Vec<String>,
    #[serde(default)]
    webdav: WebDavConfig,
}

fn default_backup_count() -> usize {
    10
}

fn default_die_machine_types() -> Vec<String> {
    ["003", "3/16", "1/4", "6R"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn default_punch_specs() -> Vec<String> {
    ["12*15", "14*15", "18*18"]
        .into_iter()
        .map(String::from)
        .collect()
}

struct AppState {
    file_path: Mutex<String>,
    config: Mutex<Config>,
    config_path: Mutex<PathBuf>,
    webdav_operation: Mutex<()>,
}

fn get_config_path() -> PathBuf {
    // 优先使用用户数据目录（Windows: %APPDATA%\mold-management\config.json），
    // 避免配置文件位于构建目录时被安全软件拦截写入。
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            let dir = PathBuf::from(appdata).join("mold-management");
            let _ = std::fs::create_dir_all(&dir);
            return dir.join("config.json");
        }
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("config.json")))
        .unwrap_or_else(|| PathBuf::from("config.json"))
}

/// 从旧位置（exe 同目录 config.json）迁移配置到新位置（用户数据目录）。
fn migrate_legacy_config(new_path: &Path) {
    if new_path.exists() {
        return;
    }
    let legacy = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("config.json")));
    if let Some(legacy) = legacy {
        if legacy != new_path && legacy.exists() {
            if let Some(parent) = new_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::copy(&legacy, new_path);
        }
    }
}

/// 规范化路径：消除中间的 `..`/`.` 段（如 `src-tauri/../data` → `data`）。
/// 不依赖文件系统（备份目录可能尚不存在），纯组件级归一化，保持路径语义不变。
/// 只弹出普通目录组件，不会误删根目录/盘符前缀。
fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut parts: Vec<Component> = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if parts
                    .last()
                    .is_some_and(|last| matches!(last, Component::Normal(_)))
                {
                    parts.pop();
                }
            }
            other => parts.push(other),
        }
    }
    let mut out = PathBuf::new();
    for (index, component) in parts.iter().enumerate() {
        if index == 0 {
            out = PathBuf::from(component.as_os_str());
        } else {
            out.push(component.as_os_str());
        }
    }
    out
}

fn normalized_string(path: &Path) -> String {
    normalize_path(path).to_string_lossy().to_string()
}

fn load_config(config_path: &Path) -> Config {
    if let Ok(content) = fs::read_to_string(config_path) {
        let mut config = serde_json::from_str::<Config>(&content).unwrap_or(Config {
            file_path: None,
            backup_count: 10,
            backup_path: None,
            allow_delete: false,
            die_machine_types: default_die_machine_types(),
            punch_specs: default_punch_specs(),
            agent: AgentConfig::default(),
            agent_memory: Vec::new(),
            webdav: WebDavConfig::default(),
        });
        migrate_legacy_agent_config(&mut config, &content);
        config
    } else {
        Config {
            file_path: None,
            backup_count: 10,
            backup_path: None,
            allow_delete: false,
            die_machine_types: default_die_machine_types(),
            punch_specs: default_punch_specs(),
            agent: AgentConfig::default(),
            agent_memory: Vec::new(),
            webdav: WebDavConfig::default(),
        }
    }
}

/// 旧版单份 agent 配置（provider/endpoint/model/api_key_encrypted）迁移为配置档案。
/// 旧 API Key 若已存在系统凭据库则复制到新档案账户；DPAPI 密文无法解密时由用户重新填写。
fn migrate_legacy_agent_config(config: &mut Config, raw_content: &str) {
    if !config.agent.profiles.is_empty() {
        return;
    }
    let Ok(root) = serde_json::from_str::<Value>(raw_content) else {
        return;
    };
    let Some(legacy) = root.get("agent") else {
        return;
    };
    let provider = legacy
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let endpoint = legacy
        .get("endpoint")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let model = legacy
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if provider.is_empty() && endpoint.is_empty() && model.is_empty() {
        return;
    }
    let kind = if agent_preset(&provider).is_some() {
        "builtin".to_string()
    } else {
        "custom".to_string()
    };
    let format = if kind == "custom" && provider == AGENT_PROVIDER_CUSTOM_ANTHROPIC {
        AGENT_FORMAT_CC.to_string()
    } else {
        AGENT_PROTOCOL_OPENAI.to_string()
    };
    let id = format!("legacy-{}", provider);
    let legacy_key_account = if kind == "builtin" {
        agent_preset(&provider)
            .map(|preset| preset.key_account.to_string())
            .unwrap_or_default()
    } else if provider == AGENT_PROVIDER_CUSTOM_ANTHROPIC {
        secret_store::AGENT_CUSTOM_ANTHROPIC_API_KEY.to_string()
    } else {
        secret_store::AGENT_API_KEY.to_string()
    };
    if let Ok(key) = secret_store::get(&legacy_key_account) {
        let _ = secret_store::set(&profile_key_account(&id), &key);
    }
    // 兼容旧版 DPAPI 密文：仅 Windows 且可解密时迁移
    if let Some(encrypted) = legacy
        .get("api_key_encrypted")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    {
        if let Ok(key) = secret_store::unprotect_legacy(encrypted) {
            let _ = secret_store::set(&profile_key_account(&id), &key);
        }
    }
    config.agent.profiles.push(AgentProfile {
        id: id.clone(),
        name: String::new(),
        kind,
        provider,
        format,
        endpoint,
        model,
    });
    config.agent.active = id;
}

fn save_config(config_path: &Path, config: &Config) -> Result<(), String> {
    storage::atomic_write_json(config_path, config)
        .map_err(|e| format!("保存应用配置失败「{}」: {}", config_path.display(), e))
}

fn mask_username(username: &str) -> String {
    let username = username.trim();
    if let Some((name, domain)) = username.split_once('@') {
        let first = name.chars().next().unwrap_or('*');
        return format!("{}***@{}", first, domain);
    }
    let first = username.chars().next().unwrap_or('*');
    format!("{}***", first)
}

fn read_webdav_secret(config: &Config) -> Result<(String, String), String> {
    if let Ok(stored) = secret_store::get(secret_store::WEBDAV_CREDENTIALS) {
        let value: Value = serde_json::from_str(&stored)
            .map_err(|e| format!("解析系统凭据库中的 WebDAV 凭据失败: {}", e))?;
        let username = value
            .get("username")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let password = value
            .get("password")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !username.is_empty() && !password.is_empty() {
            return Ok((username.to_string(), password.to_string()));
        }
    }
    if !config.webdav.username_encrypted.is_empty() && !config.webdav.password_encrypted.is_empty()
    {
        return Ok((
            secret_store::unprotect_legacy(&config.webdav.username_encrypted)?,
            secret_store::unprotect_legacy(&config.webdav.password_encrypted)?,
        ));
    }
    Err("WebDAV 凭据尚未保存到系统凭据库".to_string())
}

fn resolved_webdav_credentials(config: &Config) -> Result<webdav_sync::Credentials, String> {
    if let Ok((username, password)) = read_webdav_secret(config) {
        return Ok(webdav_sync::Credentials {
            url: config.webdav.url.clone(),
            remote_path: config.webdav.remote_path.clone(),
            username,
            password,
        });
    }
    let mut development = webdav_sync::development_credentials().ok_or_else(|| {
        "WebDAV 尚未配置；开发环境可使用项目根目录 .dev，正式使用请在设置页保存凭据".to_string()
    })?;
    if !config.webdav.url.is_empty() {
        development.url = config.webdav.url.clone();
    }
    if !config.webdav.remote_path.is_empty() {
        development.remote_path = config.webdav.remote_path.clone();
    }
    Ok(development)
}

fn persist_config(state: &AppState, next: Config) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let config_path = state.config_path.lock().map_err(|e| e.to_string())?;
    save_config(&config_path, &next)?;
    *config = next;
    Ok(())
}

#[tauri::command]
fn get_webdav_config(state: State<AppState>) -> Result<Value, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let credentials = resolved_webdav_credentials(&config).ok();
    let username_masked = credentials
        .as_ref()
        .map(|value| mask_username(&value.username))
        .unwrap_or_default();
    let url = if config.webdav.url.is_empty() {
        credentials
            .as_ref()
            .map(|value| value.url.clone())
            .unwrap_or_default()
    } else {
        config.webdav.url.clone()
    };
    let remote_path = if config.webdav.remote_path.is_empty() {
        credentials
            .as_ref()
            .map(|value| value.remote_path.clone())
            .unwrap_or_else(webdav_sync::default_remote_path)
    } else {
        config.webdav.remote_path.clone()
    };
    let native_credentials_configured = secret_store::exists(secret_store::WEBDAV_CREDENTIALS)
        || (cfg!(windows)
            && !config.webdav.username_encrypted.is_empty()
            && !config.webdav.password_encrypted.is_empty());
    Ok(json!({
        "url": url,
        "remotePath": remote_path,
        "usernameMasked": username_masked,
        "credentialsConfigured": credentials.is_some(),
        "credentialStore": secret_store::platform_store_name(),
        "usingDevelopmentConfig": !native_credentials_configured && credentials.is_some(),
        "autoUploadOnStart": config.webdav.auto_upload_on_start,
        "autoUploadOnExit": config.webdav.auto_upload_on_exit,
        "lastEtag": config.webdav.last_etag,
        "lastUploadedAt": config.webdav.last_uploaded_at,
        "lastDownloadedAt": config.webdav.last_downloaded_at,
    }))
}

#[tauri::command]
fn set_webdav_config(
    state: State<AppState>,
    url: String,
    remote_path: String,
    username: Option<String>,
    password: Option<String>,
    auto_upload_on_start: bool,
    auto_upload_on_exit: bool,
) -> Result<Value, String> {
    webdav_sync::validate_settings(&url, &remote_path)?;
    let _operation = state
        .webdav_operation
        .lock()
        .map_err(|e| format!("WebDAV 同步操作锁定失败: {}", e))?;
    let mut next = state.config.lock().map_err(|e| e.to_string())?.clone();
    let username = username
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let password = password.filter(|value| !value.is_empty());
    if username.is_some() != password.is_some() {
        return Err("更新 WebDAV 凭据时必须同时填写账户和应用密码".to_string());
    }
    if let (Some(username), Some(password)) = (username, password) {
        let stored = serde_json::to_string(&json!({ "username": username, "password": password }))
            .map_err(|e| format!("生成 WebDAV 凭据失败: {}", e))?;
        secret_store::set(secret_store::WEBDAV_CREDENTIALS, &stored)?;
        next.webdav.username_encrypted.clear();
        next.webdav.password_encrypted.clear();
        next.webdav.last_etag = None;
    } else if !secret_store::exists(secret_store::WEBDAV_CREDENTIALS)
        && (next.webdav.username_encrypted.is_empty() || next.webdav.password_encrypted.is_empty())
        && webdav_sync::development_credentials().is_none()
    {
        return Err("首次配置 WebDAV 时必须填写账户和应用密码".to_string());
    }
    let normalized_url = url.trim().to_string();
    let normalized_remote_path = remote_path.trim().trim_start_matches('/').to_string();
    if next.webdav.url != normalized_url || next.webdav.remote_path != normalized_remote_path {
        next.webdav.last_etag = None;
    }
    next.webdav.url = normalized_url;
    next.webdav.remote_path = normalized_remote_path;
    next.webdav.auto_upload_on_start = auto_upload_on_start;
    next.webdav.auto_upload_on_exit = auto_upload_on_exit;
    persist_config(&state, next)?;
    Ok(json!({ "success": true }))
}

#[tauri::command]
async fn test_webdav_connection(state: State<'_, AppState>) -> Result<Value, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let credentials = resolved_webdav_credentials(&config)?;
    tauri::async_runtime::spawn_blocking(move || {
        serde_json::to_value(webdav_sync::test_connection(&credentials)?)
            .map_err(|e| format!("序列化 WebDAV 状态失败: {}", e))
    })
    .await
    .map_err(|e| format!("执行 WebDAV 连接测试失败: {}", e))?
}

#[tauri::command]
async fn get_webdav_status(state: State<'_, AppState>) -> Result<Value, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let credentials = resolved_webdav_credentials(&config)?;
    tauri::async_runtime::spawn_blocking(move || {
        serde_json::to_value(webdav_sync::remote_status(&credentials)?)
            .map_err(|e| format!("序列化 WebDAV 状态失败: {}", e))
    })
    .await
    .map_err(|e| format!("查询 WebDAV 远端状态失败: {}", e))?
}

fn upload_webdav_snapshot(app_state: &AppState, force_overwrite: bool) -> Result<Value, String> {
    let _operation = app_state
        .webdav_operation
        .lock()
        .map_err(|e| format!("WebDAV 同步操作锁定失败: {}", e))?;
    let file_path = app_state
        .file_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let config = app_state.config.lock().map_err(|e| e.to_string())?.clone();
    let credentials = resolved_webdav_credentials(&config)?;
    let config_component = serde_json::to_vec_pretty(&json!({
        "format": "mold-management-portable-config",
        "version": 1,
        "allowDelete": config.allow_delete,
        "dieMachineTypes": config.die_machine_types,
        "punchSpecs": config.punch_specs,
    }))
    .map_err(|e| format!("生成 WebDAV 配置组件失败: {}", e))?;
    let document_content_id = db::content_sha256(&file_path)?;
    let document_component = data_package::export_document_snapshot(&file_path)?;
    let attachments_component = data_package::export_attachments_package(&file_path)?;
    let result = webdav_sync::upload(
        &credentials,
        vec![
            webdav_sync::UploadComponent {
                kind: webdav_sync::RemoteComponent::Config,
                data: config_component,
                content_id: None,
            },
            webdav_sync::UploadComponent {
                kind: webdav_sync::RemoteComponent::Document,
                data: document_component,
                content_id: Some(document_content_id),
            },
            webdav_sync::UploadComponent {
                kind: webdav_sync::RemoteComponent::Attachments,
                data: attachments_component,
                content_id: None,
            },
        ],
        config.webdav.last_etag.as_deref(),
        force_overwrite,
    )?;
    let mut next = config;
    next.webdav.last_etag = Some(
        result
            .etag
            .clone()
            .unwrap_or_else(|| "__NO_ETAG__".to_string()),
    );
    next.webdav.last_uploaded_at = Some(result.uploaded_at.clone());
    persist_config(app_state, next)?;
    serde_json::to_value(result).map_err(|e| format!("序列化 WebDAV 上传结果失败: {}", e))
}

#[tauri::command]
async fn upload_webdav_now(app: tauri::AppHandle, force_overwrite: bool) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        upload_webdav_snapshot(&state, force_overwrite)
    })
    .await
    .map_err(|e| format!("执行 WebDAV 上传失败: {}", e))?
}

#[tauri::command]
async fn download_webdav_now(app: tauri::AppHandle) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let _operation = state
            .webdav_operation
            .lock()
            .map_err(|e| format!("WebDAV 同步操作锁定失败: {}", e))?;
        let file_path = state.file_path.lock().map_err(|e| e.to_string())?.clone();
        let config = state.config.lock().map_err(|e| e.to_string())?.clone();
        let credentials = resolved_webdav_credentials(&config)?;
        let downloaded = webdav_sync::download(&credentials)?;
        let backup_dir = get_backup_dir_for_file(&file_path, &config);
        do_backup(&file_path, &backup_dir, "WebDAV 恢复前备份")?;
        let import_result = data_package::import_split_package(
            &file_path,
            &downloaded.document,
            &downloaded.attachments,
        )?;
        cleanup_old_backups(&backup_dir, config.backup_count)?;
        let portable_config: Value = serde_json::from_slice(&downloaded.config)
            .map_err(|e| format!("解析 WebDAV 配置组件失败: {}", e))?;
        if portable_config.get("format").and_then(Value::as_str)
            != Some("mold-management-portable-config")
            || portable_config.get("version").and_then(Value::as_u64) != Some(1)
        {
            return Err("WebDAV 配置组件格式无效".to_string());
        }
        let mut next = config;
        if let Some(value) = portable_config.get("allowDelete").and_then(Value::as_bool) {
            next.allow_delete = value;
        }
        if let Some(values) = portable_config
            .get("dieMachineTypes")
            .and_then(Value::as_array)
        {
            next.die_machine_types = values
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect();
        }
        if let Some(values) = portable_config.get("punchSpecs").and_then(Value::as_array) {
            next.punch_specs = values
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect();
        }
        next.webdav.last_etag = Some(
            downloaded
                .etag
                .clone()
                .unwrap_or_else(|| "__NO_ETAG__".to_string()),
        );
        next.webdav.last_downloaded_at = Some(Local::now().to_rfc3339());
        persist_config(&state, next)?;
        Ok(json!({
            "success": true,
            "etag": downloaded.etag,
            "sha256": downloaded.sha256,
            "checksumVerified": true,
            "importResult": import_result,
        }))
    })
    .await
    .map_err(|e| format!("执行 WebDAV 下载恢复失败: {}", e))?
}

#[tauri::command]
fn get_all_records(state: State<AppState>, sheet_name: String) -> Result<Vec<Value>, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let items = excel::get_all(&path, &sheet_name)?;
    Ok(items
        .into_iter()
        .map(|m| serde_json::to_value(m).unwrap_or(json!({})))
        .collect())
}

#[tauri::command]
fn get_record(
    state: State<AppState>,
    sheet_name: String,
    id: String,
) -> Result<Option<Value>, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let item = excel::get_by_id(&path, &sheet_name, &id)?;
    Ok(item.map(|m| serde_json::to_value(m).unwrap_or(json!({}))))
}

#[tauri::command]
fn add_record(state: State<AppState>, sheet_name: String, item: Value) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    // 将所有值转为字符串，数组用逗号连接
    let map: std::collections::HashMap<String, String> = item
        .as_object()
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| {
                    let val = match v {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => String::new(),
                        Value::Array(arr) => arr
                            .iter()
                            .filter_map(|item| match item {
                                Value::String(s) => Some(s.clone()),
                                Value::Number(n) => Some(n.to_string()),
                                Value::Bool(b) => Some(b.to_string()),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .join(","),
                        _ => v.to_string(),
                    };
                    (k.clone(), val)
                })
                .collect()
        })
        .ok_or("无效的数据格式")?;
    let result = excel::add_row(&path, &sheet_name, &map)?;
    Ok(serde_json::to_value(result).unwrap_or(json!({})))
}

#[tauri::command]
fn update_record(
    state: State<AppState>,
    sheet_name: String,
    id: String,
    data: Value,
) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    // 将所有值转为字符串，数组用逗号连接
    let map: std::collections::HashMap<String, String> = data
        .as_object()
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| {
                    let val = match v {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => String::new(),
                        Value::Array(arr) => arr
                            .iter()
                            .filter_map(|item| match item {
                                Value::String(s) => Some(s.clone()),
                                Value::Number(n) => Some(n.to_string()),
                                Value::Bool(b) => Some(b.to_string()),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .join(","),
                        _ => v.to_string(),
                    };
                    (k.clone(), val)
                })
                .collect()
        })
        .ok_or("无效的数据格式")?;
    let result = excel::update_row(&path, &sheet_name, &id, &map)?;
    Ok(serde_json::to_value(result).unwrap_or(json!({})))
}

#[tauri::command]
fn delete_record(state: State<AppState>, sheet_name: String, id: String) -> Result<bool, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let deleted = excel::delete_row(&path, &sheet_name, &id)?;
    if deleted && sheet_name == "螺丝规格表" {
        attachments::delete_for_screw(&path, &id)?;
    }
    Ok(deleted)
}

#[tauri::command]
fn get_operation_logs(state: State<AppState>, limit: i64, offset: i64) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let conn = db::connect(&path)?;
    log::ensure_table(&conn)?;
    let total = log::count_logs(&conn)?;
    let items = log::get_logs(&conn, limit.max(1).min(1000), offset.max(0))?;
    Ok(json!({ "total": total, "items": items }))
}

#[tauri::command]
fn clear_operation_logs(state: State<AppState>) -> Result<bool, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let conn = db::connect(&path)?;
    log::clear_logs(&conn)?;
    Ok(true)
}

#[tauri::command]
fn list_screw_attachments(state: State<AppState>, screw_spec_id: String) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let items = attachments::list(&path, &screw_spec_id)?;
    serde_json::to_value(items).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_screw_attachment_counts(state: State<AppState>) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let counts = attachments::counts(&path)?;
    serde_json::to_value(counts).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_screw_attachment(
    state: State<AppState>,
    screw_spec_id: String,
    source_path: String,
) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let item = attachments::import(&path, &screw_spec_id, &source_path)?;
    serde_json::to_value(item).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_screw_attachment(state: State<AppState>, attachment_id: String) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    attachments::read_content(&path, &attachment_id)
}

#[tauri::command]
fn update_screw_attachment(
    state: State<AppState>,
    attachment_id: String,
    display_name: Option<String>,
    annotations: Option<Vec<Value>>,
    sort_order: Option<usize>,
) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let item = attachments::update(&path, &attachment_id, display_name, annotations, sort_order)?;
    serde_json::to_value(item).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_screw_attachment(state: State<AppState>, attachment_id: String) -> Result<bool, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    attachments::delete(&path, &attachment_id)
}

#[tauri::command]
fn export_data(state: State<AppState>) -> Result<Value, String> {
    // 兼容旧入口：导出全部业务表到单个 Excel（新流程请使用 export_excel_group）
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let bytes = excel::export_data(&path)?;
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(json!({ "filename": "mold-data.xlsx", "data": b64 }))
}

#[tauri::command]
fn export_excel_group(
    state: State<AppState>,
    group_id: String,
    destination_path: String,
) -> Result<Value, String> {
    let destination = Path::new(&destination_path);
    if !destination
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("xlsx"))
    {
        return Err("Excel 文件必须使用 .xlsx 扩展名".to_string());
    }
    let group = excel::EXPORT_GROUPS
        .iter()
        .find(|(id, _)| *id == group_id)
        .ok_or_else(|| "未知的导出分组".to_string())?;
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let bytes = excel::export_group_xlsx(&path, group.1)?;
    storage::atomic_write(destination, &bytes)
        .map_err(|e| format!("写出 Excel 失败「{}」: {}", destination.display(), e))?;
    Ok(json!({ "success": true, "filePath": destination_path, "group": group_id }))
}

#[tauri::command]
fn list_excel_sheets(source_path: String) -> Result<Value, String> {
    let sheets = excel::list_excel_sheets(&source_path)?;
    serde_json::to_value(sheets).map_err(|e| format!("序列化工作表信息失败: {}", e))
}

#[tauri::command]
fn import_excel_sheets(
    state: State<AppState>,
    source_path: String,
    selections: Vec<Value>,
) -> Result<Value, String> {
    if selections.is_empty() {
        return Err("请至少选择一个要导入的工作表".to_string());
    }
    let mut pairs = Vec::new();
    for selection in selections {
        let name = selection
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "导入选择缺少工作表名".to_string())?;
        let table = selection
            .get("table")
            .and_then(Value::as_str)
            .ok_or_else(|| "导入选择缺少目标表名".to_string())?;
        pairs.push((name.to_string(), table.to_string()));
    }
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir_for_file(&path, &config);
    let count = config.backup_count;
    do_backup(&path, &backup_dir, "Excel 导入前备份")?;
    let stats = excel::import_sheets_from_xlsx(&path, &source_path, &pairs)?;
    cleanup_old_backups(&backup_dir, count)?;
    Ok(json!({ "success": true, "stats": stats }))
}

#[tauri::command]
fn export_data_package(state: State<AppState>, destination_path: String) -> Result<Value, String> {
    let destination = Path::new(&destination_path);
    if !destination
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("moldpkg"))
    {
        return Err("完整数据包文件必须使用 .moldpkg 扩展名".to_string());
    }
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let bytes = data_package::export_package(&path)?;
    storage::atomic_write(destination, &bytes)
        .map_err(|e| format!("写出完整数据包失败「{}」: {}", destination.display(), e))?;
    Ok(json!({ "success": true, "filePath": destination_path }))
}

#[tauri::command]
fn import_data_package(state: State<AppState>, source_path: String) -> Result<Value, String> {
    let source = Path::new(&source_path);
    if !source.is_file() {
        return Err(format!("完整数据包不存在「{}」", source.display()));
    }
    if !source
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("moldpkg"))
    {
        return Err("完整数据包文件必须使用 .moldpkg 扩展名".to_string());
    }
    let bytes = fs::read(source)
        .map_err(|e| format!("读取完整数据包失败「{}」: {}", source.display(), e))?;
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir_for_file(&path, &config);
    let count = config.backup_count;
    do_backup(&path, &backup_dir, "完整数据包导入前备份")?;
    let result = data_package::import_package(&path, &bytes)?;
    cleanup_old_backups(&backup_dir, count)?;
    Ok(result)
}

#[tauri::command]
fn import_data(state: State<AppState>, data: String) -> Result<Value, String> {
    // 兼容旧入口：解析上传的 Excel 并导入全部可识别工作表
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| format!("解析导入文件失败: {}", e))?;
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir_for_file(&path, &config);
    let count = config.backup_count;
    let temporary = storage::temporary_path(Path::new(&*path), "xlsx")?;
    fs::write(&temporary, &bytes).map_err(|e| format!("写入导入暂存文件失败: {}", e))?;
    let sheets = excel::list_excel_sheets(&temporary.to_string_lossy())?;
    let pairs: Vec<(String, String)> = sheets
        .into_iter()
        .filter(|info| !info.system_calculated)
        .map(|info| (info.name, info.table))
        .collect();
    do_backup(&path, &backup_dir, "Excel 导入前备份")?;
    let stats = excel::import_sheets_from_xlsx(&path, &temporary.to_string_lossy(), &pairs)?;
    let _ = fs::remove_file(&temporary);
    cleanup_old_backups(&backup_dir, count)?;
    Ok(json!({ "success": true, "stats": stats }))
}

#[tauri::command]
fn get_file_path_cmd(state: State<AppState>) -> Result<String, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    Ok(normalized_string(Path::new(path.as_str())))
}

#[tauri::command]
fn set_file_path(state: State<AppState>, path: String) -> Result<Value, String> {
    let target = Path::new(&path);
    if !target.is_file() {
        return Err(format!("所选数据文件不存在「{}」", path));
    }
    excel::validate_workbook(target)?;

    // 与其他同时读取 file_path/config 的命令保持统一锁顺序，避免反向加锁。
    // 配置文件写入成功前不修改内存状态，写入失败时两者都保持原值。
    let mut file_path = state.file_path.lock().map_err(|e| e.to_string())?;
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let config_path = state.config_path.lock().map_err(|e| e.to_string())?;
    let mut next_config = config.clone();
    next_config.file_path = Some(path.clone());
    save_config(&config_path, &next_config)?;
    *file_path = path.clone();
    *config = next_config;

    Ok(json!({ "success": true, "filePath": path }))
}

#[tauri::command]
fn calculate_stock(state: State<AppState>, stock_type: String) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    if stock_type == "all" {
        let types = ["punch", "die", "belt", "mainMold", "scissor", "upperPunch"];
        let mut results = serde_json::Map::new();
        for t in types {
            let data = excel::calculate_stock(&path, t)?;
            results.insert(
                t.to_string(),
                serde_json::to_value(data).unwrap_or(json!([])),
            );
        }
        Ok(Value::Object(results))
    } else {
        let data = excel::calculate_stock(&path, &stock_type)?;
        Ok(serde_json::to_value(data).unwrap_or(json!([])))
    }
}

#[tauri::command]
fn backup_data(state: State<AppState>) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir_for_file(&path, &config);
    let count = config.backup_count;
    let backup_file = do_backup(&path, &backup_dir, "手动备份")?;
    cleanup_old_backups(&backup_dir, count)?;
    if backup_file.is_empty() {
        Ok(json!({ "success": true, "skipped": true, "message": "文件内容未变化，跳过备份" }))
    } else {
        Ok(json!({ "success": true, "backupPath": backup_file }))
    }
}

#[tauri::command]
fn get_backup_config(state: State<AppState>) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir_for_file(&path, &config);
    let default_dir = Path::new(&*path)
        .parent()
        .map(|parent| parent.join("backups").to_string_lossy().to_string())
        .unwrap_or_else(default_backup_dir);
    Ok(json!({
        "backupCount": config.backup_count,
        "backupPath": config.backup_path,
        "defaultBackupDir": default_dir,
        "effectiveBackupDir": backup_dir,
    }))
}

#[tauri::command]
fn set_backup_config(
    state: State<AppState>,
    backup_count: usize,
    backup_path: Option<String>,
) -> Result<Value, String> {
    if !(1..=100).contains(&backup_count) {
        return Err("备份保留数量必须在 1 到 100 之间".to_string());
    }
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let previous_count = config.backup_count;
    let previous_path = config.backup_path.clone();
    config.backup_count = backup_count;
    config.backup_path = backup_path;
    let config_path = state.config_path.lock().map_err(|e| e.to_string())?;
    if let Err(error) = save_config(&config_path, &config) {
        config.backup_count = previous_count;
        config.backup_path = previous_path;
        return Err(error);
    }
    Ok(json!({ "success": true }))
}

#[tauri::command]
fn get_allow_delete(state: State<AppState>) -> Result<bool, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.allow_delete)
}

#[tauri::command]
fn set_allow_delete(state: State<AppState>, allow: bool) -> Result<Value, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let previous = config.allow_delete;
    config.allow_delete = allow;
    let config_path = state.config_path.lock().map_err(|e| e.to_string())?;
    if let Err(error) = save_config(&config_path, &config) {
        config.allow_delete = previous;
        return Err(error);
    }
    Ok(json!({ "success": true }))
}

#[tauri::command]
fn get_die_machine_types(state: State<AppState>) -> Result<Vec<String>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.die_machine_types.clone())
}

#[tauri::command]
fn set_die_machine_types(
    state: State<AppState>,
    machine_types: Vec<String>,
) -> Result<Value, String> {
    let mut normalized = Vec::new();
    for machine_type in machine_types {
        let value = machine_type.trim();
        if value.is_empty() {
            continue;
        }
        if value.chars().count() > 40 {
            return Err(format!("机型「{}」长度不能超过 40 个字符", value));
        }
        if !normalized
            .iter()
            .any(|item: &String| item.eq_ignore_ascii_case(value))
        {
            normalized.push(value.to_string());
        }
    }
    if normalized.is_empty() {
        return Err("牙板机型列表至少保留一个选项".to_string());
    }
    if normalized.len() > 100 {
        return Err("牙板机型列表最多允许 100 个选项".to_string());
    }

    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let previous = config.die_machine_types.clone();
    config.die_machine_types = normalized.clone();
    let config_path = state.config_path.lock().map_err(|e| e.to_string())?;
    if let Err(error) = save_config(&config_path, &config) {
        config.die_machine_types = previous;
        return Err(error);
    }
    Ok(json!({ "success": true, "machineTypes": normalized }))
}

#[tauri::command]
fn get_punch_specs(state: State<AppState>) -> Result<Vec<String>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.punch_specs.clone())
}

#[tauri::command]
fn set_punch_specs(state: State<AppState>, specs: Vec<String>) -> Result<Value, String> {
    let mut normalized = Vec::new();
    for spec in specs {
        let value = spec.trim();
        if value.is_empty() {
            continue;
        }
        if value.chars().count() > 40 {
            return Err(format!("规格「{}」长度不能超过 40 个字符", value));
        }
        if !normalized
            .iter()
            .any(|item: &String| item.eq_ignore_ascii_case(value))
        {
            normalized.push(value.to_string());
        }
    }
    if normalized.is_empty() {
        return Err("冲头规格列表至少保留一个选项".to_string());
    }
    if normalized.len() > 100 {
        return Err("冲头规格列表最多允许 100 个选项".to_string());
    }

    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    let previous = config.punch_specs.clone();
    config.punch_specs = normalized.clone();
    let config_path = state.config_path.lock().map_err(|e| e.to_string())?;
    if let Err(error) = save_config(&config_path, &config) {
        config.punch_specs = previous;
        return Err(error);
    }
    Ok(json!({ "success": true, "specs": normalized }))
}

fn agent_writable_columns(sheet_name: &str) -> Result<Vec<&'static str>, String> {
    if sheet_name.ends_with("库存汇总") {
        return Err("库存汇总为系统计算结果，不能直接修改".to_string());
    }
    excel::SHEETS
        .iter()
        .find(|(name, _)| *name == sheet_name)
        .map(|(_, columns)| {
            columns
                .iter()
                .map(|(_, key)| *key)
                .filter(|key| *key != "id")
                .collect()
        })
        .ok_or_else(|| format!("未知业务表「{}」", sheet_name))
}

fn normalize_agent_change(file_path: &str, change: &Value) -> Result<Value, String> {
    let mut operation = change
        .get("operation")
        .and_then(Value::as_str)
        .ok_or_else(|| "AI 变更缺少 operation".to_string())?
        .to_string();
    if !matches!(
        operation.as_str(),
        "add" | "update" | "delete" | "set_setting" | "import"
    ) {
        return Err(
            "AI 变更 operation 只能是 add、update、delete、set_setting 或 import".to_string(),
        );
    }
    let fields = change
        .get("fields")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if operation == "set_setting" {
        if fields.is_empty() {
            return Err("AI 系统设置变更缺少字段".to_string());
        }
        if let Some(theme) = fields.get("theme") {
            match theme {
                Value::String(value) if matches!(value.as_str(), "light" | "dark" | "system") => {}
                _ => return Err("主题设置值只能是 light、dark 或 system".to_string()),
            }
        }
        return Ok(json!({
            "operation": "set_setting",
            "table": "system_settings",
            "id": "",
            "fields": fields,
            "before": null,
            "after": fields,
        }));
    }
    if operation == "import" {
        let file_path_value = fields
            .get("file_path")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if file_path_value.trim().is_empty() {
            return Err("导入操作必须提供 file_path".to_string());
        }
        let table = change
            .get("table")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        return Ok(json!({
            "operation": "import",
            "table": table,
            "id": "",
            "fields": fields,
            "before": null,
            "after": null,
        }));
    }
    let table = change
        .get("table")
        .and_then(Value::as_str)
        .ok_or_else(|| "AI 变更缺少 table".to_string())?;
    let allowed_columns = agent_writable_columns(table)?;
    let id = change
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    if operation != "delete" && fields.is_empty() {
        return Err("AI 变更没有可写字段".to_string());
    }
    for key in fields.keys() {
        if !allowed_columns.contains(&key.as_str()) {
            return Err(format!("表「{}」不允许修改字段「{}」", table, key));
        }
    }
    if operation != "add" && id.is_empty() {
        return Err("更新或删除操作必须提供记录 id".to_string());
    }
    // 防御：AI 误用 add 创建与现有记录同 id 的新记录时，自动转为 update（合并字段），避免产生重复记录。
    if operation == "add" && !id.is_empty() && excel::get_by_id(file_path, table, &id)?.is_some() {
        operation = "update".to_string();
    }
    let before = if operation == "add" {
        Value::Null
    } else {
        excel::get_by_id(file_path, table, &id)?
            .map(|row| serde_json::to_value(row).unwrap_or(Value::Null))
            .ok_or_else(|| format!("表「{}」中未找到记录「{}」", table, id))?
    };
    let after = if operation == "delete" {
        Value::Null
    } else if operation == "add" {
        Value::Object(fields.clone())
    } else {
        let mut candidate = before.as_object().cloned().unwrap_or_default();
        for (key, value) in &fields {
            candidate.insert(key.clone(), value.clone());
        }
        Value::Object(candidate)
    };
    Ok(json!({
        "operation": operation,
        "table": table,
        "id": id,
        "fields": fields,
        "before": before,
        "after": after,
    }))
}

fn build_agent_context(file_path: &str, memory: &[String]) -> Result<String, String> {
    let schema: Vec<Value> = excel::SHEETS
        .iter()
        .map(|(name, columns)| json!({
            "table": name,
            "readOnly": name.ends_with("库存汇总"),
            "columns": columns.iter().map(|(label, key)| json!({ "key": key, "label": label })).collect::<Vec<_>>()
        }))
        .collect();
    // 一次性读取全部表（本地 SQLite，毫秒级），避免重复建连。
    let mut rows_by_sheet: Vec<(&'static str, Vec<std::collections::HashMap<String, String>>)> =
        Vec::with_capacity(excel::SHEETS.len());
    for (sheet_name, _) in excel::SHEETS {
        let rows = excel::get_all(file_path, sheet_name)?;
        rows_by_sheet.push((sheet_name, rows));
    }
    let is_summary = |name: &str| name.ends_with("库存汇总");
    let is_record = |name: &str| name.ends_with("记录") || name.ends_with("关联");
    // 汇总表排在最前：库存查询的核心数据，且行数 = 物料数，业务上有限，尽量全量给 AI。
    let mut order: Vec<usize> = (0..rows_by_sheet.len()).collect();
    order.sort_by_key(|&i| !is_summary(rows_by_sheet[i].0));

    let mut remaining = 500usize;
    let mut data = serde_json::Map::new();
    let mut totals = serde_json::Map::new();
    for index in order {
        if remaining == 0 {
            break;
        }
        let (sheet_name, rows) = &rows_by_sheet[index];
        let total = rows.len();
        totals.insert((*sheet_name).to_string(), json!(total));
        // 汇总表全量；记录/关联表最多 40 条；信息表最多 120 条；均取最新（倒序）。
        let max_rows = if is_summary(sheet_name) {
            total
        } else if is_record(sheet_name) {
            40.min(total)
        } else {
            120.min(total)
        };
        let take = max_rows.min(remaining);
        remaining = remaining.saturating_sub(take);
        let selected: Vec<_> = rows.iter().rev().take(take).cloned().collect();
        data.insert(
            (*sheet_name).to_string(),
            serde_json::to_value(selected).unwrap_or(json!([])),
        );
    }
    // 新品配模推荐证据图：完整保留历史螺丝 → 已验证冲头/牙板关联，并附带库存。
    // 这部分不受上面的 500 行通用上下文预算限制，避免关联表被截断后 AI 误判“无可推荐”。
    let sheet_rows = |name: &str| {
        rows_by_sheet
            .iter()
            .find(|(sheet, _)| *sheet == name)
            .map(|(_, rows)| rows.as_slice())
            .unwrap_or(&[])
    };
    let screws = sheet_rows("螺丝规格表");
    let punches = sheet_rows("冲头信息表");
    let dies = sheet_rows("牙板信息表");
    let punch_links = sheet_rows("冲头-螺丝规格关联");
    let die_links = sheet_rows("牙板-螺丝规格关联");
    let punch_stock = sheet_rows("冲头库存汇总");
    let die_stock = sheet_rows("牙板库存汇总");
    let main_molds = sheet_rows("主模具信息表");
    let main_mold_links = sheet_rows("主模具-线材关联");
    let main_mold_stock = sheet_rows("主模具库存汇总");

    let mut tooling_history: Vec<Value> = Vec::with_capacity(screws.len());
    for screw in screws {
        let screw_id = screw.get("id").map(String::as_str).unwrap_or("");
        let linked_punches: Vec<Value> = punch_links
            .iter()
            .filter(|link| link.get("screwSpecId").map(String::as_str) == Some(screw_id))
            .filter_map(|link| {
                let punch_id = link.get("punchId")?.as_str();
                let tool = punches
                    .iter()
                    .find(|row| row.get("id").map(String::as_str) == Some(punch_id))?;
                let stock = punch_stock
                    .iter()
                    .find(|row| row.get("punchId").map(String::as_str) == Some(punch_id));
                Some(json!({
                    "tool": tool,
                    "stock": stock,
                    "evidence": "verified_link"
                }))
            })
            .collect();
        let linked_dies: Vec<Value> = die_links
            .iter()
            .filter(|link| link.get("screwSpecId").map(String::as_str) == Some(screw_id))
            .filter_map(|link| {
                let die_id = link.get("dieId")?.as_str();
                let tool = dies
                    .iter()
                    .find(|row| row.get("id").map(String::as_str) == Some(die_id))?;
                let stock = die_stock
                    .iter()
                    .find(|row| row.get("dieId").map(String::as_str) == Some(die_id));
                Some(json!({
                    "tool": tool,
                    "stock": stock,
                    "evidence": "verified_link"
                }))
            })
            .collect();
        tooling_history.push(json!({
            "screw": screw,
            "verifiedPunches": linked_punches,
            "verifiedDies": linked_dies
        }));
    }

    let main_mold_candidates: Vec<Value> = main_molds
        .iter()
        .map(|tool| {
            let tool_id = tool.get("id").map(String::as_str).unwrap_or("");
            let wires: Vec<&str> = main_mold_links
                .iter()
                .filter(|link| link.get("mainMoldId").map(String::as_str) == Some(tool_id))
                .filter_map(|link| link.get("wireMaterial").map(String::as_str))
                .collect();
            let stock = main_mold_stock
                .iter()
                .find(|row| row.get("mainMoldId").map(String::as_str) == Some(tool_id));
            json!({ "tool": tool, "wireMaterials": wires, "stock": stock })
        })
        .collect();
    let tooling_recommendation = json!({
        "history": tooling_history,
        "allPunches": punches,
        "allDies": dies,
        "mainMolds": main_mold_candidates,
        "evidenceRules": {
            "verified_link": "系统中已有螺丝与该工具的关联记录，可作为强证据",
            "similar_spec": "仅由相似螺丝规格/头型/牙型推测，必须人工确认",
            "wire_compatible": "仅线材或线径相容，不能代表已验证可生产"
        }
    });

    // 附件感知：列出有附件的螺丝规格（id → 附件数），未列出 = 无附件。
    let attachment_counts = attachments::counts(file_path).unwrap_or_default();
    let mut attachments_map = serde_json::Map::new();
    for (id, count) in &attachment_counts {
        if *count > 0 {
            attachments_map.insert(id.clone(), json!(count));
        }
    }
    // 跨会话记忆：最近 8 条问答摘要。
    let recent_memory: Vec<&String> = memory.iter().rev().take(8).collect();
    serde_json::to_string(&json!({
        "schema": schema,
        "data": data,
        "totals": totals,
        "attachments": attachments_map,
        "memory": recent_memory,
        "toolingRecommendation": tooling_recommendation,
        "formatRules": SPEC_FORMAT_RULES
    }))
    .map_err(|e| format!("构建 AI 数据上下文失败: {}", e))
}

/// 规格/尺寸类字段的输入格式规范（与前端 SpecInput/parseSpecText 一致）。
/// AI 在修正已有数据的格式错误时，以此规则判断并生成 update 变更。
const SPEC_FORMAT_RULES: &str = r#"规格/尺寸类字段（列名含“规格”或“尺寸”的字段，如螺丝规格、冲头规格）遵循以下输入格式规范：
1. 数值表达式（尺寸组合/公差等，如 4.2 × 13）的乘号统一为 ×：4.2*13、4.2x13、4.2×13、4.2 × 13 都应规范为「4.2 × 13」。注意：本条只适用于纯数值表达式，不适用于编码名（见第 5 条）。
2. 范围用 ~ 分隔，两侧带空格。例如 7.7~8.1 规范为「7.7 ~ 8.1」。注意“1.9-2.0”这类后数大于前数的写法也属于范围，应规范为「1.9 ~ 2.0」。
3. 公差表示（注意：8.0-0.5、3.0±0.15 这类属于公差，不是纯数字）：
   - 对称公差用 ±：8±0.5 → 「8 ± 0.5」，3.0±0.15 → 「3 ± 0.15」
   - 仅上偏差：8+0.05 → 「8 +0.05」
   - 仅下偏差：8-0.02 → 「8 -0.02」，8.0-0.5 → 「8 -0.5」，46.0-0.1 → 「46 -0.1」
   - 非对称上下偏差：8+0.05/-0.02 → 「8 +0.05 / -0.02」
   - 无偏差：8 → 「8」
4. 公差的名义值、范围的端点、纯数字，所有数值都最多保留 3 位小数并去掉尾零：5.20 → 「5.2」，8.0 → 「8」。
5. 编码名（如牙板 M1.7-48*6、M2.6-28*11.8束、M4.2-18X16）：整体不改写，乘号符号也保持原样、不要修改（牙板等编码名业务上统一使用 *，个别历史数据可能是 X，但不要提议把它们改成 ×）。只有用户明确要求统一时，才按用户指定的符号处理。
6. 可带前缀（Φ、W=、D=、介、垫）和后缀（牙10、束、割尾），规范显示时前缀/后缀保留，如「Φ 4.2 × 13 牙10」。
当用户要求“修正格式错误/规范化数据”时，逐条检查这些字段是否符合上述规范，对不符合的记录生成 update 变更，把字段值修正为规范显示值。"#;

/// 当前启用的配置档案；无 active 匹配时回退到第一份。
fn active_agent_profile(config: &AgentConfig) -> Option<&AgentProfile> {
    config
        .profiles
        .iter()
        .find(|profile| profile.id == config.active)
        .or_else(|| config.profiles.first())
}

/// 配置档案的 API 协议：Zen 按模型注册表自动选择；自定义 cc 使用 Anthropic；其余内置按预设。
fn profile_protocol(profile: &AgentProfile) -> &'static str {
    if profile.kind == "builtin"
        && matches!(
            profile.provider.as_str(),
            AGENT_PROVIDER_OPENCODE_ZEN | AGENT_PROVIDER_OPENCODE_ZEN_FREE
        )
    {
        zen_model(&profile.model)
            .map(|preset| preset.protocol)
            .unwrap_or(AGENT_PROTOCOL_OPENAI)
    } else if profile.kind == "custom" && profile.format == AGENT_FORMAT_CC {
        AGENT_PROTOCOL_ANTHROPIC
    } else if profile.kind == "builtin" {
        agent_preset(&profile.provider)
            .map(|preset| preset.protocol)
            .unwrap_or(AGENT_PROTOCOL_OPENAI)
    } else {
        AGENT_PROTOCOL_OPENAI
    }
}

/// 配置档案的请求地址：Zen 按模型协议自动路由；自定义用用户填写的 endpoint；其他内置用预设。
fn profile_endpoint(profile: &AgentProfile) -> Result<String, String> {
    if profile.kind == "builtin"
        && matches!(
            profile.provider.as_str(),
            AGENT_PROVIDER_OPENCODE_ZEN | AGENT_PROVIDER_OPENCODE_ZEN_FREE
        )
    {
        return Ok(zen_endpoint(profile_protocol(profile)));
    }
    if profile.kind == "custom" {
        let endpoint = profile.endpoint.trim().trim_end_matches('/');
        if !(endpoint.starts_with("https://")
            || endpoint.starts_with("http://localhost")
            || endpoint.starts_with("http://127.0.0.1"))
        {
            return Err("API 地址必须使用 HTTPS；仅本机服务允许 HTTP".to_string());
        }
        Ok(endpoint.to_string())
    } else {
        Ok(agent_preset(&profile.provider)
            .map(|preset| preset.endpoint.to_string())
            .unwrap_or_default())
    }
}

/// 每份配置的 API Key 在系统凭据库中的独立账户名。
fn profile_key_account(id: &str) -> String {
    format!("agent-profile-{}", id)
}

/// 内置服务商列表（前端「添加配置-内置」下拉使用）。
/// 注意：opencode Zen（OpenAI 兼容）与 Claude Code / Anthropic Messages（cc）是两个独立服务，不要绑定。
fn builtin_agent_providers() -> Value {
    json!([
        {"value":"deepseek","label":"DeepSeek","model":"deepseek-v4-flash","protocol":"openai","needsApiKey":true},
        {"value":"openai","label":"OpenAI","model":"gpt-4.1-mini","protocol":"openai","needsApiKey":true},
        {"value":"glm","label":"智谱 GLM","model":"glm-4-flash-250414","protocol":"openai","needsApiKey":true},
        {"value":"anthropic","label":"Claude Code","model":"claude-sonnet-4-6","protocol":"anthropic","needsApiKey":true},
        {"value":"opencode-zen-free","label":"OpenCode Zen（免费）","model":"deepseek-v4-flash-free","protocol":"openai","needsApiKey":false},
        {"value":"opencode-zen","label":"OpenCode Zen（需 Key）","model":"deepseek-v4-flash","protocol":"openai","needsApiKey":true},
        {"value":"qwen","label":"通义千问","model":"qwen-plus","protocol":"openai","needsApiKey":true},
        {"value":"gemini","label":"Gemini","model":"gemini-3.6-flash","protocol":"gemini","needsApiKey":true}
    ])
}

#[tauri::command]
fn get_agent_config(state: State<AppState>) -> Result<Value, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let profiles: Vec<Value> = config
        .agent
        .profiles
        .iter()
        .map(|profile| {
            let api_key_required = if profile.kind == "builtin" {
                agent_preset(&profile.provider)
                    .map(|preset| preset.needs_api_key)
                    .unwrap_or(true)
            } else {
                true
            };
            json!({
                "id": profile.id,
                "name": profile.name,
                "kind": profile.kind,
                "provider": profile.provider,
                "format": profile.format,
                "endpoint": profile.endpoint,
                "model": profile.model,
                "apiKeyConfigured": secret_store::exists(&profile_key_account(&profile.id)),
                "apiKeyRequired": api_key_required,
            })
        })
        .collect();
    Ok(json!({
        "profiles": profiles,
        "active": config.agent.active,
        "credentialStore": secret_store::platform_store_name(),
        "builtins": builtin_agent_providers(),
        "ccEndpoint": OPENCODE_ZEN_ENDPOINT,
        "ccModels": zen_models_json(false),
        "zenFreeModels": zen_models_json(true),
        "zenModels": zen_models_json(false),
    }))
}

#[tauri::command]
fn set_agent_config(
    state: State<AppState>,
    profiles: Vec<Value>,
    active: String,
) -> Result<Value, String> {
    let mut parsed: Vec<AgentProfile> = Vec::new();
    let mut keys: Vec<(String, String)> = Vec::new();
    for raw in &profiles {
        let id = raw
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| "AI 配置缺少 id".to_string())?;
        let name = raw
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let kind = raw
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        if !matches!(kind.as_str(), "builtin" | "custom") {
            return Err("配置类型只能是内置(builtin)或自定义(custom)".to_string());
        }
        let provider = raw
            .get("provider")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let format = raw
            .get("format")
            .and_then(Value::as_str)
            .unwrap_or(AGENT_PROTOCOL_OPENAI)
            .trim()
            .to_string();
        let mut endpoint = raw
            .get("endpoint")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let mut model = raw
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        if kind == "builtin" {
            let preset =
                agent_preset(&provider).ok_or_else(|| format!("未知内置服务商「{}」", provider))?;
            endpoint = preset.endpoint.to_string();
            if model.is_empty() {
                model = preset.model.to_string();
            }
            if model.chars().count() > 120 {
                return Err("模型名称不能超过 120 个字符".to_string());
            }
            if matches!(
                provider.as_str(),
                AGENT_PROVIDER_OPENCODE_ZEN | AGENT_PROVIDER_OPENCODE_ZEN_FREE
            ) {
                let zen = zen_model(&model).ok_or_else(|| {
                    format!("未知 OpenCode Zen 模型「{}」，请从预设列表选择", model)
                })?;
                if provider == AGENT_PROVIDER_OPENCODE_ZEN_FREE && !zen.free {
                    return Err(format!("模型「{}」不在 OpenCode Zen 免费预设中", model));
                }
                endpoint = zen_endpoint(zen.protocol);
            }
        } else {
            if !matches!(format.as_str(), AGENT_PROTOCOL_OPENAI | AGENT_FORMAT_CC) {
                return Err("自定义配置格式只能是 openai 或 cc".to_string());
            }
            if !(endpoint.starts_with("https://")
                || endpoint.starts_with("http://localhost")
                || endpoint.starts_with("http://127.0.0.1"))
            {
                return Err("API 地址必须使用 HTTPS；仅本机服务允许 HTTP".to_string());
            }
            endpoint = endpoint.trim_end_matches('/').to_string();
            if model.is_empty() || model.chars().count() > 120 {
                return Err("模型名称不能为空且不能超过 120 个字符".to_string());
            }
        }
        let api_key = raw
            .get("apiKey")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        parsed.push(AgentProfile {
            id: id.clone(),
            name,
            kind,
            provider,
            format,
            endpoint,
            model,
        });
        keys.push((id, api_key));
    }
    if parsed.is_empty() {
        return Err("至少需要一个 AI 配置".to_string());
    }
    let active = if parsed.iter().any(|profile| profile.id == active) {
        active
    } else {
        parsed[0].id.clone()
    };
    for (id, api_key) in &keys {
        if api_key.is_empty() {
            continue;
        }
        secret_store::set(&profile_key_account(id), api_key)?;
    }
    let mut next = state.config.lock().map_err(|e| e.to_string())?.clone();
    next.agent.profiles = parsed;
    next.agent.active = active;
    persist_config(&state, next)?;
    Ok(json!({ "success": true }))
}

/// AI 调试日志：把每次请求的原始响应写入临时目录 mold-agent-debug.log，
/// 便于排查 AI 返回格式/解析问题。仅保留最近 512KB。
fn append_agent_debug_log(question: &str, reply: &agent::AgentReply) {
    let entry = format!(
        "=== {} | Q: {}\nRAW: {}\nANSWER: {}\nCHANGES: {}\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        question.chars().take(150).collect::<String>(),
        reply.raw,
        reply.answer.chars().take(150).collect::<String>(),
        reply.changes.len(),
    );
    let path = std::env::temp_dir().join("mold-agent-debug.log");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut content = existing;
    content.push_str(&entry);
    if content.len() > 512 * 1024 {
        content = content.split_off(content.len() - 256 * 1024);
        content.insert_str(0, "=== [log truncated] ===\n");
    }
    let _ = std::fs::write(&path, content);
}

#[tauri::command]
async fn agent_chat(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    question: String,
    history: Vec<Value>,
    page_context: Option<String>,
) -> Result<Value, String> {
    let file_path = state.file_path.lock().map_err(|e| e.to_string())?.clone();
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let profile = active_agent_profile(&config.agent).ok_or_else(|| {
        "尚未配置 AI 服务，请先在「AI 助手」的配置中添加并启用一个配置".to_string()
    })?;
    let provider = profile.provider.clone();
    let protocol = profile_protocol(profile).to_string();
    let endpoint = profile_endpoint(profile)?;
    let model = if profile.model.trim().is_empty() {
        agent_preset(&provider)
            .map(|preset| preset.model.to_string())
            .unwrap_or_default()
    } else {
        profile.model.clone()
    };
    let key_account = profile_key_account(&profile.id);
    let needs_api_key = agent_preset(&provider)
        .map(|preset| preset.needs_api_key)
        .unwrap_or(true);
    let api_key = if needs_api_key {
        secret_store::get(&key_account)
            .map_err(|error| format!("读取该 AI 配置的 API Key 失败：{}", error))?
    } else {
        String::new() // 免费服务（opencode Zen 等）无需 API Key
    };
    let memory = config.agent_memory.clone();
    let context = build_agent_context(&file_path, &memory)?;
    let page_context = page_context.unwrap_or_default();
    let question_for_memory = question.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // 流式增量通过 Tauri 事件推送到前端（"agent-stream"，增量以 "r:" 前缀标识推理过程）
        let stream_app = app.clone();
        let on_stream = |chunk: &str| {
            let _ = stream_app.emit("agent-stream", chunk);
        };
        let reply = agent::chat(
            &provider,
            &protocol,
            &endpoint,
            &model,
            &api_key,
            &question,
            &context,
            &page_context,
            &history,
            None,
            Some(&on_stream),
        )?;
        append_agent_debug_log(&question, &reply);
        let changes: Vec<Value> = reply
            .changes
            .into_iter()
            .filter_map(|change| match normalize_agent_change(&file_path, &change) {
                Ok(change) => Some(change),
                Err(error) => {
                    eprintln!("AI 返回了不可执行变更：{}；原始响应：{}", error, reply.raw);
                    None
                }
            })
            .collect();
        Ok(json!({ "answer": reply.answer, "changes": changes, "reasoning": reply.reasoning }))
    })
    .await
    .map_err(|e| format!("执行 AI 查询失败: {}", e))?
    .and_then(|result| {
        // 跨会话记忆：成功后保存本轮问答摘要（question 与 answer 短摘要）
        let answer = result
            .get("answer")
            .and_then(Value::as_str)
            .unwrap_or_default();
        append_agent_memory(&state, &question_for_memory, answer)?;
        Ok(result)
    })
}

/// 保存一轮问答摘要到跨会话记忆（保留最近 30 条）。
fn append_agent_memory(
    state: &State<AppState>,
    question: &str,
    answer: &str,
) -> Result<(), String> {
    let now = chrono::Local::now().format("%m-%d %H:%M").to_string();
    let q = question.trim().chars().take(60).collect::<String>();
    let a = answer.trim().chars().take(80).collect::<String>();
    let mut next = state.config.lock().map_err(|e| e.to_string())?.clone();
    next.agent_memory
        .push(format!("[{}] 问:{} | 答:{}", now, q, a));
    if next.agent_memory.len() > 30 {
        next.agent_memory = next.agent_memory.split_off(next.agent_memory.len() - 30);
    }
    persist_config(state, next)?;
    Ok(())
}

/// 对比 Excel 文件与当前数据库，返回各业务表的差异（新增/修改/缺失/无变化）。
#[tauri::command]
fn compare_excel(state: State<AppState>, xlsx_path: String) -> Result<Value, String> {
    let file_path = state.file_path.lock().map_err(|e| e.to_string())?.clone();
    excel::compare_xlsx_with_db(&xlsx_path, &file_path)
}

/// 构建 Excel 文件数据上下文：只做 Excel -> JSON 格式转换，不预处理差异。
/// 所有识别到的业务表、所有原始行完整交给 AI。
fn build_excel_context(xlsx_path: &str) -> Result<String, String> {
    let sheets = excel::list_excel_sheets(xlsx_path)?;
    if sheets.is_empty() {
        return Err("Excel 中未识别到系统支持的业务表".to_string());
    }
    let mut data = serde_json::Map::new();
    let mut totals = serde_json::Map::new();
    let mut sources = serde_json::Map::new();
    for info in &sheets {
        if info.system_calculated {
            continue;
        }
        let table = info.table.clone();
        let rows = excel::read_xlsx_all(xlsx_path, &info.name)?;
        totals.insert(table.clone(), json!(rows.len()));
        sources.insert(table.clone(), json!(info.name));
        data.insert(table, serde_json::to_value(rows).unwrap_or(json!([])));
    }
    let file_name = Path::new(xlsx_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| xlsx_path.to_string());
    serde_json::to_string(&json!({
        "excelFile": file_name,
        "excelData": data,
        "excelTotals": totals,
        "excelSourceSheets": sources
    }))
    .map_err(|e| format!("Excel 转 JSON 失败: {}", e))
}

/// 直接把 Excel 文件原始数据交给 AI 分析（不做差异预处理），
/// 系统数据上下文与 Excel 数据上下文一并注入，AI 自行对比并给出建议（可通过 changes 执行）。
#[tauri::command]
async fn agent_analyze_excel(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    xlsx_path: String,
    history: Vec<Value>,
    page_context: Option<String>,
) -> Result<Value, String> {
    let file_path = state.file_path.lock().map_err(|e| e.to_string())?.clone();
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let profile = active_agent_profile(&config.agent).ok_or_else(|| {
        "尚未配置 AI 服务，请先在「AI 助手」的配置中添加并启用一个配置".to_string()
    })?;
    let provider = profile.provider.clone();
    let protocol = profile_protocol(profile).to_string();
    let endpoint = profile_endpoint(profile)?;
    let model = if profile.model.trim().is_empty() {
        agent_preset(&provider)
            .map(|preset| preset.model.to_string())
            .unwrap_or_default()
    } else {
        profile.model.clone()
    };
    let key_account = profile_key_account(&profile.id);
    let needs_api_key = agent_preset(&provider)
        .map(|preset| preset.needs_api_key)
        .unwrap_or(true);
    let api_key = if needs_api_key {
        secret_store::get(&key_account)
            .map_err(|error| format!("读取该 AI 配置的 API Key 失败：{}", error))?
    } else {
        String::new() // 免费服务（opencode Zen 等）无需 API Key
    };
    let memory = config.agent_memory.clone();
    let system_context_text = build_agent_context(&file_path, &memory)?;
    let excel_context_text = build_excel_context(&xlsx_path)?;
    // 合并为单个合法 JSON 对象。不能把两个 JSON 字符串直接换行拼接——模型可能只解析第一段而忽略 Excel。
    let mut system_value: Value = serde_json::from_str(&system_context_text)
        .map_err(|e| format!("解析系统数据上下文失败: {}", e))?;
    let excel_value: Value = serde_json::from_str(&excel_context_text)
        .map_err(|e| format!("解析 Excel 数据上下文失败: {}", e))?;
    let system_object = system_value
        .as_object_mut()
        .ok_or_else(|| "系统数据上下文格式异常".to_string())?;
    if let Some(excel_data) = excel_value.get("excelData") {
        system_object.insert("excelData".to_string(), excel_data.clone());
    }
    if let Some(excel_totals) = excel_value.get("excelTotals") {
        system_object.insert("excelTotals".to_string(), excel_totals.clone());
    }
    let full_context = serde_json::to_string(&system_value)
        .map_err(|e| format!("序列化完整分析上下文失败: {}", e))?;
    let excel_context_summary = excel_value.get("excelTotals").cloned().unwrap_or(json!({}));
    eprintln!(
        "Excel AI 分析上下文已构建：文件={}，总字符={}，Excel 表行数={}",
        xlsx_path,
        full_context.chars().count(),
        excel_context_summary
    );
    let page_context = page_context.unwrap_or_default();
    let file_name = Path::new(&xlsx_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| xlsx_path.clone());
    let question_for_memory = format!("分析 Excel 文件「{}」与系统数据差异", file_name);
    tauri::async_runtime::spawn_blocking(move || {
        let stream_app = app.clone();
        let on_stream = |chunk: &str| {
            let _ = stream_app.emit("agent-stream", chunk);
        };
        let question = format!(
            "请分析用户上传的 Excel 文件「{}」：对比其中的数据与当前系统数据，找出差异（新增/更新/缺失）、可疑重复，并给出处理建议。",
            file_name
        );
        let reply = agent::chat(
            &provider,
            &protocol,
            &endpoint,
            &model,
            &api_key,
            &question,
            &full_context,
            &page_context,
            &history,
            Some(agent::EXCEL_ANALYSIS_SYSTEM_PROMPT),
            Some(&on_stream),
        )?;
        append_agent_debug_log(&question, &reply);
        let changes: Vec<Value> = reply
            .changes
            .into_iter()
            .filter_map(|change| match normalize_agent_change(&file_path, &change) {
                Ok(change) => Some(change),
                Err(error) => {
                    eprintln!("AI 返回了不可执行变更：{}；原始响应：{}", error, reply.raw);
                    None
                }
            })
            .collect();
        Ok(json!({ "answer": reply.answer, "changes": changes, "reasoning": reply.reasoning }))
    })
    .await
    .map_err(|e| format!("执行 Excel 分析失败: {}", e))?
    .and_then(|result| {
        let answer = result
            .get("answer")
            .and_then(Value::as_str)
            .unwrap_or_default();
        append_agent_memory(&state, &question_for_memory, answer)?;
        Ok(result)
    })
}

/// 应用 AI 提出的系统设置变更（set_setting）。
/// 支持白名单配置项：backup_count、allow_delete、backup_path；theme 由界面端执行，不在此处处理。
fn apply_agent_setting(
    state: &State<AppState>,
    fields: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let mut next = state.config.lock().map_err(|e| e.to_string())?.clone();
    let mut applied: Vec<String> = Vec::new();
    for (key, value) in fields {
        match key.as_str() {
            "backup_count" => {
                if let Some(count) = value.as_u64() {
                    next.backup_count = count.max(1).min(100) as usize;
                    applied.push(format!("备份保留数量={}", next.backup_count));
                }
            }
            "allow_delete" => {
                if let Some(flag) = value.as_bool() {
                    next.allow_delete = flag;
                    applied.push(format!(
                        "允许删除数据={}",
                        if flag { "开启" } else { "关闭" }
                    ));
                }
            }
            "backup_path" => {
                if let Value::String(path) = value {
                    let path = path.trim().to_string();
                    if !path.is_empty() {
                        next.backup_path = Some(path.clone());
                        applied.push(format!("备份目录={}", path));
                    }
                }
            }
            _ => {}
        }
    }
    if applied.is_empty() {
        return Err("没有可应用的系统设置项（支持的配置：backup_count、allow_delete、backup_path；主题由界面直接执行）".to_string());
    }
    persist_config(&state, next)?;
    Ok(
        json!({ "success": true, "operation": "set_setting", "table": "system_settings", "result": applied }),
    )
}

/// 执行单条已归一化的 AI 变更（不负责备份与清理，由调用方统一处理）。
fn apply_one_normalized_change(
    state: &State<AppState>,
    file_path: &str,
    normalized: &Value,
) -> Result<Value, String> {
    let operation = normalized
        .get("operation")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let table = normalized
        .get("table")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let id = normalized
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let fields = normalized
        .get("fields")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    if operation == "delete" && !config.allow_delete {
        return Err("系统当前未启用删除权限，请先在配置管理中开启".to_string());
    }
    if operation == "set_setting" {
        return apply_agent_setting(state, &fields);
    }
    if operation == "import" {
        return Err("AI 自动导入暂不支持，请确认后在界面手动导入该文件".to_string());
    }
    let map: std::collections::HashMap<String, String> = fields
        .into_iter()
        .map(|(key, value)| {
            let text = match value {
                Value::String(value) => value,
                Value::Number(value) => value.to_string(),
                Value::Bool(value) => value.to_string(),
                Value::Null => String::new(),
                other => other.to_string(),
            };
            (key, text)
        })
        .collect();
    let result = match operation {
        "add" => serde_json::to_value(excel::add_row(file_path, table, &map)?).unwrap_or(json!({})),
        "update" => serde_json::to_value(excel::update_row(file_path, table, id, &map)?)
            .unwrap_or(json!({})),
        "delete" => {
            let deleted = excel::delete_row(file_path, table, id)?;
            if deleted && table == "螺丝规格表" {
                attachments::delete_for_screw(file_path, id)?;
            }
            json!({ "deleted": deleted, "id": id })
        }
        _ => return Err("未知 AI 变更操作".to_string()),
    };
    Ok(json!({ "success": true, "operation": operation, "table": table, "result": result }))
}

#[tauri::command]
fn apply_agent_change(state: State<AppState>, change: Value) -> Result<Value, String> {
    let file_path = state.file_path.lock().map_err(|e| e.to_string())?.clone();
    let normalized = normalize_agent_change(&file_path, &change)?;
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let backup_dir = get_backup_dir_for_file(&file_path, &config);
    do_backup(&file_path, &backup_dir, "AI 变更前备份")?;
    let result = apply_one_normalized_change(&state, &file_path, &normalized)?;
    cleanup_old_backups(&backup_dir, config.backup_count)?;
    Ok(result)
}

/// 批量应用 AI 提出的多条变更：一次「变更前备份」+ 逐条执行 + 一次清理。
/// 避免逐条调用 apply_agent_change 时每条都备份（大数量级变更会产生大量备份并刷掉旧备份）。
#[tauri::command]
fn apply_agent_changes(state: State<AppState>, changes: Vec<Value>) -> Result<Value, String> {
    let file_path = state.file_path.lock().map_err(|e| e.to_string())?.clone();
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    if changes.is_empty() {
        return Ok(json!({ "success": true, "applied": [], "failed": [] }));
    }
    let backup_dir = get_backup_dir_for_file(&file_path, &config);
    do_backup(&file_path, &backup_dir, "AI 批量变更前备份")?;
    let mut applied: Vec<Value> = Vec::new();
    let mut failed: Vec<Value> = Vec::new();
    for (index, change) in changes.iter().enumerate() {
        let outcome = normalize_agent_change(&file_path, change)
            .and_then(|normalized| apply_one_normalized_change(&state, &file_path, &normalized));
        match outcome {
            Ok(result) => {
                // changeIndex 让前端能精确把成功结果对应回原始变更，避免失败项导致摘要错位。
                let mut entry = result;
                if let Some(map) = entry.as_object_mut() {
                    map.insert("changeIndex".to_string(), json!(index));
                }
                applied.push(entry);
            }
            Err(error) => failed.push(json!({ "change": change, "error": error })),
        }
    }
    cleanup_old_backups(&backup_dir, config.backup_count)?;
    Ok(json!({ "success": true, "applied": applied, "failed": failed }))
}

#[tauri::command]
fn restore_backup(state: State<AppState>, backup_path: String) -> Result<Value, String> {
    let backup_file = Path::new(&backup_path);
    if !backup_file.is_file() {
        return Err("备份文件不存在".to_string());
    }
    backup::validate_backup_zip(backup_file)?;

    let file_path = state.file_path.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir_for_file(&file_path, &config);
    do_backup(&file_path, &backup_dir, "恢复前备份")?;

    let attachments_restored = backup::restore_backup_zip(backup_file, &file_path)?;
    Ok(json!({
        "success": true,
        "attachmentsRestored": attachments_restored,
        "message": if attachments_restored {
            "数据与附件已从 ZIP 备份同步恢复"
        } else {
            "备份没有附件快照，已保留当前附件"
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_millisecond_zip_backup_names_are_managed() {
        let root =
            std::env::temp_dir().join(format!("mold-backup-name-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let current = root.join("mold-backup-20260803_204014_123.zip");
        let legacy_seconds = root.join("mold-backup-20260803_204014.zip");
        let legacy_old_format = root.join("mold-data-backup-20260803_204014_123.xlsx");
        fs::write(&current, b"current").unwrap();
        fs::write(&legacy_seconds, b"legacy").unwrap();
        fs::write(&legacy_old_format, b"legacy").unwrap();

        assert!(is_managed_backup_path(&current));
        assert!(!is_managed_backup_path(&legacy_seconds));
        assert!(!is_managed_backup_path(&legacy_old_format));
        fs::remove_dir_all(root).unwrap();
    }
}

fn default_backup_dir() -> String {
    // 优先使用 exe 同级的 data/backups 目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let backup_dir = exe_dir.join("data").join("backups");
            return backup_dir.to_string_lossy().to_string();
        }
    }
    // 回退到当前工作目录
    std::env::current_dir()
        .map(|d| d.join("data").join("backups").to_string_lossy().to_string())
        .unwrap_or_else(|_| "./data/backups".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config_path = get_config_path();
    migrate_legacy_config(&config_path);
    let mut config = load_config(&config_path);

    let initial_path = config
        .file_path
        .clone()
        .filter(|p| !p.is_empty() && Path::new(p).exists())
        .unwrap_or_else(|| excel::get_default_file_path());

    // SQLite 存储初始化：旧 .xlsx 数据文件一次性迁移到数据库，之后一律使用 .db。
    let is_legacy_xlsx = Path::new(&initial_path)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("xlsx"));
    let data_path = if is_legacy_xlsx {
        let legacy_xlsx = initial_path.clone();
        let db_path = Path::new(&legacy_xlsx)
            .with_extension("db")
            .to_string_lossy()
            .to_string();
        match db::connect(&db_path) {
            Ok(conn) => {
                if let Err(error) = db::init_schema(&conn) {
                    eprintln!("初始化数据库结构失败: {}", error);
                }
                if let Err(error) = db::migrate_from_xlsx(&conn, &legacy_xlsx) {
                    eprintln!("迁移旧 Excel 数据失败: {}", error);
                }
                drop(conn);
            }
            Err(error) => eprintln!("打开数据库失败: {}", error),
        }
        config.file_path = Some(db_path.clone());
        if let Err(error) = save_config(&config_path, &config) {
            eprintln!("保存数据文件配置失败: {}", error);
        }
        db_path
    } else {
        match db::connect(&initial_path) {
            Ok(conn) => {
                if let Err(error) = db::init_schema(&conn) {
                    eprintln!("初始化数据库结构失败: {}", error);
                }
                let legacy_xlsx = Path::new(&initial_path)
                    .with_extension("xlsx")
                    .to_string_lossy()
                    .to_string();
                if Path::new(&legacy_xlsx).is_file() {
                    if let Err(error) = db::migrate_from_xlsx(&conn, &legacy_xlsx) {
                        eprintln!("迁移旧 Excel 数据失败: {}", error);
                    }
                }
                drop(conn);
            }
            Err(error) => eprintln!("打开数据库失败: {}", error),
        }
        initial_path
    };

    // 规范化：消除默认回退路径（debug 下 current_dir/../data）中的 `..` 段，保证显示与存储均为干净绝对路径。
    let data_path = normalized_string(Path::new(&data_path));

    // 启动时备份
    let backup_dir = get_backup_dir_for_file(&data_path, &config);
    if let Err(error) = do_backup(&data_path, &backup_dir, "应用启动") {
        eprintln!("启动备份失败: {}", error);
    }
    if let Err(error) = cleanup_old_backups(&backup_dir, config.backup_count) {
        eprintln!("清理启动备份失败: {}", error);
    }

    let auto_upload_on_start = config.webdav.auto_upload_on_start;
    let app_state = AppState {
        file_path: Mutex::new(data_path),
        config: Mutex::new(config),
        config_path: Mutex::new(config_path),
        webdav_operation: Mutex::new(()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_all_records,
            get_record,
            add_record,
            update_record,
            delete_record,
            get_operation_logs,
            clear_operation_logs,
            export_data,
            import_data,
            export_excel_group,
            list_excel_sheets,
            import_excel_sheets,
            export_data_package,
            import_data_package,
            get_file_path_cmd,
            set_file_path,
            calculate_stock,
            backup_data,
            get_backup_config,
            set_backup_config,
            list_backups,
            toggle_backup_lock,
            restore_backup,
            get_allow_delete,
            set_allow_delete,
            get_die_machine_types,
            set_die_machine_types,
            get_punch_specs,
            set_punch_specs,
            compare_excel,
            agent_analyze_excel,
            get_agent_config,
            set_agent_config,
            agent_chat,
            apply_agent_change,
            apply_agent_changes,
            list_screw_attachments,
            get_screw_attachment_counts,
            import_screw_attachment,
            read_screw_attachment,
            update_screw_attachment,
            delete_screw_attachment,
            get_webdav_config,
            set_webdav_config,
            test_webdav_connection,
            get_webdav_status,
            upload_webdav_now,
            download_webdav_now,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |app_handle, event| {
            if let tauri::RunEvent::Ready = event {
                if auto_upload_on_start {
                    let app = app_handle.clone();
                    tauri::async_runtime::spawn_blocking(move || {
                        if let Some(state) = app.try_state::<AppState>() {
                            if let Err(error) = upload_webdav_snapshot(&state, false) {
                                eprintln!("启动 WebDAV 同步失败: {}", error);
                            }
                        }
                    });
                }
            }
            if let tauri::RunEvent::Exit = event {
                if let Some(state) = app_handle.try_state::<AppState>() {
                    let fp = state.file_path.lock().unwrap().clone();
                    let config = state.config.lock().unwrap().clone();
                    let backup_dir = get_backup_dir_for_file(&fp, &config);
                    if let Err(error) = do_backup(&fp, &backup_dir, "应用退出") {
                        eprintln!("退出备份失败: {}", error);
                    }
                    if let Err(error) = cleanup_old_backups(&backup_dir, config.backup_count) {
                        eprintln!("清理退出备份失败: {}", error);
                    }
                    if config.webdav.auto_upload_on_exit {
                        if let Err(error) = upload_webdav_snapshot(&state, false) {
                            eprintln!("退出 WebDAV 同步失败: {}", error);
                        }
                    }
                }
            }
        });
}
