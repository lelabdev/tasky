use crate::cmd::ReportArgs;
use crate::config::Config;
use crate::storage::list_tasks;
use crate::task::TaskStatus;
use anyhow::Result;

/// Format a duration in minutes to a human-readable string.
/// If >= 60min show "Xh Ym", if < 60min show "Ym".
fn format_duration(minutes: u64) -> String {
    if minutes >= 60 {
        let h = minutes / 60;
        let m = minutes % 60;
        if m == 0 {
            format!("{}h", h)
        } else {
            format!("{}h {}m", h, m)
        }
    } else {
        format!("{}m", minutes)
    }
}

pub fn run(args: ReportArgs) -> Result<()> {
    let config = Config::ensure_loaded()?;
    let vault_path = &config.vault.path;

    let mut tasks = list_tasks(vault_path, args.project.as_deref())?;

    // Filter: --done shows only Done tasks, default shows all
    if args.done {
        tasks.retain(|t| matches!(t.frontmatter.status, TaskStatus::Done));
    }

    if tasks.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    // --- Calculate totals ---
    let total_duration: u64 = tasks.iter().map(|t| t.frontmatter.duration).sum();
    let total_pomodoros: u64 = tasks.iter().map(|t| t.frontmatter.pomodoro_count).sum();

    let count_todo = tasks
        .iter()
        .filter(|t| matches!(t.frontmatter.status, TaskStatus::Todo))
        .count();
    let count_in_progress = tasks
        .iter()
        .filter(|t| matches!(t.frontmatter.status, TaskStatus::InProgress))
        .count();
    let count_done = tasks
        .iter()
        .filter(|t| matches!(t.frontmatter.status, TaskStatus::Done))
        .count();

    // Average duration per done task
    let done_durations: Vec<u64> = tasks
        .iter()
        .filter(|t| matches!(t.frontmatter.status, TaskStatus::Done))
        .map(|t| t.frontmatter.duration)
        .collect();
    let avg_done = if done_durations.is_empty() {
        0
    } else {
        done_durations.iter().sum::<u64>() / done_durations.len() as u64
    };

    // --- Header ---
    let project_label = args.project.as_deref().unwrap_or("all projects");
    println!("Report: {}\n", project_label);

    // --- Summary line ---
    println!(
        "{} tasks ({} todo, {} in progress, {} done) — {}, {} pomodoros",
        tasks.len(),
        count_todo,
        count_in_progress,
        count_done,
        format_duration(total_duration),
        total_pomodoros,
    );

    // --- Average ---
    if !done_durations.is_empty() {
        println!("Avg per done task: {}", format_duration(avg_done));
    }

    // --- Estimate vs actual summary ---
    let estimated_tasks: Vec<_> = tasks
        .iter()
        .filter(|t| t.frontmatter.estimate.is_some() && t.frontmatter.duration > 0)
        .collect();

    if !estimated_tasks.is_empty() {
        let total_estimated: u64 = estimated_tasks
            .iter()
            .map(|t| t.frontmatter.estimate.unwrap())
            .sum();
        let total_actual: u64 = estimated_tasks.iter().map(|t| t.frontmatter.duration).sum();
        let diff = total_actual as i64 - total_estimated as i64;
        let diff_str = if diff > 0 {
            format!("+{} over", format_duration(diff as u64))
        } else if diff < 0 {
            format!("{} under", format_duration((-diff) as u64))
        } else {
            "on target".to_string()
        };
        println!(
            "Estimates: {} estimated vs {} actual ({})",
            format_duration(total_estimated),
            format_duration(total_actual),
            diff_str,
        );
    }

    println!();

    // --- Sort tasks by duration descending ---
    let mut sorted_tasks = tasks.clone();
    sorted_tasks.sort_by(|a, b| b.frontmatter.duration.cmp(&a.frontmatter.duration));

    let show_project = args.project.is_none();

    for task in &sorted_tasks {
        let icon = match task.frontmatter.status {
            TaskStatus::Todo => "○",
            TaskStatus::InProgress => "▶",
            TaskStatus::Done => "✓",
        };

        let duration_str = if task.frontmatter.duration > 0 {
            format_duration(task.frontmatter.duration)
        } else {
            "—".to_string()
        };

        // Estimate vs actual annotation
        let est_actual = match (task.frontmatter.estimate, task.frontmatter.duration > 0) {
            (Some(est), true) => {
                let actual = task.frontmatter.duration;
                let diff = actual as i64 - est as i64;
                let diff_str = if diff > 0 {
                    format!("+{}", format_duration(diff as u64))
                } else if diff < 0 {
                    format!("-{}", format_duration((-diff) as u64))
                } else {
                    "on target".to_string()
                };
                format!(
                    " [est: {}, actual: {} ({})]",
                    format_duration(est),
                    format_duration(actual),
                    diff_str,
                )
            }
            _ => String::new(),
        };

        let issue_str = task
            .frontmatter
            .issue
            .map(|i| format!(" #{}", i))
            .unwrap_or_default();

        let project_str = if show_project {
            format!(" [{}]", task.project)
        } else {
            String::new()
        };

        println!(
            "{} {} — {}{}{}{}",
            icon, task.frontmatter.title, duration_str, est_actual, issue_str, project_str,
        );
    }

    Ok(())
}
