use std::io::{self, Write};

use anyhow::{Context, Result};
use chrono::Local;

use crate::config::Config;
use crate::storage::{slugify_title, write_task};
use crate::task::{Frontmatter, Task, TaskStatus};
use crate::utils::{detect_project, get_tasky_dir};

use super::NewArgs;

pub fn run(args: NewArgs) -> Result<()> {
    // 1. Load config
    let config = Config::ensure_loaded()?;

    // 2. Detect project name
    let project = match &args.project {
        Some(p) => p.clone(),
        None => detect_project().context("could not detect project name")?,
    };

    // 3. Resolve title
    let mut title = match &args.title {
        Some(t) => t.clone(),
        None => {
            // 4. If --issue provided but no title, fetch from GitHub
            if let Some(issue_num) = args.issue {
                fetch_issue_title(issue_num).unwrap_or_else(|e| {
                    eprintln!("Warning: could not fetch issue title: {e}");
                    String::new()
                })
            } else {
                String::new()
            }
        }
    };

    // If still no title, prompt user via stdin
    if title.is_empty() {
        title = prompt_title()?;
    }

    if title.trim().is_empty() {
        anyhow::bail!("task title cannot be empty");
    }

    // 5. Build Frontmatter
    let today = Local::now().date_naive();
    let frontmatter = Frontmatter {
        title: title.trim().to_string(),
        status: TaskStatus::Todo,
        created: today,
        issue: args.issue,
        estimate: args.estimate,
        duration: 0,
        pomodoro_count: 0,
        start_date: None,
        done_date: None,
        tags: None,
    };

    // 7. Build body
    let body = match &args.description {
        Some(desc) => format!("{desc}\n"),
        None => "\n".to_string(),
    };

    // 8. Determine file path
    let tasky_dir = get_tasky_dir(&config.vault.path, &project);
    let slug = slugify_title(&title);
    let filename = resolve_filename(&tasky_dir, &slug);
    let file_path = tasky_dir.join(&filename);

    // 9. Build and write task
    let task = Task {
        frontmatter,
        body,
        file_path: file_path.clone(),
        project: project.clone(),
    };

    write_task(&task).context("failed to write task file")?;

    // 10. Print confirmation
    println!("Created task: {}", task.frontmatter.title);
    println!("  Project: {project}");
    println!("  File:    {}", file_path.display());

    Ok(())
}

/// Fetch an issue title from GitHub CLI.
fn fetch_issue_title(issue_number: u64) -> Result<String> {
    let output = std::process::Command::new("gh")
        .args([
            "issue",
            "view",
            &issue_number.to_string(),
            "--json",
            "title",
            "-q",
            ".title",
        ])
        .output()
        .context("failed to run `gh issue view`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh issue view failed: {stderr}");
    }

    let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if title.is_empty() {
        anyhow::bail!("gh returned empty title for issue #{issue_number}");
    }

    Ok(title)
}

/// Prompt the user for a task title via stdin.
fn prompt_title() -> Result<String> {
    print!("Task title: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
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
