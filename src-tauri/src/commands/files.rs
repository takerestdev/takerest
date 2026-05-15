use std::path::PathBuf;
use crate::error::AppError;

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
