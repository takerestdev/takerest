use tauri::{WebviewUrl, WebviewWindowBuilder};
use window_vibrancy::apply_acrylic;

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;
#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn open_folder_window(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let label = format!("folder-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());

    let encoded = path.replace(' ', "%20");
    let url = format!("/app?path={}", encoded);

    let mut win_builder = WebviewWindowBuilder::new(
        &app,
        &label,
        WebviewUrl::App(url.into()),
    )
    .title(path.split(['/', '\\']).last().unwrap_or("Project"))
    .inner_size(1200.0, 800.0)
    .transparent(true);

    #[cfg(target_os = "windows")]
    { win_builder = win_builder.decorations(false); }

    #[cfg(target_os = "macos")]
    { win_builder = win_builder.title_bar_style(TitleBarStyle::Transparent); }

    let window = win_builder.build().map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    apply_acrylic(&window, Some((18, 18, 18, 125))).map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, open_folder_window])
        .setup(|app| {
            let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("takerest")
                .inner_size(1200.0, 800.0)
                .transparent(true);

            #[cfg(target_os = "macos")]
            let win_builder = win_builder.title_bar_style(TitleBarStyle::Transparent);

            #[cfg(target_os = "windows")]
            let win_builder = win_builder.decorations(false);

            let window = win_builder.build()?;

            #[cfg(target_os = "windows")]
            apply_acrylic(&window, Some((18, 18, 18, 125)))
                .expect("Failed to apply acrylic effect");

            #[cfg(target_os = "macos")]
            apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
                .expect("Failed to apply vibrancy effect");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}