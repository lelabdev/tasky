use anyhow::Result;

use crate::config::Config;
use crate::storage::list_tasks;
use crate::task::TaskStatus;
use crate::utils::{detect_project, extract_issue_from_branch, get_current_branch};

pub fn run() -> Result<()> {
    let config = Config::ensure_loaded()?;
    let project = detect_project()?;
    let branch = get_current_branch()?;
    let issue = extract_issue_from_branch(&branch)
        .ok_or_else(|| anyhow::anyhow!("No active task detected (branch doesn't contain an issue number)"))?;

    let tasks = list_tasks(&config.vault.path, Some(&project))?;

    let task = tasks
        .iter()
        .find(|t| t.frontmatter.issue == Some(issue) && t.frontmatter.status == TaskStatus::InProgress)
        .ok_or_else(|| anyhow::anyhow!("No active task to stop"))?;

    println!("Stopped: {}", task.frontmatter.title);
    println!(
        "Duration so far: {} pomodoro{} ({}m tracked)",
        task.frontmatter.pomodoro_count,
        if task.frontmatter.pomodoro_count == 1 { "" } else { "s" },
        task.frontmatter.duration
    );

    Ok(())
}
