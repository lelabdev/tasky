use anyhow::{Context, Result};
use chrono::Local;
use std::process::Command;

use crate::config::Config;
use crate::storage::{find_task, list_tasks, write_task};
use crate::task::TaskStatus;
use crate::utils::{detect_project, extract_issue_from_branch, get_current_branch};

pub fn run() -> Result<()> {
    // 1. Load config
    let config = Config::ensure_loaded()?;

    // 2. Detect project
    let project = detect_project()?;

    // 3. Get current branch
    let branch = get_current_branch()?;

    // 4. Bail if on main or prod
    if branch == "main" || branch == "prod" {
        anyhow::bail!("Cannot finish from main/prod branch. Switch to a feature branch.");
    }

    // 5. Extract issue number from branch
    let issue_num = extract_issue_from_branch(&branch);

    // 6. Get all tasks, find matching one
    let tasks = list_tasks(&config.vault.path, Some(&project))?;

    let task = if let Some(num) = issue_num {
        // Try to find by issue number
        find_task(&tasks, &num.to_string())
    } else {
        // No issue number: try to find by branch slug in title
        let slug = branch
            .split('/')
            .last()
            .unwrap_or(&branch)
            .to_string();
        // Remove leading number prefix if any (e.g. "42-login-page" -> "login-page")
        let slug = slug
            .split_once('-')
            .map(|(_, rest)| rest.to_string())
            .unwrap_or(slug);
        tasks.iter().find(|t| {
            t.frontmatter
                .title
                .to_lowercase()
                .contains(&slug.replace('-', " ").to_lowercase())
        })
    };

    let task = task.ok_or_else(|| anyhow::anyhow!("No task found for this branch"))?;

    // Clone so we can mutate
    let mut task = task.clone();

    println!("Pushing branch {}...", branch);

    // 8. Git push
    let push_status = Command::new("git")
        .args(["push", "origin", &branch])
        .status()
        .context("failed to run git push")?;

    if !push_status.success() {
        anyhow::bail!("git push failed");
    }

    println!("Creating PR...");

    // 9. Create PR
    let pr_output = Command::new("gh")
        .args(["pr", "create", "--fill", "--base", "main"])
        .output()
        .context("failed to run gh pr create")?;

    if !pr_output.status.success() {
        let stderr = String::from_utf8_lossy(&pr_output.stderr);
        anyhow::bail!("gh pr create failed: {}", stderr.trim());
    }

    let pr_stdout = String::from_utf8_lossy(&pr_output.stdout);
    let pr_url = pr_stdout.trim();

    println!("PR created: {}", pr_url);

    // 10. Merge PR
    println!("Merging PR...");

    let merge_status = Command::new("gh")
        .args(["pr", "merge", pr_url, "--squash", "--delete-branch"])
        .status()
        .context("failed to run gh pr merge")?;

    if !merge_status.success() {
        anyhow::bail!("gh pr merge failed");
    }

    // 11. Mark task done
    task.frontmatter.status = TaskStatus::Done;
    task.frontmatter.done_date = Some(Local::now().date_naive());

    write_task(&task)?;

    // 12. Print summary
    println!();
    println!("Finished: {}", task.frontmatter.title);

    if task.frontmatter.duration > 0 {
        let hours = task.frontmatter.duration / 60;
        let mins = task.frontmatter.duration % 60;
        if hours > 0 {
            println!("Duration: {}h {}m", hours, mins);
        } else {
            println!("Duration: {}m", mins);
        }
    }

    if task.frontmatter.pomodoro_count > 0 {
        println!("Pomodoros: {}", task.frontmatter.pomodoro_count);
    }

    println!("PR merged and branch deleted ✓");

    Ok(())
}
