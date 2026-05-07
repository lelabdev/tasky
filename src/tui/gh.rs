use anyhow::Result;
use serde::Deserialize;

/// GitHub issue returned by `gh issue list --json`
#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub labels: Vec<Label>,
    #[serde(default)]
    pub body: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    pub name: String,
}

/// Fetch open issues for the current repo via `gh`
pub fn fetch_issues() -> Result<Vec<Issue>> {
    let output = std::process::Command::new("gh")
        .args([
            "issue",
            "list",
            "--state",
            "open",
            "--json",
            "number,title,labels",
            "--limit",
            "100",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh issue list failed: {stderr}");
    }

    let issues: Vec<Issue> = serde_json::from_slice(&output.stdout)?;
    Ok(issues)
}

/// Fetch the full body of an issue
pub fn fetch_issue_body(number: u64) -> Result<String> {
    let output = std::process::Command::new("gh")
        .args([
            "issue",
            "view",
            &number.to_string(),
            "--json",
            "body",
            "-q",
            ".body",
        ])
        .output()?;

    if !output.status.success() {
        return Ok(String::from("(could not load description)"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Create a git branch for an issue: `type/issue-number-slug`
pub fn create_branch(issue_number: u64, title: &str) -> Result<String> {
    let slug = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    // Detect issue type from labels
    let branch_type = "feat"; // default, could be inferred from labels
    let branch_name = format!("{branch_type}/{issue_number}-{slug}");

    let output = std::process::Command::new("git")
        .args(["checkout", "-b", &branch_name])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git checkout failed: {stderr}");
    }

    Ok(branch_name)
}
