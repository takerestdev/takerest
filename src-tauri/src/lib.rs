mod commands;
mod error;
mod utils;

use commands::watcher::WatcherState;
use std::sync::Mutex;
use tauri::{WebviewUrl, WebviewWindowBuilder};
use window_vibrancy::apply_acrylic;

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;
#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(WatcherState(Mutex::new(None)))
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // project commands
            commands::project::init_project,
            commands::project::scan_project,
            commands::project::save_readme,
            // env commands
            commands::env::list_env_files,
            commands::env::read_env_file,
            commands::env::write_env_file,
            commands::env::create_env_file,
            commands::env::delete_env_file,
            commands::env::add_env_to_gitignore,
            commands::env::remove_env_from_gitignore,
            // git commands
            commands::git::git_status,
            commands::git::git_diff_file,
            commands::git::git_stage_file,
            commands::git::git_unstage_file,
            commands::git::git_stage_all,
            commands::git::git_unstage_all,
            commands::git::git_commit,
            commands::git::git_log,
            commands::git::git_branches,
            commands::git::git_checkout_branch,
            commands::git::git_create_branch,
            commands::git::git_stash,
            commands::git::git_checkout_force,
            commands::git::git_fetch,
            commands::git::git_pull,
            commands::git::git_merge_abort,
            commands::git::git_push,
            commands::git::git_publish_branch,
            commands::git::git_commit_files,
            commands::git::git_diff_commit_file,
            commands::git::git_read_blob_worktree,
            commands::git::git_read_blob_head,
            commands::git::git_remote_status,
            // api commands
            commands::api::init_requests_dir,
            commands::api::get_request_tree,
            commands::api::read_request,
            commands::api::create_request,
            commands::api::update_request,
            commands::api::delete_request,
            commands::api::duplicate_request,
            commands::api::create_collection,
            // watcher commands
            commands::watcher::watch_project,
            commands::watcher::unwatch_project,
        ])
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
