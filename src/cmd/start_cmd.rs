use anyhow::Result;
use chrono::Local;

use crate::config::Config;
use crate::storage::{find_task, list_tasks, write_task};
use crate::task::TaskStatus;
use crate::utils::detect_project;

use super::TaskIdentifier;

pub fn run(args: TaskIdentifier) -> Result<()> {
    let config = Config::ensure_loaded()?;
    let project = detect_project()?;
    let tasks = list_tasks(&config.vault.path, Some(&project))?;

    let task = find_task(&tasks, &args.query)
        .ok_or_else(|| anyhow::anyhow!("task not found: {}", args.query))?;

    // Clone so we can mutate
    let mut task = task.clone();

    let was_done = task.frontmatter.status == TaskStatus::Done;

    if task.frontmatter.status == TaskStatus::InProgress {
        println!("Task is already in progress: {}", task.frontmatter.title);
        return Ok(());
    }

    let previous_start = task.frontmatter.start_date.clone();

    // Update frontmatter
    task.frontmatter.status = TaskStatus::InProgress;
    task.frontmatter.start_date = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

    write_task(&task)?;

    if was_done {
        println!(
            "⚠ Restarted: {} (was done)",
            task.frontmatter.title
        );
        if let Some(prev) = previous_start {
            println!("  Previous start: {}", prev);
        }
    } else {
        println!("Started: {}", task.frontmatter.title);
    }

    Ok(())
}
