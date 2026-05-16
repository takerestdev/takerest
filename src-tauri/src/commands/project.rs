use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::error::AppError;
use crate::utils::scanner;

// ── Types ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    /// Whether .anide/ folder existed (false = we just created it)
    pub anide_initialized: bool,

    /// .env file paths relative to project root
    /// Matches any file with ".env" in name: .env, .env.local, .env.production, etc.
    pub env_files: Vec<String>,

    /// docker-compose file paths relative to project root
    /// Matches any file with "docker-compose" in name
    pub compose_files: Vec<String>,

    /// Git info if this is a git repo, None otherwise
    pub git: Option<GitInfo>,

    /// Dominant file extension in the project, e.g. ".ts", ".py", ".rs"
    pub major_filetype: Option<FiletypeInfo>,

    /// Project metadata from .takerest/README.md, None if not found or invalid
    pub readme_metadata: Option<ReadmeMetadata>,
    
    /// Root README.md content from project root, None if not found
    pub root_readme: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitInfo {
    /// Repository name (derived from folder name)
    pub repo_name: String,
    /// Current branch name (or detached HEAD hash)
    pub branch: String,
    /// True if this path is a git worktree (not the main worktree)
    pub is_worktree: bool,
    /// Worktree name if is_worktree is true
    pub worktree_name: Option<String>,
}

#[derive(Serialize)]
pub struct FiletypeInfo {
    /// Extension including dot, e.g. ".ts"
    pub extension: String,
    /// Number of files with this extension
    pub count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadmeMetadata {
    /// Project name from README header
    pub project_name: String,
    /// Full content of the README
    pub content: String,
}

fn init_config_file(anide_dir: &Path, project_path: &str) -> Result<(), AppError> {
    let project_name = Path::new(project_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project");

    let content = format!(
        r#"# {project_name}

> Managed by [Anide](https://anide.app) — AI-native developer workspace.

---

Clone the repo, open Anide, point it here. Everything is ready.
"#
    );

    fs::write(anide_dir.join("README.md"), content)?;
    Ok(())
}

// ── Commands ─────────────────────────────────────────────────────────────

/// Ensures .takerest/ folder exists. Creates it if missing.
/// Returns true if it already existed, false if it was just created.
#[tauri::command]
pub fn init_project(project_path: String) -> Result<bool, AppError> {
    let root = Path::new(&project_path);
    if !root.is_dir() {
        return Err(AppError::InvalidPath(format!(
            "Project path is not a directory: {}",
            project_path
        )));
    }

    let anide_dir = root.join(".anide");
    if anide_dir.exists() && anide_dir.is_dir() {
        Ok(true)
    } else if anide_dir.exists() && !anide_dir.is_dir() {
        Err(AppError::InvalidPath(
            ".anide exists but is not a directory".to_string()
        ))
    } else {
        fs::create_dir_all(&anide_dir)?;
        init_config_file(&anide_dir, &project_path)?;
        Ok(false)
    }
}

/// Scans the project folder and returns aggregated info.
/// Uses .gitignore rules to skip ignored paths for performance.
#[tauri::command]
pub fn scan_project(project_path: String) -> Result<ProjectInfo, AppError> {
    let root = Path::new(&project_path);
    if !root.is_dir() {
        return Err(AppError::InvalidPath(format!(
            "Project path is not a directory: {}",
            project_path
        )));
    }

    // Ensure .anide/ exists
    let anide_initialized = init_project(project_path.clone())?;

    // Collect results in a single pass
    let mut env_files: Vec<String> = Vec::new();
    let mut compose_files: Vec<String> = Vec::new();
    let mut ext_counts: HashMap<String, usize> = HashMap::new();

    for entry in scanner::walk_project(&project_path) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // skip permission errors etc.
        };

        // Skip directories themselves — we only care about files
        if entry.file_type().map_or(true, |ft| ft.is_dir()) {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();

        // Get relative path from project root
        let rel_path = match entry.path().strip_prefix(root) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => continue,
        };

        // Skip files inside .anide/ — those are our own files
        if Path::new(&rel_path)
            .components()
            .next()
            .map(|c| c.as_os_str() == OsStr::new(".anide"))
            .unwrap_or(false)
        {
            continue;
        }

        // Check for .env files (filename contains ".env")
        if file_name.contains(".env") {
            env_files.push(rel_path.clone());
        }

        // Check for docker-compose files (exact filename matches)
        let file_name_lower = file_name.to_lowercase();
        if file_name_lower == "docker-compose.yml"
            || file_name_lower == "docker-compose.yaml"
            || file_name_lower == "compose.yml"
            || file_name_lower == "compose.yaml"
        {
            compose_files.push(rel_path.clone());
        }

        // Track file extensions for major filetype detection
        if let Some(ext) = Path::new(&file_name).extension() {
            let ext_str = format!(".{}", ext.to_string_lossy());
            *ext_counts.entry(ext_str).or_insert(0) += 1;
        }
    }

    // Sort for consistent output
    env_files.sort();
    compose_files.sort();

    // Find the dominant file extension
    let major_filetype = ext_counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)))
        .map(|(extension, count)| FiletypeInfo { extension, count });

    // Git detection — read .git/HEAD directly (no git2 dependency needed)
    let git = detect_git(root);

    // Read README.md metadata from .anide/ folder
    let readme_metadata = read_readme_metadata(root);
    let root_readme = read_root_readme(root);

    Ok(ProjectInfo {
        anide_initialized,
        env_files,
        compose_files,
        git,
        major_filetype,
        readme_metadata,
        root_readme,
    })
}

/// Save the README.md file in .takerest/ folder
#[tauri::command]
pub fn save_readme(project_path: String, content: String) -> Result<(), AppError> {
    
    let root = Path::new(&project_path);
    if !root.is_dir() {
        return Err(AppError::InvalidPath(format!(
            "Project path is not a directory: {}",
            project_path
        )));
    }

    let readme_path = root.join("README.md");

    // Safety: Check if the path is a symlink before writing
    // Using symlink_metadata to avoid following the symlink
    match fs::symlink_metadata(&readme_path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(AppError::InvalidPath(
                    "Cannot write to README.md: path is a symlink".to_string()
                ));
            }
            // Path exists and is not a symlink - safe to write
        }
        Err(_) => {
            // Path doesn't exist - safe to create and write
        }
    }

    // Write the README.md file
    fs::write(readme_path, content)?;
    
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────

/// Detect if the folder is a git repo by reading .git/HEAD.
/// Returns repo name (folder name) and current branch.
/// Handles worktrees and submodules where .git is a file pointer.
fn detect_git(root: &Path) -> Option<GitInfo> {
    let git_path = root.join(".git");
    
    // Determine the actual git directory; track whether this is a linked worktree
    let (git_dir, is_worktree) = if git_path.is_dir() {
        (git_path, false)
    } else if git_path.is_file() {
        // Git worktree or submodule — .git is a file containing "gitdir: <path>"
        let git_file_content = fs::read_to_string(&git_path).ok()?;
        let git_file_trimmed = git_file_content.trim();

        if let Some(gitdir_path) = git_file_trimmed.strip_prefix("gitdir: ") {
            let gitdir_path = gitdir_path.trim();
            let resolved_path = if Path::new(gitdir_path).is_absolute() {
                Path::new(gitdir_path).to_path_buf()
            } else {
                root.join(gitdir_path)
            };

            if resolved_path.is_dir() {
                // Linked worktrees live under .git/worktrees/<name>; check immediate parent only
                let is_linked = resolved_path
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|n| n == "worktrees")
                    .unwrap_or(false);
                (resolved_path, is_linked)
            } else {
                return None;
            }
        } else {
            return None;
        }
    } else {
        return None;
    };

    let head_path = git_dir.join("HEAD");
    let head_content = fs::read_to_string(&head_path).ok()?;
    let head_trimmed = head_content.trim();

    let branch = if let Some(ref_path) = head_trimmed.strip_prefix("ref: refs/heads/") {
        ref_path.to_string()
    } else {
        head_trimmed.chars().take(8).collect()
    };

    let repo_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Worktree name = the folder name of the linked worktree git dir
    let worktree_name = if is_worktree {
        git_dir.file_name().map(|n| n.to_string_lossy().to_string())
    } else {
        None
    };

    Some(GitInfo { repo_name, branch, is_worktree, worktree_name })
}

/// Read the README.md file from project root.
fn read_root_readme(root: &Path) -> Option<String> {
    let readme_path = root.join("README.md");
    
    if !readme_path.is_file() {
        return None;
    }
    
    fs::read_to_string(&readme_path).ok()
}

/// Read and parse the .anide/README.md file for project metadata.
fn read_readme_metadata(root: &Path) -> Option<ReadmeMetadata> {
    let readme_path = root.join(".anide").join("README.md");
    
    if !readme_path.is_file() {
        return None;
    }
    
    let content = fs::read_to_string(&readme_path).ok()?;
    
    // Extract project name from the first markdown header (# Project Name)
    let project_name = content
        .lines()
        .find(|line| line.starts_with("# "))
        .and_then(|line| line.strip_prefix("# "))
        .map(|name| name.trim().to_string())
        .unwrap_or_else(|| "Unknown Project".to_string());
    
    Some(ReadmeMetadata {
        project_name,
        content,
    })
}