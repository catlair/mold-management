mod excel;
mod attachments;

use std::sync::Mutex;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{State, Manager};
use chrono::Local;

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

fn load_backup_index(backup_dir: &str) -> Vec<BackupRecord> {
    let index_path = get_backup_index_path(backup_dir);
    if index_path.exists() {
        if let Ok(content) = fs::read_to_string(&index_path) {
            if let Ok(records) = serde_json::from_str::<Vec<BackupRecord>>(&content) {
                return records;
            }
        }
    }
    vec![]
}

fn save_backup_index(backup_dir: &str, records: &[BackupRecord]) {
    let index_path = get_backup_index_path(backup_dir);
    if let Ok(content) = serde_json::to_string_pretty(records) {
        let _ = fs::write(index_path, content);
    }
}

fn attachment_dir_for_data(file_path: &str) -> Result<PathBuf, String> {
    Path::new(file_path)
        .parent()
        .map(|parent| parent.join("attachments"))
        .ok_or_else(|| "无法确定附件数据目录".to_string())
}

fn attachment_snapshot_dir(backup_file: &Path) -> Result<PathBuf, String> {
    let parent = backup_file
        .parent()
        .ok_or_else(|| "无法确定备份目录".to_string())?;
    let stem = backup_file
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "备份文件名无效".to_string())?;
    Ok(parent.join(format!("{}-attachments", stem)))
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

fn copy_directory(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target).map_err(|e| format!("创建附件备份目录失败: {}", e))?;
    if !source.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(source).map_err(|e| format!("读取附件目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取附件目录项失败: {}", e))?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory(&source_path, &target_path)?;
        } else if source_path.is_file() {
            fs::copy(&source_path, &target_path).map_err(|e| {
                format!("复制附件失败 {}: {}", source_path.display(), e)
            })?;
        }
    }
    Ok(())
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

    // Excel 与附件目录共同组成一个数据集；任一内容变化都会创建新快照。
    let records = load_backup_index(backup_dir);
    for record in &records {
        if record.backup_md5 == current_md5 {
            return Ok(String::new());
        }
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("mold-data-backup-{}.xlsx", timestamp);
    let backup_file = PathBuf::from(backup_dir).join(&backup_name);
    let bytes_copied = fs::copy(file_path, &backup_file)
        .map_err(|e| format!("复制数据文件失败: {}", e))?;

    let source_attachments = attachment_dir_for_data(file_path)?;
    if source_attachments.exists() {
        let snapshot_dir = attachment_snapshot_dir(&backup_file)?;
        if snapshot_dir.exists() {
            fs::remove_dir_all(&snapshot_dir)
                .map_err(|e| format!("清理旧附件快照失败: {}", e))?;
        }
        copy_directory(&source_attachments, &snapshot_dir)?;
    }

    // 验证备份文件已创建且有内容
    if !backup_file.exists() {
        return Err("备份文件创建失败".to_string());
    }
    if bytes_copied == 0 {
        return Err("备份文件为空".to_string());
    }

    // 写入索引记录
    let mut new_records = records;
    new_records.push(BackupRecord {
        file_path: backup_file.to_string_lossy().to_string(),
        backup_time: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        backup_reason: reason.to_string(),
        backup_md5: current_md5,
        locked: false,
    });
    save_backup_index(backup_dir, &new_records);

    Ok(backup_file.to_string_lossy().to_string())
}

fn cleanup_old_backups(backup_dir: &str, keep_count: usize) {
    let mut records = load_backup_index(backup_dir);

    // 统计未锁定的记录
    let unlocked: Vec<usize> = records.iter()
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
            let _ = fs::remove_file(backup_file);
            if let Ok(snapshot_dir) = attachment_snapshot_dir(backup_file) {
                if snapshot_dir.exists() {
                    let _ = fs::remove_dir_all(snapshot_dir);
                }
            }
            records.remove(idx);
        }

        save_backup_index(backup_dir, &records);
    }
}

#[tauri::command]
fn list_backups(state: State<AppState>) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir_for_file(&path, &config);
    let records = load_backup_index(&backup_dir);
    Ok(serde_json::to_value(records).unwrap_or(json!([])))
}

#[tauri::command]
fn toggle_backup_lock(state: State<AppState>, index: usize) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir_for_file(&path, &config);
    let mut records = load_backup_index(&backup_dir);
    if index < records.len() {
        records[index].locked = !records[index].locked;
        save_backup_index(&backup_dir, &records);
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
    Path::new(file_path).parent()
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
}

fn default_backup_count() -> usize { 10 }

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
        })
    } else {
        Config {
            file_path: None,
            backup_count: 10,
            backup_path: None,
            allow_delete: false,
        }
    }
}

fn save_config(config_path: &Path, config: &Config) {
    if let Ok(content) = serde_json::to_string_pretty(config) {
        let _ = fs::write(config_path, content);
    }
}

#[tauri::command]
fn get_all_records(state: State<AppState>, sheet_name: String) -> Result<Vec<Value>, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let items = excel::get_all(&path, &sheet_name)?;
    Ok(items.into_iter().map(|m| serde_json::to_value(m).unwrap_or(json!({}))).collect())
}

#[tauri::command]
fn get_record(state: State<AppState>, sheet_name: String, id: String) -> Result<Option<Value>, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let item = excel::get_by_id(&path, &sheet_name, &id)?;
    Ok(item.map(|m| serde_json::to_value(m).unwrap_or(json!({}))))
}

#[tauri::command]
fn add_record(state: State<AppState>, sheet_name: String, item: Value) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    // 将所有值转为字符串，数组用逗号连接
    let map: std::collections::HashMap<String, String> = item.as_object()
        .map(|obj| obj.iter().map(|(k, v)| {
            let val = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                Value::Array(arr) => {
                    arr.iter().filter_map(|item| {
                        match item {
                            Value::String(s) => Some(s.clone()),
                            Value::Number(n) => Some(n.to_string()),
                            Value::Bool(b) => Some(b.to_string()),
                            _ => None,
                        }
                    }).collect::<Vec<_>>().join(",")
                }
                _ => v.to_string(),
            };
            (k.clone(), val)
        }).collect())
        .ok_or("无效的数据格式")?;
    let result = excel::add_row(&path, &sheet_name, &map)?;
    Ok(serde_json::to_value(result).unwrap_or(json!({})))
}

#[tauri::command]
fn update_record(state: State<AppState>, sheet_name: String, id: String, data: Value) -> Result<Value, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    // 将所有值转为字符串，数组用逗号连接
    let map: std::collections::HashMap<String, String> = data.as_object()
        .map(|obj| obj.iter().map(|(k, v)| {
            let val = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                Value::Array(arr) => {
                    arr.iter().filter_map(|item| {
                        match item {
                            Value::String(s) => Some(s.clone()),
                            Value::Number(n) => Some(n.to_string()),
                            Value::Bool(b) => Some(b.to_string()),
                            _ => None,
                        }
                    }).collect::<Vec<_>>().join(",")
                }
                _ => v.to_string(),
            };
            (k.clone(), val)
        }).collect())
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
fn import_screw_attachment(state: State<AppState>, screw_spec_id: String, source_path: String) -> Result<Value, String> {
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
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let bytes = excel::export_data(&path)?;
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(json!({ "filename": "mold-data.xlsx", "data": b64 }))
}

#[tauri::command]
fn import_data(state: State<AppState>, data: String) -> Result<Value, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD.decode(&data).map_err(|e| e.to_string())?;
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    let stats = excel::import_data(&path, &bytes)?;
    Ok(json!({ "success": true, "stats": stats }))
}

#[tauri::command]
fn get_file_path_cmd(state: State<AppState>) -> Result<String, String> {
    let path = state.file_path.lock().map_err(|e| e.to_string())?;
    Ok(path.clone())
}

#[tauri::command]
fn set_file_path(state: State<AppState>, path: String) -> Result<Value, String> {
    {
        let mut fp = state.file_path.lock().map_err(|e| e.to_string())?;
        *fp = path.clone();
    }
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.file_path = Some(path.clone());
    let config_path = state.config_path.lock().map_err(|e| e.to_string())?;
    save_config(&config_path, &config);
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
            results.insert(t.to_string(), serde_json::to_value(data).unwrap_or(json!([])));
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
    drop(config);
    drop(path);
    let backup_file = do_backup(&state.file_path.lock().map_err(|e| e.to_string())?, &backup_dir, "手动备份")?;
    cleanup_old_backups(&backup_dir, count);
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
    Ok(json!({
        "backupCount": config.backup_count,
        "backupPath": config.backup_path,
        "defaultBackupDir": default_backup_dir(),
        "effectiveBackupDir": backup_dir,
    }))
}

#[tauri::command]
fn set_backup_config(state: State<AppState>, backup_count: usize, backup_path: Option<String>) -> Result<Value, String> {
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        config.backup_count = backup_count;
        config.backup_path = backup_path;
        let config_path = state.config_path.lock().map_err(|e| e.to_string())?;
        save_config(&config_path, &config);
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
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        config.allow_delete = allow;
        let config_path = state.config_path.lock().map_err(|e| e.to_string())?;
        save_config(&config_path, &config);
    }
    Ok(json!({ "success": true }))
}

#[tauri::command]
fn restore_backup(state: State<AppState>, backup_path: String) -> Result<Value, String> {
    if !Path::new(&backup_path).exists() {
        return Err("备份文件不存在".to_string());
    }
    let file_path = state.file_path.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir_for_file(&file_path, &config);
    do_backup(&file_path, &backup_dir, "恢复前备份")?;
    fs::copy(&backup_path, &*file_path)
        .map_err(|e| format!("恢复数据文件失败: {}", e))?;

    let source_snapshot = attachment_snapshot_dir(Path::new(&backup_path))?;
    let mut attachments_restored = false;
    if source_snapshot.exists() {
        let target_attachments = attachment_dir_for_data(&file_path)?;
        if target_attachments.exists() {
            fs::remove_dir_all(&target_attachments)
                .map_err(|e| format!("清理当前附件目录失败: {}", e))?;
        }
        copy_directory(&source_snapshot, &target_attachments)?;
        attachments_restored = true;
    }
    Ok(json!({
        "success": true,
        "attachmentsRestored": attachments_restored,
        "message": if attachments_restored {
            "数据与附件已同步恢复"
        } else {
            "历史备份没有附件快照，已保留当前附件"
        }
    }))
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
    let config = load_config(&config_path);

    let initial_path = config.file_path.clone()
        .filter(|p| !p.is_empty() && Path::new(p).exists())
        .unwrap_or_else(|| excel::get_default_file_path());

    // 启动时备份
    let backup_dir = get_backup_dir_for_file(&initial_path, &config);
    let _ = do_backup(&initial_path, &backup_dir, "应用启动");
    cleanup_old_backups(&backup_dir, config.backup_count);

    let app_state = AppState {
        file_path: Mutex::new(initial_path),
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
                    let _ = do_backup(&fp, &backup_dir, "应用退出");
                    cleanup_old_backups(&backup_dir, config.backup_count);
                }
            }
        });
}
