use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use notify_debouncer_full::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
    DebounceEventResult, Debouncer, FileIdMap,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::error::AppError;

pub struct WatcherState(pub Mutex<Option<Debouncer<RecommendedWatcher, FileIdMap>>>);

#[derive(Serialize, Clone)]
pub struct FsChangedPayload {
    pub modified: Vec<String>,
    pub created: Vec<String>,
    pub deleted: Vec<String>,
}

/// Normalise a raw event path to a forward-slash relative path string,
/// or return None if we should ignore this path.
fn to_rel(base: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(base).ok()?;
    let s = rel.to_string_lossy().replace('\\', "/");

    // Ignore lock files and git internal churn
    if s.ends_with(".lock")
        || s == ".git/index"
        || s.starts_with(".git/objects/")
        || s.starts_with(".git/logs/")
        || s.starts_with(".git/pack-refs")
    {
        return None;
    }

    Some(s)
}

#[tauri::command]
pub fn watch_project(
    app: AppHandle,
    state: State<'_, WatcherState>,
    project_path: String,
) -> Result<(), AppError> {
    let mut guard = state.0.lock().map_err(|_| AppError::Other("watcher lock poisoned".into()))?;

    // Drop any existing watcher first
    *guard = None;

    let base = Path::new(&project_path).to_path_buf();
    let app_handle = app.clone();

    let debouncer = new_debouncer(
        Duration::from_millis(250),
        None,
        move |result: DebounceEventResult| {
            let events = match result {
                Ok(ev) => ev,
                Err(_) => return,
            };

            let mut modified: Vec<String> = Vec::new();
            let mut created: Vec<String> = Vec::new();
            let mut deleted: Vec<String> = Vec::new();

            for event in events {
                use notify_debouncer_full::notify::EventKind;

                for path in &event.event.paths {
                    let Some(rel) = to_rel(&base, path) else { continue };
                    match &event.event.kind {
                        EventKind::Create(_) => created.push(rel),
                        EventKind::Modify(_) => modified.push(rel),
                        EventKind::Remove(_) => deleted.push(rel),
                        _ => {}
                    }
                }
            }

            if modified.is_empty() && created.is_empty() && deleted.is_empty() {
                return;
            }

            let _ = app_handle.emit("fs:changed", FsChangedPayload { modified, created, deleted });
        },
    )
    .map_err(|e| AppError::Other(e.to_string()))?;

    // Watch the project root recursively
    let mut debouncer = debouncer;
    debouncer
        .watch(Path::new(&project_path), RecursiveMode::Recursive)
        .map_err(|e| AppError::Other(e.to_string()))?;

    *guard = Some(debouncer);
    Ok(())
}

#[tauri::command]
pub fn unwatch_project(state: State<'_, WatcherState>) -> Result<(), AppError> {
    let mut guard = state.0.lock().map_err(|_| AppError::Other("watcher lock poisoned".into()))?;
    *guard = None;
    Ok(())
}
