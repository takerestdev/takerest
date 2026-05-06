use ignore::WalkBuilder;

/// Create a gitignore-aware walker for the project directory.
/// Respects .gitignore, .git/info/exclude, and global gitignore.
/// Does NOT skip dotfiles (we need .env), but does skip .git/ directory.
pub fn walk_project(project_path: &str) -> ignore::Walk {
    WalkBuilder::new(project_path)
        .hidden(false) // don't skip dotfiles — we want .env files
        .git_ignore(true) // respect .gitignore
        .git_global(true) // respect global gitignore
        .git_exclude(true) // respect .git/info/exclude
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            // Always skip .git directory itself (but not other dotfiles)
            if name == ".git" {
                return false;
            }
            true
        })
        .build()
}
