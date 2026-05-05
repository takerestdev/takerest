use tauri::{WebviewUrl, WebviewWindowBuilder};
#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("takerest.dev")
                .inner_size(1200.0, 800.0);

            #[cfg(target_os = "macos")]
            let win_builder = win_builder.title_bar_style(TitleBarStyle::Transparent);

            #[cfg(not(target_os = "macos"))]
            let win_builder = win_builder.decorations(false);

            win_builder.build().unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}