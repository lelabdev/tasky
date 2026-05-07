use anyhow::{Context, Result};
use std::path::Path;

use crate::task::{Frontmatter, Task};

/// Read a markdown file with YAML frontmatter and return a Task.
///
/// Expected format:
/// ```md
/// ---
/// title: "My Task"
/// status: todo
/// created: 2024-01-15
/// ---
/// Body content here
/// ```
pub fn read_task(path: &Path) -> Result<Task> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read task file: {}", path.display()))?;

    let (frontmatter, body) = parse_frontmatter(&content)
        .with_context(|| format!("failed to parse frontmatter from: {}", path.display()))?;

    // Derive project name from the file's parent directory structure
    // Expected layout: <vault>/1_Projects/<project>/<filename>.md
    let project = derive_project(path);

    Ok(Task {
        frontmatter,
        body: body.to_string(),
        file_path: path.to_path_buf(),
        project,
    })
}

/// Write a Task to disk as a markdown file with YAML frontmatter.
pub fn write_task(task: &Task) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = task.file_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory: {}", parent.display()))?;
    }

    let yaml = serde_yaml::to_string(&task.frontmatter)
        .context("failed to serialize frontmatter to YAML")?;

    let content = format!("---\n{}---\n{}", yaml, task.body);

    std::fs::write(&task.file_path, content)
        .with_context(|| format!("failed to write task file: {}", task.file_path.display()))?;

    Ok(())
}

/// List all tasks in the vault, optionally filtered by project.
///
/// Walks the vault directory (or a specific project subdirectory),
/// reads all `.md` files with valid YAML frontmatter that contain a `status` field,
/// and returns them sorted by created date (newest first).
pub fn list_tasks(vault_path: &str, project: Option<&str>) -> Result<Vec<Task>> {
    let base = match project {
        Some(proj) => {
            let dir = crate::utils::get_tasky_dir(vault_path, proj);
            if !dir.exists() {
                return Ok(Vec::new());
            }
            dir
        }
        None => {
            let vault = Path::new(vault_path).join("1_Projects");
            if !vault.exists() {
                return Ok(Vec::new());
            }
            vault
        }
    };

    let mut tasks = Vec::new();
    collect_tasks(&base, &mut tasks)?;

    // Sort by created date, newest first
    tasks.sort_by(|a, b| b.frontmatter.created.cmp(&a.frontmatter.created));

    Ok(tasks)
}

/// Find a task by title (case-insensitive substring match) or issue number.
pub fn find_task<'a>(tasks: &'a [Task], query: &str) -> Option<&'a Task> {
    // Try numeric query as issue number first
    if let Ok(issue_num) = query.parse::<u64>() {
        if let Some(task) = tasks
            .iter()
            .find(|t| t.frontmatter.issue == Some(issue_num))
        {
            return Some(task);
        }
    }

    // Fall back to case-insensitive title contains
    let query_lower = query.to_lowercase();
    tasks
        .iter()
        .find(|t| t.frontmatter.title.to_lowercase().contains(&query_lower))
}

/// Convert a title into a filename-safe slug.
///
/// Lowercases, replaces spaces and special characters with `-`,
/// collapses consecutive dashes, and strips leading/trailing dashes.
///
/// Example: `"Fix login page!!!"` → `"fix-login-page"`
pub fn slugify_title(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();

    // Collapse consecutive dashes and trim edges
    let mut result = String::with_capacity(slug.len());
    let mut prev_dash = false;
    for ch in slug.chars() {
        if ch == '-' {
            if !prev_dash {
                result.push(ch);
            }
            prev_dash = true;
        } else {
            result.push(ch);
            prev_dash = false;
        }
    }

    // Trim leading/trailing dashes
    result.trim_matches('-').to_string()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Parse YAML frontmatter delimited by `---` from a markdown string.
///
/// Returns (Frontmatter, body_string).
fn parse_frontmatter(content: &str) -> Result<(Frontmatter, &str)> {
    let content = content
        .strip_prefix("---")
        .ok_or_else(|| anyhow::anyhow!("file does not start with '---' frontmatter delimiter"))?;

    // Find the closing ---
    let rest = content.strip_prefix('\n').unwrap_or(content);
    let end_pos = rest
        .find("\n---")
        .ok_or_else(|| anyhow::anyhow!("missing closing '---' frontmatter delimiter"))?;

    let yaml_str = &rest[..end_pos];
    let body = &rest[end_pos + 4..]; // skip "\n---"

    let frontmatter: Frontmatter =
        serde_yaml::from_str(yaml_str).context("failed to deserialize YAML frontmatter")?;

    Ok((frontmatter, body))
}

/// Derive project name from a file path.
///
/// Expected layout: `<vault>/1_Projects/<project>/...`
/// Looks for the `1_Projects` segment and takes the next component.
fn derive_project(path: &Path) -> String {
    let mut components = path.components().peekable();
    while let Some(comp) = components.next() {
        if comp.as_os_str() == "1_Projects" {
            if let Some(proj) = components.next() {
                return proj.as_os_str().to_string_lossy().to_string();
            }
        }
    }
    // Fallback: use parent directory name
    path.parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Recursively collect tasks from all `.md` files in a directory.
fn collect_tasks(dir: &Path, tasks: &mut Vec<Task>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("failed to read directory: {}", dir.display()))?
    {
        let entry = entry.context("failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() {
            collect_tasks(&path, tasks)?;
            continue;
        }

        // Only process .md files
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        // Try to read as a task; skip files without valid frontmatter
        match read_task(&path) {
            Ok(task) => tasks.push(task),
            Err(_) => continue, // Not a task file, skip silently
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskStatus;
    use chrono::NaiveDate;
    use std::fs;

    fn sample_frontmatter() -> Frontmatter {
        Frontmatter {
            title: "Test Task".to_string(),
            status: TaskStatus::Todo,
            created: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            issue: None,
            estimate: None,
            duration: 0,
            pomodoro_count: 0,
            start_date: None,
            done_date: None,
            tags: None,
        }
    }

    #[test]
    fn test_slugify_simple() {
        assert_eq!(slugify_title("Hello World"), "hello-world");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify_title("Fix login page!!!"), "fix-login-page");
    }

    #[test]
    fn test_slugify_multiple_spaces() {
        assert_eq!(slugify_title("a   big   task"), "a-big-task");
    }

    #[test]
    fn test_slugify_leading_trailing() {
        assert_eq!(slugify_title("  task  "), "task");
    }

    #[test]
    fn test_slugify_mixed() {
        assert_eq!(
            slugify_title("Implement OAuth2/SAML & JWT tokens"),
            "implement-oauth2-saml-jwt-tokens"
        );
    }

    #[test]
    fn test_read_write_roundtrip() {
        let dir = std::env::temp_dir().join("tasky_test_roundtrip");
        fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test-task.md");

        let fm = sample_frontmatter();
        let task = Task {
            frontmatter: fm.clone(),
            body: "This is the body.\nSecond line.\n".to_string(),
            file_path: file_path.clone(),
            project: "test-project".to_string(),
        };

        write_task(&task).unwrap();
        let read_back = read_task(&file_path).unwrap();

        assert_eq!(read_back.frontmatter.title, fm.title);
        assert_eq!(read_back.frontmatter.status, fm.status);
        assert_eq!(read_back.frontmatter.created, fm.created);
        assert!(read_back.body.contains("This is the body."));
        assert!(read_back.body.contains("Second line."));

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_read_with_issue() {
        let dir = std::env::temp_dir().join("tasky_test_issue");
        fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("issue-task.md");

        let content = "---\ntitle: \"Fix bug\"\nstatus: in progress\ncreated: 2024-06-10\nissue: 42\n---\nBug description here.\n";
        fs::write(&file_path, content).unwrap();

        let task = read_task(&file_path).unwrap();
        assert_eq!(task.frontmatter.title, "Fix bug");
        assert_eq!(task.frontmatter.status, TaskStatus::InProgress);
        assert_eq!(task.frontmatter.issue, Some(42));
        assert!(task.body.contains("Bug description here."));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_read_invalid_no_frontmatter() {
        let dir = std::env::temp_dir().join("tasky_test_invalid");
        fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("plain.md");

        fs::write(&file_path, "Just a plain markdown file.\n").unwrap();
        assert!(read_task(&file_path).is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_task_by_title() {
        let tasks = vec![make_task("Fix login", 1), make_task("Add logout", 2)];

        let found = find_task(&tasks, "login").unwrap();
        assert_eq!(found.frontmatter.title, "Fix login");

        let found = find_task(&tasks, "LOGOUT").unwrap();
        assert_eq!(found.frontmatter.title, "Add logout");
    }

    #[test]
    fn test_find_task_by_issue() {
        let tasks = vec![make_task("Fix login", 42), make_task("Add logout", 99)];

        let found = find_task(&tasks, "42").unwrap();
        assert_eq!(found.frontmatter.title, "Fix login");
    }

    #[test]
    fn test_find_task_none() {
        let tasks = vec![make_task("Fix login", 1)];
        assert!(find_task(&tasks, "nonexistent").is_none());
    }

    #[test]
    fn test_list_tasks_from_dir() {
        let vault = std::env::temp_dir().join("tasky_test_vault");
        let proj_dir = vault.join("1_Projects").join("myproject");
        fs::create_dir_all(&proj_dir).unwrap();

        // Write two task files
        let t1 =
            format!("---\ntitle: \"Task A\"\nstatus: todo\ncreated: 2024-06-10\n---\nBody A\n");
        let t2 =
            format!("---\ntitle: \"Task B\"\nstatus: done\ncreated: 2024-06-12\n---\nBody B\n");
        fs::write(proj_dir.join("task-a.md"), &t1).unwrap();
        fs::write(proj_dir.join("task-b.md"), &t2).unwrap();

        // Write a non-task md file (no status field — serde requires it,
        // so this will fail to parse and be silently skipped)
        fs::write(
            proj_dir.join("notes.md"),
            "---\ntitle: \"Notes\"\n---\nJust notes\n",
        )
        .unwrap();

        let tasks = list_tasks(vault.to_string_lossy().as_ref(), Some("myproject")).unwrap();

        assert_eq!(tasks.len(), 2);
        // Sorted newest first
        assert_eq!(tasks[0].frontmatter.title, "Task B");
        assert_eq!(tasks[1].frontmatter.title, "Task A");

        let _ = fs::remove_dir_all(&vault);
    }

    // Helper to build a Task for search tests
    fn make_task(title: &str, issue: u64) -> Task {
        Task {
            frontmatter: Frontmatter {
                title: title.to_string(),
                status: TaskStatus::Todo,
                created: NaiveDate::from_ymd_opt(2024, 6, 10).unwrap(),
                issue: Some(issue),
                estimate: None,
                duration: 0,
                pomodoro_count: 0,
                start_date: None,
                done_date: None,
                tags: None,
            },
            body: String::new(),
            file_path: std::path::PathBuf::from(format!("/tmp/{}.md", slugify_title(title))),
            project: "test".to_string(),
        }
    }
}
