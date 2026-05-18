use std::ffi::OsStr;
use std::path::PathBuf;
use crate::error::AppError;
use crate::utils::scanner;

fn resolve_in_project(project_path: &str, rel_path: &str) -> Result<PathBuf, AppError> {
    let root = PathBuf::from(project_path);
    let canonical_root = root.canonicalize()?;

    // For reads the file must exist; canonicalize validates that
    let full = root.join(rel_path);
    let canonical_full = full
        .canonicalize()
        .map_err(|_| AppError::NotFound(rel_path.to_string()))?;

    if !canonical_full.starts_with(&canonical_root) {
        return Err(AppError::InvalidPath(rel_path.to_string()));
    }
    Ok(canonical_full)
}

fn resolve_write_in_project(project_path: &str, rel_path: &str) -> Result<PathBuf, AppError> {
    let root = PathBuf::from(project_path);
    let canonical_root = root.canonicalize()?;
    let full = root.join(rel_path);

    // The file may not exist yet; validate via the parent directory
    let parent = full
        .parent()
        .ok_or_else(|| AppError::InvalidPath(rel_path.to_string()))?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|_| AppError::InvalidPath(rel_path.to_string()))?;

    if !canonical_parent.starts_with(&canonical_root) {
        return Err(AppError::InvalidPath(rel_path.to_string()));
    }
    Ok(full)
}

#[tauri::command]
pub fn read_project_file(project_path: String, rel_path: String) -> Result<String, AppError> {
    let path = resolve_in_project(&project_path, &rel_path)?;
    std::fs::read_to_string(&path).map_err(AppError::Io)
}

#[tauri::command]
pub fn write_project_file(
    project_path: String,
    rel_path: String,
    content: String,
) -> Result<(), AppError> {
    let path = resolve_write_in_project(&project_path, &rel_path)?;
    std::fs::write(&path, content.as_bytes()).map_err(AppError::Io)
}

/// Returns relative paths (forward-slash) of all .md and .excalidraw files in the project.
/// Respects .gitignore and skips .anide/, .git/, node_modules/, etc.
#[tauri::command]
pub fn list_doc_files(project_path: String) -> Result<Vec<String>, AppError> {
    let root = PathBuf::from(&project_path);
    if !root.is_dir() {
        return Err(AppError::InvalidPath(project_path));
    }

    let mut files: Vec<String> = Vec::new();

    for entry in scanner::walk_project(&project_path) {
        let Ok(entry) = entry else { continue };
        if entry.file_type().map_or(true, |ft| ft.is_dir()) { continue; }

        let rel_path = match entry.path().strip_prefix(&root) {
            Ok(p) => p.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };

        // Skip .anide/ folder
        if std::path::Path::new(&rel_path)
            .components()
            .next()
            .map(|c| c.as_os_str() == OsStr::new(".anide"))
            .unwrap_or(false)
        {
            continue;
        }

        let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext == "md" || ext == "excalidraw" {
            files.push(rel_path);
        }
    }

    files.sort();
    Ok(files)
}

#[tauri::command]
pub fn delete_doc_file(project_path: String, rel_path: String) -> Result<(), AppError> {
    let path = resolve_in_project(&project_path, &rel_path)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "md" && ext != "excalidraw" {
        return Err(AppError::Other(
            "Only .md and .excalidraw files can be deleted via this command".to_string(),
        ));
    }
    std::fs::remove_file(&path).map_err(AppError::Io)
}
