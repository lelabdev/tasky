use anyhow::Result;

use crate::config::Config;
use crate::pomodoro::Pomodoro;
use crate::storage::{find_task, list_tasks, write_task};
use crate::utils::{detect_project, extract_issue_from_branch, get_current_branch};

use super::PomodoroCommands;

pub fn run(args: super::PomodoroArgs) -> Result<()> {
    match args.command {
        PomodoroCommands::Start { task } => run_start(task),
        PomodoroCommands::Stop => run_stop(),
        PomodoroCommands::Status => run_status(),
        PomodoroCommands::Configure => run_configure(),
    }
}

fn run_start(task_flag: Option<String>) -> Result<()> {
    let config = Config::ensure_loaded()?;
    let pomodoro = Pomodoro::new(
        config.pomodoro.work_duration,
        config.pomodoro.short_break,
        config.pomodoro.long_break,
        config.pomodoro.long_break_interval,
    );

    // Run the work timer
    pomodoro.start(config.pomodoro.work_duration)?;

    // Find the task to update
    let project = detect_project()?;
    let tasks = list_tasks(&config.vault.path, Some(&project))?;

    let task = match &task_flag {
        Some(query) => find_task(&tasks, query)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("task not found: {}", query))?,
        None => {
            // Try to find task from current branch issue number
            let branch = get_current_branch()?;
            let issue = extract_issue_from_branch(&branch).ok_or_else(|| {
                anyhow::anyhow!(
                    "No active task detected. Use --task to specify a task, or run from a feature branch."
                )
            })?;
            find_task(&tasks, &issue.to_string())
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("task not found for issue #{}", issue))?
        }
    };

    // Update task: increment pomodoro count and duration
    let mut task = task;
    task.frontmatter.pomodoro_count += 1;
    task.frontmatter.duration += config.pomodoro.work_duration;
    write_task(&task)?;

    println!(
        "Updated: {} — {} pomodoro{}, {}m tracked",
        task.frontmatter.title,
        task.frontmatter.pomodoro_count,
        if task.frontmatter.pomodoro_count == 1 {
            ""
        } else {
            "s"
        },
        task.frontmatter.duration
    );

    // Offer break
    let break_duration = if task.frontmatter.pomodoro_count % config.pomodoro.long_break_interval == 0
    {
        config.pomodoro.long_break
    } else {
        config.pomodoro.short_break
    };

    if Pomodoro::prompt_break(break_duration) {
        pomodoro.run_break(break_duration)?;
    }

    Ok(())
}

fn run_stop() -> Result<()> {
    let config = Config::ensure_loaded()?;
    let pomodoro = Pomodoro::new(
        config.pomodoro.work_duration,
        config.pomodoro.short_break,
        config.pomodoro.long_break,
        config.pomodoro.long_break_interval,
    );
    pomodoro.stop()
}

fn run_status() -> Result<()> {
    let config = Config::ensure_loaded()?;
    let pomodoro = Pomodoro::new(
        config.pomodoro.work_duration,
        config.pomodoro.short_break,
        config.pomodoro.long_break,
        config.pomodoro.long_break_interval,
    );
    pomodoro.status()
}

fn run_configure() -> Result<()> {
    let config = Config::ensure_loaded()?;
    println!("Edit ~/.config/tasky/config.toml to change pomodoro settings.");
    println!();
    println!("Current settings:");
    println!(
        "  work_duration: {}min",
        config.pomodoro.work_duration
    );
    println!(
        "  short_break: {}min",
        config.pomodoro.short_break
    );
    println!(
        "  long_break: {}min",
        config.pomodoro.long_break
    );
    println!(
        "  long_break_interval: every {} pomodoros",
        config.pomodoro.long_break_interval
    );
    Ok(())
}
