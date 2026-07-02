mod excel;

use std::sync::Mutex;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{State, Manager};
use chrono::Local;

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    file_path: Option<String>,
    #[serde(default = "default_backup_count")]
    backup_count: usize,
    #[serde(default)]
    backup_path: Option<String>,
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
        })
    } else {
        Config {
            file_path: None,
            backup_count: 10,
            backup_path: None,
        }
    }
}

fn save_config(config_path: &Path, config: &Config) {
    if let Ok(content) = serde_json::to_string_pretty(config) {
        let _ = fs::write(config_path, content);
    }
}

fn do_backup(file_path: &str, backup_dir: &str) -> Result<String, String> {
    if !Path::new(file_path).exists() {
        return Err("数据文件不存在".to_string());
    }
    let _ = fs::create_dir_all(backup_dir);
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("mold-data-backup-{}.xlsx", timestamp);
    let backup_file = PathBuf::from(backup_dir).join(&backup_name);
    fs::copy(file_path, &backup_file).map_err(|e| e.to_string())?;
    Ok(backup_file.to_string_lossy().to_string())
}

fn cleanup_old_backups(backup_dir: &str, keep_count: usize) {
    if let Ok(entries) = fs::read_dir(backup_dir) {
        let mut backups: Vec<(String, std::time::SystemTime)> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map(|ext| ext == "xlsx").unwrap_or(false)
                    && e.path().file_name().and_then(|n| n.to_str())
                        .map(|n| n.contains("backup")).unwrap_or(false)
            })
            .filter_map(|e| {
                let time = e.metadata().ok()?.modified().ok()?;
                let name = e.file_name().to_string_lossy().to_string();
                Some((name, time))
            })
            .collect();
        backups.sort_by(|a, b| b.1.cmp(&a.1));
        for (name, _) in backups.into_iter().skip(keep_count) {
            let _ = fs::remove_file(Path::new(backup_dir).join(&name));
        }
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
    // 将所有值转为字符串
    let map: std::collections::HashMap<String, String> = item.as_object()
        .map(|obj| obj.iter().map(|(k, v)| {
            let val = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
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
    // 将所有值转为字符串
    let map: std::collections::HashMap<String, String> = data.as_object()
        .map(|obj| obj.iter().map(|(k, v)| {
            let val = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
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
    excel::delete_row(&path, &sheet_name, &id)
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
    let backup_dir = get_backup_dir(&config);
    let backup_file = do_backup(&path, &backup_dir)?;
    let count = config.backup_count;
    drop(config);
    drop(path);
    cleanup_old_backups(&backup_dir, count);
    Ok(json!({ "success": true, "backupPath": backup_file }))
}

#[tauri::command]
fn get_backup_config(state: State<AppState>) -> Result<Value, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir(&config);
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
fn list_backups(state: State<AppState>) -> Result<Value, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir(&config);
    let mut backups = Vec::new();
    if let Ok(entries) = fs::read_dir(&backup_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "xlsx").unwrap_or(false) {
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let modified = entry.metadata().ok()
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        let datetime: chrono::DateTime<Local> = t.into();
                        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                    })
                    .unwrap_or_default();
                backups.push(json!({
                    "name": name,
                    "path": path.to_string_lossy(),
                    "size": size,
                    "modified": modified,
                }));
            }
        }
    }
    backups.sort_by(|a, b| {
        let a_mod = a["modified"].as_str().unwrap_or("");
        let b_mod = b["modified"].as_str().unwrap_or("");
        b_mod.cmp(a_mod)
    });
    Ok(json!({ "backups": backups }))
}

#[tauri::command]
fn restore_backup(state: State<AppState>, backup_path: String) -> Result<Value, String> {
    if !Path::new(&backup_path).exists() {
        return Err("备份文件不存在".to_string());
    }
    let file_path = state.file_path.lock().map_err(|e| e.to_string())?;
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let backup_dir = get_backup_dir(&config);
    let _ = do_backup(&file_path, &backup_dir);
    fs::copy(&backup_path, &*file_path).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true }))
}

fn default_backup_dir() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("data").join("backups").to_string_lossy().to_string()))
        .unwrap_or_else(|| "./data/backups".to_string())
}

fn get_backup_dir(config: &Config) -> String {
    config.backup_path.clone().unwrap_or_else(default_backup_dir)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config_path = get_config_path();
    let config = load_config(&config_path);

    let initial_path = config.file_path.clone()
        .filter(|p| !p.is_empty() && Path::new(p).exists())
        .unwrap_or_else(|| excel::get_default_file_path());

    // 启动时备份
    let backup_dir = get_backup_dir(&config);
    let _ = do_backup(&initial_path, &backup_dir);
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
            restore_backup,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                // 退出时备份 - 通过 app_handle 获取 state
                if let Some(state) = app_handle.try_state::<AppState>() {
                    let fp = state.file_path.lock().unwrap().clone();
                    let config = state.config.lock().unwrap().clone();
                    let backup_dir = get_backup_dir(&config);
                    let _ = do_backup(&fp, &backup_dir);
                    cleanup_old_backups(&backup_dir, config.backup_count);
                }
            }
        });
}
