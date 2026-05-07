use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;

// ── Types ────────────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnvFile {
    /// Filename only, e.g. ".env.local"
    pub name: String,
    /// Relative path from project root, e.g. "backend/.env.local"
    pub rel_path: String,
    /// Absolute path (used internally for read/write)
    pub abs_path: String,
    /// Whether this file is currently listed in .gitignore
    pub in_gitignore: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvFileContent {
    pub rel_path: String,
    pub content: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEnvFilePayload {
    /// e.g. ".env.local" or "backend/.env.staging"
    pub rel_path: String,
    /// Initial content, can be empty string
    pub content: String,
    /// Whether to immediately add to .gitignore
    pub add_to_gitignore: bool,
}

// ── Helpers ──────────────────────────────────────────────────────────────

fn resolve_gitignore_path(root: &Path) -> PathBuf {
    root.join(".gitignore")
}

/// Read .gitignore lines. Returns empty vec if file doesn't exist.
fn read_gitignore_lines(root: &Path) -> Vec<String> {
    let path = resolve_gitignore_path(root);
    match fs::read_to_string(&path) {
        Ok(content) => content.lines().map(|l| l.to_string()).collect(),
        Err(_) => Vec::new(),
    }
}

/// Write lines back to .gitignore, creating it if needed.
fn write_gitignore_lines(root: &Path, lines: &[String]) -> Result<(), AppError> {
    let path = resolve_gitignore_path(root);
    let content = lines.join("\n");
    // Preserve trailing newline if file had one or if it's non-empty
    let content = if content.is_empty() {
        content
    } else {
        format!("{}\n", content)
    };
    fs::write(&path, content)?;
    Ok(())
}

/// Check whether a relative path appears in .gitignore.
/// Matches both the exact entry and a leading-slash variant ("/rel_path").
fn is_in_gitignore(lines: &[String], rel_path: &str) -> bool {
    let normalized = rel_path.replace('\\', "/");
    let with_slash = format!("/{}", normalized);
    lines.iter().any(|line| {
        let trimmed = line.trim();
        trimmed == normalized || trimmed == with_slash.as_str()
    })
}

/// Collect all .env files under `root`, respecting .gitignore presence.
fn collect_env_files(root: &Path) -> Vec<EnvFile> {
    let gitignore_lines = read_gitignore_lines(root);
    let mut files: Vec<EnvFile> = Vec::new();

    collect_env_recursive(root, root, &gitignore_lines, &mut files);
    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    files
}

fn collect_env_recursive(
    root: &Path,
    dir: &Path,
    gitignore_lines: &[String],
    acc: &mut Vec<EnvFile>,
) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Skip hidden dirs (except .env files themselves), node_modules, .git
        if path.is_dir() {
            if file_name == "node_modules"
                || file_name == ".git"
                || file_name == ".takerest"
                || file_name == "target"
                || file_name == "dist"
                || file_name == ".next"
                || file_name == ".svelte-kit"
            {
                continue;
            }
            collect_env_recursive(root, &path, gitignore_lines, acc);
            continue;
        }

        // Only include files with ".env" in name
        if !file_name.contains(".env") {
            continue;
        }

        let rel_path = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| file_name.clone());

        let in_gitignore = is_in_gitignore(gitignore_lines, &rel_path);

        acc.push(EnvFile {
            name: file_name,
            rel_path,
            abs_path: path.to_string_lossy().to_string(),
            in_gitignore,
        });
    }
}

// ── Commands ─────────────────────────────────────────────────────────────

/// List all .env* files found in the project.
#[tauri::command]
pub fn list_env_files(project_path: String) -> Result<Vec<EnvFile>, AppError> {
    let root = Path::new(&project_path);
    if !root.is_dir() {
        return Err(AppError::InvalidPath(format!(
            "Project path is not a directory: {}",
            project_path
        )));
    }

    Ok(collect_env_files(root))
}

/// Read the content of a specific .env file by relative path.
#[tauri::command]
pub fn read_env_file(project_path: String, rel_path: String) -> Result<EnvFileContent, AppError> {
    let root = Path::new(&project_path);
    let abs_path = root.join(&rel_path);

    if !abs_path.is_file() {
        return Err(AppError::InvalidPath(format!(
            "File not found: {}",
            rel_path
        )));
    }

    let content = fs::read_to_string(&abs_path)?;
    Ok(EnvFileContent { rel_path, content })
}

/// Write (overwrite) the content of a .env file by relative path.
/// Creates parent directories if they don't exist.
#[tauri::command]
pub fn write_env_file(
    project_path: String,
    rel_path: String,
    content: String,
) -> Result<(), AppError> {
    let root = Path::new(&project_path);
    let abs_path = root.join(&rel_path);

    // Safety: only allow paths that are children of project root
    if !abs_path.starts_with(root) {
        return Err(AppError::InvalidPath(
            "rel_path escapes the project directory".to_string(),
        ));
    }

    if let Some(parent) = abs_path.parent() {
        fs::create_dir_all(parent)?;
    }

    println!("Writing to {}: {}", abs_path.display(), content);

    fs::write(&abs_path, content)?;
    Ok(())
}

/// Create a new .env file. Returns an error if it already exists.
#[tauri::command]
pub fn create_env_file(
    project_path: String,
    payload: CreateEnvFilePayload,
) -> Result<EnvFile, AppError> {
    let root = Path::new(&project_path);
    let rel_path_normalized = payload.rel_path.replace('\\', "/");
    let abs_path = root.join(&rel_path_normalized);

    // Safety check
    if !abs_path.starts_with(root) {
        return Err(AppError::InvalidPath(
            "rel_path escapes the project directory".to_string(),
        ));
    }

    // Must contain ".env" in filename
    let file_name = abs_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if !file_name.contains(".env") {
        return Err(AppError::InvalidPath(format!(
            "Filename '{}' does not contain '.env'",
            file_name
        )));
    }

    if abs_path.exists() {
        return Err(AppError::InvalidPath(format!(
            "File already exists: {}",
            rel_path_normalized
        )));
    }

    if let Some(parent) = abs_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&abs_path, &payload.content)?;

    let mut in_gitignore = false;
    if payload.add_to_gitignore {
        add_to_gitignore_internal(root, &rel_path_normalized)?;
        in_gitignore = true;
    } else {
        // Still check if it's already there
        let lines = read_gitignore_lines(root);
        in_gitignore = is_in_gitignore(&lines, &rel_path_normalized);
    }

    Ok(EnvFile {
        name: file_name.to_string(),
        rel_path: rel_path_normalized,
        abs_path: abs_path.to_string_lossy().to_string(),
        in_gitignore,
    })
}

/// Delete a .env file.
#[tauri::command]
pub fn delete_env_file(project_path: String, rel_path: String) -> Result<(), AppError> {
    let root = Path::new(&project_path);
    let abs_path = root.join(&rel_path);

    if !abs_path.starts_with(root) {
        return Err(AppError::InvalidPath(
            "rel_path escapes the project directory".to_string(),
        ));
    }

    if !abs_path.is_file() {
        return Err(AppError::InvalidPath(format!(
            "File not found: {}",
            rel_path
        )));
    }

    fs::remove_file(&abs_path)?;
    Ok(())
}

/// Add a .env file's relative path to .gitignore.
#[tauri::command]
pub fn add_env_to_gitignore(project_path: String, rel_path: String) -> Result<(), AppError> {
    let root = Path::new(&project_path);
    add_to_gitignore_internal(root, &rel_path.replace('\\', "/"))
}

fn add_to_gitignore_internal(root: &Path, rel_path: &str) -> Result<(), AppError> {
    let mut lines = read_gitignore_lines(root);

    if is_in_gitignore(&lines, rel_path) {
        return Ok(()); // already present, no-op
    }

    // Find or create a # .env section for grouping, otherwise just append
    let marker = "# .env files";
    let marker_pos = lines.iter().position(|l| l.trim() == marker);

    if let Some(pos) = marker_pos {
        // Insert after the marker (and any consecutive .env entries)
        let mut insert_at = pos + 1;
        while insert_at < lines.len()
            && (lines[insert_at].trim().contains(".env")
                || lines[insert_at].trim().is_empty())
            && insert_at < pos + 10
        {
            insert_at += 1;
        }
        lines.insert(insert_at, rel_path.to_string());
    } else {
        // Append with section header if this is the first .env entry
        if !lines.is_empty() && !lines.last().map(|l| l.is_empty()).unwrap_or(true) {
            lines.push(String::new());
        }
        lines.push(marker.to_string());
        lines.push(rel_path.to_string());
    }

    write_gitignore_lines(root, &lines)
}

/// Remove a .env file's relative path from .gitignore.
#[tauri::command]
pub fn remove_env_from_gitignore(project_path: String, rel_path: String) -> Result<(), AppError> {
    let root = Path::new(&project_path);
    let normalized = rel_path.replace('\\', "/");
    let with_slash = format!("/{}", normalized);

    let lines = read_gitignore_lines(root);
    let filtered: Vec<String> = lines
        .into_iter()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed != normalized.as_str() && trimmed != with_slash.as_str()
        })
        .collect();

    write_gitignore_lines(root, &filtered)
}