use std::collections::BTreeMap;

use anyhow::Result;
use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};

use crate::config::Config;
use crate::storage::list_tasks;
use crate::task::TaskStatus;

pub fn run() -> Result<()> {
    // 1. Load config
    let config = Config::ensure_loaded()?;

    // 2. Get ALL tasks across ALL projects
    let tasks = list_tasks(&config.vault.path, None)?;

    // 3. Calculate date range: 7 days ago → today
    let today = Local::now().date_naive();
    let week_ago = today - Duration::days(7);

    // 4. Filter tasks relevant to this week
    let weekly: Vec<_> = tasks
        .into_iter()
        .filter(|t| {
            // Tasks where done_date is within the last 7 days
            if let Some(done) = t.frontmatter.done_date {
                if done >= week_ago && done <= today {
                    return true;
                }
            }
            // Tasks whose start_date string contains a date within the last 7 days
            if let Some(ref start) = t.frontmatter.start_date {
                if let Some(date) = parse_date_from_start(start) {
                    if date >= week_ago && date <= today {
                        return true;
                    }
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
    if weekly.is_empty() {
        println!("No activity this week.");
        return Ok(());
    }

    // 6. Calculate totals
    let total_duration: u64 = weekly.iter().map(|t| t.frontmatter.duration).sum();
    let tasks_done: usize = weekly
        .iter()
        .filter(|t| {
            t.frontmatter
                .done_date
                .map_or(false, |d| d >= week_ago && d <= today)
        })
        .count();
    let total_pomodoros: u64 = weekly.iter().map(|t| t.frontmatter.pomodoro_count).sum();

    // 7. Header
    println!(
        "This week: {} tracked, {} task{} done, {} pomodoro{}",
        format_duration(total_duration),
        tasks_done,
        if tasks_done == 1 { "" } else { "s" },
        total_pomodoros,
        if total_pomodoros == 1 { "" } else { "s" },
    );

    // 8. Determine the day for each task and group by day
    let mut by_day: BTreeMap<NaiveDate, Vec<_>> = BTreeMap::new();
    for task in &weekly {
        let day = determine_day(task, week_ago, today);
        by_day.entry(day).or_default().push(task);
    }

    // 9. Display each day, newest first
    for (date, tasks) in by_day.iter().rev() {
        let day_name = weekday_short(date.weekday());
        println!("── {} {} ──", day_name, date.format("%Y-%m-%d"));

        for task in tasks {
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
    }

    // 10. Summary by project if multiple projects
    let mut projects: BTreeMap<&str, (u64, usize)> = BTreeMap::new();
    for task in &weekly {
        let entry = projects.entry(&task.project).or_default();
        entry.0 += task.frontmatter.duration;
        entry.1 += 1;
    }
    if projects.len() > 1 {
        println!();
        println!("By project:");
        for (proj, (dur, count)) in &projects {
            println!(
                "  {:<16} {} ({} task{})",
                proj,
                format_duration(*dur),
                count,
                if *count == 1 { "" } else { "s" }
            );
        }
    }

    Ok(())
}

/// Determine which day a task belongs to for grouping.
/// Uses done_date if it falls within range, then start_date, then today (for in-progress).
fn determine_day(task: &crate::task::Task, week_ago: NaiveDate, today: NaiveDate) -> NaiveDate {
    // Prefer done_date if within range
    if let Some(done) = task.frontmatter.done_date {
        if done >= week_ago && done <= today {
            return done;
        }
    }
    // Then start_date if within range
    if let Some(ref start) = task.frontmatter.start_date {
        if let Some(date) = parse_date_from_start(start) {
            if date >= week_ago && date <= today {
                return date;
            }
        }
    }
    // Fallback: today for in-progress tasks
    today
}

/// Try to extract a date from the start_date string.
/// start_date may be a plain "YYYY-MM-DD" or contain additional text.
fn parse_date_from_start(start: &str) -> Option<NaiveDate> {
    // Try parsing the whole string as a date
    if let Ok(d) = NaiveDate::parse_from_str(start.trim(), "%Y-%m-%d") {
        return Some(d);
    }
    // Try to find a date pattern in the string
    for (i, c) in start.char_indices() {
        if c.is_ascii_digit() && i + 10 <= start.len() {
            let slice = &start[i..i + 10];
            if let Ok(d) = NaiveDate::parse_from_str(slice, "%Y-%m-%d") {
                return Some(d);
            }
        }
    }
    None
}

fn weekday_short(w: Weekday) -> &'static str {
    match w {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
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
