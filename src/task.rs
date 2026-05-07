use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Task status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Todo,
    #[serde(rename = "in progress")]
    InProgress,
    Done,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "todo"),
            TaskStatus::InProgress => write!(f, "in progress"),
            TaskStatus::Done => write!(f, "done"),
        }
    }
}

/// YAML frontmatter for a task file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub status: TaskStatus,
    pub created: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimate: Option<u64>,
    #[serde(default)]
    pub duration: u64,
    #[serde(default)]
    pub pomodoro_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// A complete task (frontmatter + body)
#[derive(Debug, Clone)]
pub struct Task {
    pub frontmatter: Frontmatter,
    pub body: String,
    pub file_path: std::path::PathBuf,
    pub project: String,
}
