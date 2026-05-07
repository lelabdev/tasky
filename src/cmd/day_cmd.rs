use anyhow::Result;
use chrono::Local;

use crate::config::Config;
use crate::storage::list_tasks;
use crate::task::TaskStatus;

pub fn run() -> Result<()> {
    // 1. Load config
    let config = Config::ensure_loaded()?;

    // 2. Get ALL tasks across ALL projects
    let tasks = list_tasks(&config.vault.path, None)?;

    // 3. Determine today's date
    let today = Local::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();

    // 4. Filter tasks relevant to today
    let daily: Vec<_> = tasks
        .into_iter()
        .filter(|t| {
            // Tasks done today
            if t.frontmatter.done_date == Some(today) {
                return true;
            }
            // Tasks whose start_date contains today's date
            if let Some(ref start) = t.frontmatter.start_date {
                if start.contains(&today_str) {
                    return true;
                }
            }
            // Tasks currently in progress
            if matches!(t.frontmatter.status, TaskStatus::InProgress) {
                return true;
            }
            false
        })
        .collect();

    // 5. No activity
    if daily.is_empty() {
        println!("No activity today.");
        return Ok(());
    }

    // 6. Calculate totals
    let total_duration: u64 = daily.iter().map(|t| t.frontmatter.duration).sum();
    let tasks_done: usize = daily
        .iter()
        .filter(|t| t.frontmatter.done_date == Some(today))
        .count();
    let total_pomodoros: u64 = daily.iter().map(|t| t.frontmatter.pomodoro_count).sum();

    // 7. Header
    println!(
        "Today: {} tracked, {} task{} done, {} pomodoro{}",
        format_duration(total_duration),
        tasks_done,
        if tasks_done == 1 { "" } else { "s" },
        total_pomodoros,
        if total_pomodoros == 1 { "" } else { "s" },
    );

    // 8. List each task
    for task in &daily {
        let icon = match task.frontmatter.status {
            TaskStatus::Done => "✓",
            TaskStatus::InProgress => "▶",
            TaskStatus::Todo => "○",
        };
        let title = &task.frontmatter.title;
        let duration = format_duration(task.frontmatter.duration);
        let issue = match task.frontmatter.issue {
            Some(n) => format!("  (#{})", n),
            None => String::new(),
        };
        println!("{} {:<24} {:<8}{}", icon, title, duration, issue);
    }

    Ok(())
}

fn format_duration(minutes: u64) -> String {
    if minutes == 0 {
        return String::new();
    }
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours > 0 && mins > 0 {
        format!("{}h{}m", hours, mins)
    } else if hours > 0 {
        format!("{}h", hours)
    } else {
        format!("{}m", mins)
    }
}
