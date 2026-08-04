mod attachments;
mod backup;
mod data_package;
mod db;
mod excel;
mod storage;

use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{Manager, State};

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
    let data_path = Path::new(file_path);
    let data = fs::read(data_path).map_err(|e| e.to_string())?;
    let mut hasher = Md5::new();
    hasher.update(&data);

    let attachment_dir = attachment_dir_for_data(file_path)?;
    let mut files = Vec::new();
    collect_files(&attachment_dir, &mut files)?;
    files.sort();
    for path in files {
        if let Ok(relative) = path.strip_prefix(&attachment_dir) {
            hasher.update(relative.to_string_lossy().as_bytes());
        }
        let bytes = fs::read(&path)
            .map_err(|e| format!("读取附件备份内容失败 {}: {}", path.display(), e))?;
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
            fs::remove_file(backup_file)
                .map_err(|e| format!("删除过期备份失败「{}」: {}", backup_file.display(), e))?;
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
            return custom.clone();
        }
    }
    // 没配置则用数据文件同级 backups 目录
    Path::new(file_path)
        .parent()
        .map(|p| p.join("backups").to_string_lossy().to_string())
        .unwrap_or_else(|| default_backup_dir())
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
}

fn get_config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("config.json")))
        .unwrap_or_else(|| PathBuf::from("config.json"))
}

fn load_config(config_path: &Path) -> Config {
    if let Ok(content) = fs::read_to_string(config_path) {
        serde_json::from_str(&content).unwrap_or(Config {
            file_path: None,
            backup_count: 10,
            backup_path: None,
            allow_delete: false,
            die_machine_types: default_die_machine_types(),
            punch_specs: default_punch_specs(),
        })
    } else {
        Config {
            file_path: None,
            backup_count: 10,
            backup_path: None,
            allow_delete: false,
            die_machine_types: default_die_machine_types(),
            punch_specs: default_punch_specs(),
        }
    }
}

fn save_config(config_path: &Path, config: &Config) -> Result<(), String> {
    storage::atomic_write_json(config_path, config)
        .map_err(|e| format!("保存应用配置失败「{}」: {}", config_path.display(), e))
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
    Ok(json!({ "sheets": sheets }))
}

#[tauri::command]
fn import_excel_sheets(
    state: State<AppState>,
    source_path: String,
    selected_sheets: Vec<String>,
) -> Result<Value, String> {
    if selected_sheets.is_empty() {
        return Err("请至少选择一个要导入的工作表".to_string());
    }
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir_for_file(&path, &config);
    let count = config.backup_count;
    do_backup(&path, &backup_dir, "Excel 导入前备份")?;
    let stats = excel::import_sheets_from_xlsx(&path, &source_path, &selected_sheets)?;
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
    do_backup(&path, &backup_dir, "Excel 导入前备份")?;
    let stats = excel::import_sheets_from_xlsx(&path, &temporary.to_string_lossy(), &sheets)?;
    let _ = fs::remove_file(&temporary);
    cleanup_old_backups(&backup_dir, count)?;
    Ok(json!({ "success": true, "stats": stats }))
}

#[tauri::command]
fn get_file_path_cmd(state: State<AppState>) -> Result<String, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    Ok(path.clone())
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

    // 启动时备份
    let backup_dir = get_backup_dir_for_file(&data_path, &config);
    if let Err(error) = do_backup(&data_path, &backup_dir, "应用启动") {
        eprintln!("启动备份失败: {}", error);
    }
    if let Err(error) = cleanup_old_backups(&backup_dir, config.backup_count) {
        eprintln!("清理启动备份失败: {}", error);
    }

    let app_state = AppState {
        file_path: Mutex::new(data_path),
        config: Mutex::new(config),
        config_path: Mutex::new(config_path),
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
            list_screw_attachments,
            get_screw_attachment_counts,
            import_screw_attachment,
            read_screw_attachment,
            update_screw_attachment,
            delete_screw_attachment,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |app_handle, event| {
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
                }
            }
        });
}
