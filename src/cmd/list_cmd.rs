use anyhow::Result;

use crate::config::Config;
use crate::storage::list_tasks;
use crate::task::TaskStatus;
use crate::utils::detect_project;

use super::ListArgs;

pub fn run(args: ListArgs) -> Result<()> {
    // 1. Load config
    let config = Config::ensure_loaded()?;

    // 2. Detect project: use args.project if provided, else detect from cwd
    let project = match &args.project {
        Some(p) => p.clone(),
        None => detect_project()?,
    };

    // 3. Get tasks
    let tasks = list_tasks(&config.vault.path, Some(&project))?;

    // 4. Filter
    let filtered: Vec<_> = if args.all {
        tasks.into_iter().collect()
    } else if args.done {
        tasks
            .into_iter()
            .filter(|t| matches!(t.frontmatter.status, TaskStatus::Done))
            .collect()
    } else {
        // Default: show only Todo and InProgress
        tasks
            .into_iter()
            .filter(|t| !matches!(t.frontmatter.status, TaskStatus::Done))
            .collect()
    };

    // 5. Sort
    let mut sorted = filtered;
    match args.sort.as_deref() {
        Some("duration") => {
            sorted.sort_by(|a, b| b.frontmatter.duration.cmp(&a.frontmatter.duration));
        }
        Some("created") | None => {
            // Already sorted by created date (newest first) from list_tasks
        }
        Some(other) => {
            anyhow::bail!("unknown sort field: '{}'. Use 'duration' or 'created'.", other);
        }
    }

    // 6. Display
    if sorted.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    // Count totals for summary (across ALL tasks, not just filtered)
    let all_tasks = list_tasks(&config.vault.path, Some(&project))?;
    let active_count = all_tasks
        .iter()
        .filter(|t| !matches!(t.frontmatter.status, TaskStatus::Done))
        .count();
    let done_count = all_tasks
        .iter()
        .filter(|t| matches!(t.frontmatter.status, TaskStatus::Done))
        .count();
    let total = all_tasks.len();

    // Print table header
    println!(
        "{:<4} {:<30} {:<10} {:<8} {}",
        "", "Title", "Duration", "Issue", "Project"
    );
    println!("{}", "-".repeat(72));

    for task in &sorted {
        let icon = status_icon(&task.frontmatter.status);
        let title = truncate(&task.frontmatter.title, 28);
        let duration = format_duration(task.frontmatter.duration);
        let issue = match task.frontmatter.issue {
            Some(n) => format!("#{}", n),
            None => String::new(),
        };
        let project = &task.project;

        println!(
            "{:<4} {:<30} {:<10} {:<8} {}",
            icon, title, duration, issue, project
        );
    }

    println!();
    println!(
        "{} tasks ({} active, {} done)",
        total, active_count, done_count
    );

    Ok(())
}

fn status_icon(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "○",
        TaskStatus::InProgress => "▶",
        TaskStatus::Done => "✓",
    }
}

fn format_duration(minutes: u64) -> String {
    if minutes == 0 {
        return String::new();
    }
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours > 0 && mins > 0 {
        format!("{}h {}m", hours, mins)
    } else if hours > 0 {
        format!("{}h", hours)
    } else {
        format!("{}m", mins)
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut truncated: String = s.chars().take(max_len - 1).collect();
        truncated.push('…');
        truncated
    }
}
