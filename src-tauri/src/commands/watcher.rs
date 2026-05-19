use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify_debouncer_full::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
    DebounceEventResult, Debouncer, FileIdCache, FileIdMap, NoCache,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::error::AppError;

#[cfg(target_os = "linux")]
type DebouncerCache = NoCache;
#[cfg(not(target_os = "linux"))]
type DebouncerCache = FileIdMap;

pub struct WatcherState(
    pub Mutex<Option<Debouncer<RecommendedWatcher, DebouncerCache>>>,
    pub Arc<AtomicU64>,
);

#[derive(Serialize, Clone)]
pub struct FsChangedPayload {
    pub modified: Vec<String>,
    pub created: Vec<String>,
    pub deleted: Vec<String>,
}

fn to_rel(base: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(base).ok()?;
    let s = rel.to_string_lossy().replace('\\', "/");

    if s.ends_with(".lock")
        || s.starts_with(".git/objects/")
        || s.starts_with(".git/logs/")
        || s.starts_with(".git/pack-refs")
    {
        return None;
    }

    Some(s)
}

// Large build/dependency directories to never scan or watch
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    "dist",
    ".svelte-kit",
    ".next",
    ".nuxt",
    ".output",
    ".cache",
    "__pycache__",
    ".venv",
    "venv",
    ".tox",
    "coverage",
    ".turbo",
    ".expo",
    ".parcel-cache",
    "out",
];

fn is_skip_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
}

// Selective watch: root non-recursive + each non-skip depth-1 dir recursive.
// This avoids NoCache scanning node_modules / target which can have 100k+ files,
// while ensuring new subdirectories created under watched dirs are picked up.
fn setup_watches<C: FileIdCache>(debouncer: &mut Debouncer<RecommendedWatcher, C>, root: &Path) {
    // Root itself (non-recursive) — catches root-level file changes
    let _ = debouncer.watch(root, RecursiveMode::NonRecursive);

    let Ok(depth1) = std::fs::read_dir(root) else { return };
    for d1_entry in depth1.flatten() {
        let Ok(ft) = d1_entry.file_type() else { continue };
        if !ft.is_dir() { continue; }

        let d1_name = d1_entry.file_name();
        if is_skip_dir(&d1_name.to_string_lossy()) { continue; }

        // Recursive so new subdirectories created later are automatically covered
        let _ = debouncer.watch(&d1_entry.path(), RecursiveMode::Recursive);
    }
}

#[tauri::command]
pub fn watch_project(
    app: AppHandle,
    state: State<'_, WatcherState>,
    project_path: String,
) -> Result<(), AppError> {
    // Drop existing watcher and advance the generation counter atomically.
    // The background thread captures my_gen and aborts if the counter has moved on.
    let my_gen = {
        let mut guard = state
            .0
            .lock()
            .map_err(|_| AppError::Other("watcher lock poisoned".into()))?;
        *guard = None;
        state.1.fetch_add(1, Ordering::SeqCst) + 1
    };

    // Run the actual setup in a background thread so this command returns
    // immediately and doesn't block the Tauri thread pool.
    let app_clone = app.clone();
    let gen_arc = Arc::clone(&state.1);
    std::thread::spawn(move || {
        let root = std::path::PathBuf::from(&project_path);
        let base = root.clone();
        let app_for_events = app_clone.clone();
        let gen_for_events = Arc::clone(&gen_arc);

        let debouncer_result = new_debouncer(
            Duration::from_millis(250),
            None,
            move |result: DebounceEventResult| {
                // Abort if a newer watch_project call has superseded this one
                if gen_for_events.load(Ordering::SeqCst) != my_gen { return; }

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

                let _ = app_for_events
                    .emit("fs:changed", FsChangedPayload { modified, created, deleted });
            },
        );

        let mut debouncer = match debouncer_result {
            Ok(d) => d,
            Err(e) => {
                eprintln!("watcher: failed to create debouncer: {e}");
                return;
            }
        };

        setup_watches(&mut debouncer, &root);

        // Only store if this thread is still the current one
        let watcher_state = app_clone.state::<WatcherState>();
        if let Ok(mut guard) = watcher_state.0.lock() {
            if watcher_state.1.load(Ordering::SeqCst) == my_gen {
                *guard = Some(debouncer);
            }
        };
    });

    Ok(())
}

#[tauri::command]
pub fn unwatch_project(state: State<'_, WatcherState>) -> Result<(), AppError> {
    let mut guard = state
        .0
        .lock()
        .map_err(|_| AppError::Other("watcher lock poisoned".into()))?;
    *guard = None;
    state.1.fetch_add(1, Ordering::SeqCst);
    Ok(())
}
