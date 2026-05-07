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

    if task.frontmatter.status == TaskStatus::Done {
        println!("Task is already done: {}", task.frontmatter.title);
        return Ok(());
    }

    // Update frontmatter
    task.frontmatter.status = TaskStatus::Done;
    task.frontmatter.done_date = Some(Local::now().date_naive());

    write_task(&task)?;

    print!("Done: {}", task.frontmatter.title);

    // Show total duration if > 0
    if task.frontmatter.duration > 0 {
        let hours = task.frontmatter.duration / 60;
        let mins = task.frontmatter.duration % 60;
        if hours > 0 {
            println!(" ({}h {}m)", hours, mins);
        } else {
            println!(" ({}m)", mins);
        }
    } else {
        println!();
    }

    Ok(())
}
