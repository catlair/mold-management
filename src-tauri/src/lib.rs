use tauri::Manager;
use tauri_plugin_shell::ShellExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 启动 sidecar 后端服务
            let sidecar_command = app.shell().sidecar("sidecar-server").unwrap();
            let (mut _rx, _child) = sidecar_command.spawn().expect("Failed to spawn sidecar");

            // 将 child 存储到 app 管理状态中，关闭时自动清理
            app.manage(_child);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
