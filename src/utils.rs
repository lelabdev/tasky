use std::path::PathBuf;

/// Get the tasky directory for a project in the vault
pub fn get_tasky_dir(vault_path: &str, project: &str) -> PathBuf {
    PathBuf::from(vault_path)
        .join("1_Projects")
        .join(project)
}

/// Detect project name from current directory
/// Priority: git remote → git root dir → current dir
pub fn detect_project() -> anyhow::Result<String> {
    // Try git remote URL first
    if let Ok(output) = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
    {
        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout);
            if let Some(name) = extract_repo_name(&url) {
                return Ok(name);
            }
        }
    }

    // Try git root directory name
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if output.status.success() {
            let root = String::from_utf8_lossy(&output.stdout);
            let root = root.trim();
            if let Some(name) = PathBuf::from(root).file_name() {
                return Ok(name.to_string_lossy().to_string());
            }
        }
    }

    // Fallback: current directory name
    let current = std::env::current_dir()?;
    let name = current
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("cannot determine project name"))?;
    Ok(name.to_string_lossy().to_string())
}

/// Extract repo name from a git remote URL
/// e.g. "https://github.com/lelabdev/tasky.git" → "tasky"
fn extract_repo_name(url: &str) -> Option<String> {
    let url = url.trim().trim_end_matches(".git");
    let parts: Vec<&str> = url.split('/').collect();
    parts.last().map(|s| s.to_string())
}

/// Get current git branch name
pub fn get_current_branch() -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .output()?;
    if !output.status.success() {
        anyhow::bail!("not in a git repository or no branch checked out");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Extract issue number from branch name
/// e.g. "feat/42-login-page" → Some(42)
pub fn extract_issue_from_branch(branch: &str) -> Option<u64> {
    // Try pattern: type/123-slug
    let parts: Vec<&str> = branch.split('/').collect();
    let last = parts.last()?;
    let num_part: Vec<&str> = last.split('-').collect();
    num_part.first()?.parse().ok()
}
