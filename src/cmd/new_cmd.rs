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

    // 4. Resolve description: from args or interactive prompt
    let description = resolve_description(&args)?;

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

    // 6. Build body
    let body = if description.is_empty() {
        "\n".to_string()
    } else {
        format!("{description}\n")
    };

    // 7. Determine file path
    let tasky_dir = get_tasky_dir(&config.vault.path, &project);
    let slug = slugify_title(&title);
    let filename = resolve_filename(&tasky_dir, &slug);
    let file_path = tasky_dir.join(&filename);

    // 8. Build and write task
    let task = Task {
        frontmatter,
        body,
        file_path: file_path.clone(),
        project: project.clone(),
    };

    write_task(&task).context("failed to write task file")?;

    // 9. Print confirmation
    println!("Created task: {}", task.frontmatter.title);
    println!("  Project: {project}");
    println!("  File:    {}", file_path.display());

    // 10. Offer to create GitHub issue
    if args.issue.is_none() && has_github_remote() {
        if prompt_yes_no("Create a GitHub issue? [Y/n]")? {
            match create_github_issue(&task.frontmatter.title, &description) {
                Ok((issue_num, issue_url)) => {
                    // Update task frontmatter with issue number
                    let mut updated_task = task;
                    updated_task.frontmatter.issue = Some(issue_num);
                    write_task(&updated_task).context("failed to update task file with issue number")?;
                    println!("Created issue #{issue_num}: {issue_url}");
                }
                Err(e) => {
                    eprintln!("Warning: failed to create GitHub issue: {e}");
                }
            }
        }
    }

    Ok(())
}

/// Resolve description: from args flag, or prompt interactively when title was also prompted.
fn resolve_description(args: &NewArgs) -> Result<String> {
    // If description was provided via flag, use it
    if let Some(ref desc) = args.description {
        return Ok(desc.clone());
    }

    // Only prompt for description when title was also not provided (interactive mode)
    if args.title.is_none() && args.issue.is_none() {
        return prompt_description();
    }

    Ok(String::new())
}

/// Prompt the user for a task title via stdin.
fn prompt_title() -> Result<String> {
    print!("Task title: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Prompt the user for an optional description via stdin.
fn prompt_description() -> Result<String> {
    print!("Description (optional): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Prompt yes/no. Defaults to yes on empty input.
fn prompt_yes_no(question: &str) -> Result<bool> {
    print!("{question} ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();
    Ok(answer.is_empty() || answer == "y" || answer == "yes")
}

/// Check if the current directory is in a git repo with a GitHub remote.
fn has_github_remote() -> bool {
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let url = String::from_utf8_lossy(&out.stdout);
            url.contains("github.com")
        }
        _ => false,
    }
}

/// Create a GitHub issue using `gh issue create`.
/// Returns (issue_number, issue_url) on success.
fn create_github_issue(title: &str, body: &str) -> Result<(u64, String)> {
    let mut cmd = std::process::Command::new("gh");
    cmd.args(["issue", "create", "--title", title]);

    if !body.is_empty() {
        cmd.arg("--body").arg(body);
    }

    let output = cmd
        .output()
        .context("failed to run `gh issue create`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh issue create failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // gh issue create prints the issue URL as the last line of output
    let issue_url = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .last()
        .context("gh issue create returned no URL")?
        .trim()
        .to_string();

    // Parse issue number from URL like https://github.com/org/repo/issues/42
    let issue_num = parse_issue_number_from_url(&issue_url)
        .context(format!("could not parse issue number from URL: {issue_url}"))?;

    Ok((issue_num, issue_url))
}

/// Extract the issue number from a GitHub issue URL.
/// e.g. "https://github.com/org/repo/issues/42" → 42
fn parse_issue_number_from_url(url: &str) -> Option<u64> {
    let url = url.trim().trim_end_matches('/');
    let parts: Vec<&str> = url.split('/').collect();
    // Look for "issues" segment, take the next one
    let idx = parts.iter().position(|&p| p == "issues")?;
    parts.get(idx + 1)?.parse().ok()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_issue_number_from_url() {
        assert_eq!(
            parse_issue_number_from_url("https://github.com/lelabdev/tasky/issues/42"),
            Some(42)
        );
        assert_eq!(
            parse_issue_number_from_url("https://github.com/org/repo/issues/123"),
            Some(123)
        );
        assert_eq!(
            parse_issue_number_from_url("https://github.com/org/repo/issues/123/"),
            Some(123)
        );
        assert_eq!(parse_issue_number_from_url("https://example.com/nope"), None);
    }
}
