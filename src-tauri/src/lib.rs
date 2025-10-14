// 后端简化版本
// 注意：当前所有数据处理逻辑已在前端JavaScript中实现

use tauri::command;

// 简单的健康检查命令
#[command]
fn health_check() -> String {
    "Backend is running".to_string()
}

// 启动应用
pub fn run() {
    tauri::Builder::default()
        // 保留插件以便前端继续使用它们
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        // 只注册实际存在的命令
        .invoke_handler(tauri::generate_handler![
            health_check
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
