use anyhow::{Context, Result};
use chrono::Local;

use crate::config::Config;
use crate::storage::{find_task, list_tasks, slugify_title, write_task};
use crate::task::{Frontmatter, Task, TaskStatus};
use crate::utils::{detect_project, get_tasky_dir};

/// GitHub issue from `gh issue list`
#[derive(serde::Deserialize)]
struct GhIssue {
    number: u64,
    title: String,
}

pub fn run() -> Result<()> {
    let config = Config::ensure_loaded()?;
    let project = detect_project().context("could not detect project name")?;

    // Fetch open issues from GitHub
    let output = std::process::Command::new("gh")
        .args([
            "issue",
            "list",
            "--state",
            "open",
            "--json",
            "number,title",
            "--limit",
            "100",
        ])
        .output()
        .context("failed to run `gh issue list`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh issue list failed: {stderr}");
    }

    let issues: Vec<GhIssue> = serde_json::from_slice(&output.stdout)?;
    if issues.is_empty() {
        println!("No open issues found.");
        return Ok(());
    }

    // Get existing tasks for this project
    let tasks = list_tasks(&config.vault.path, Some(&project))?;

    // Find which issue numbers already have tasks
    let existing_issues: std::collections::HashSet<u64> = tasks
        .iter()
        .filter_map(|t| t.frontmatter.issue)
        .collect();

    // Create tasks for missing issues
    let tasky_dir = get_tasky_dir(&config.vault.path, &project);
    let today = Local::now().date_naive();
    let mut created = 0;
    let mut skipped = 0;

    for issue in &issues {
        if existing_issues.contains(&issue.number) {
            skipped += 1;
            continue;
        }

        let frontmatter = Frontmatter {
            title: issue.title.clone(),
            status: TaskStatus::Todo,
            created: today,
            issue: Some(issue.number),
            estimate: None,
            duration: 0,
            pomodoro_count: 0,
            start_date: None,
            done_date: None,
            tags: None,
        };

        let slug = slugify_title(&issue.title);
        let filename = resolve_filename(&tasky_dir, &slug);
        let file_path = tasky_dir.join(&filename);

        let task = Task {
            frontmatter,
            body: "\n".to_string(),
            file_path,
            project: project.clone(),
        };

        write_task(&task)?;
        println!("  #{} {}", issue.number, issue.title);
        created += 1;
    }

    println!();
    if created > 0 {
        println!("Pulled {created} new task{} to {project}", if created > 1 { "s" } else { "" });
    }
    if skipped > 0 {
        println!("Skipped {skipped} (already in vault)");
    }
    if created == 0 && skipped > 0 {
        println!("All issues already tracked.");
    }

    Ok(())
}

/// Resolve filename, appending `-2`, `-3`, etc. if a file already exists.
fn resolve_filename(dir: &std::path::Path, slug: &str) -> String {
    let base = format!("{slug}.md");
    if !dir.join(&base).exists() {
        return base;
    }
    let mut counter = 2u32;
    loop {
        let candidate = format!("{slug}-{counter}.md");
        if !dir.join(&candidate).exists() {
            return candidate;
        }
        counter += 1;
    }
}
