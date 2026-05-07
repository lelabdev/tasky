pub mod day_cmd;
pub mod done_cmd;
pub mod finish_cmd;
pub mod init;
pub mod link_cmd;
pub mod list_cmd;
pub mod new_cmd;
pub mod pomodoro_cmd;
pub mod start_cmd;
pub mod stop_cmd;
pub mod week_cmd;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tasky")]
#[command(about = "CLI task manager with Obsidian and GitHub integration")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize tasky configuration
    Init,

    /// Create a new task
    New(NewArgs),

    /// List tasks
    List(ListArgs),

    /// Start a task (set in-progress + begin tracking)
    Start(TaskIdentifier),

    /// Stop time tracking on active task
    Stop,

    /// Mark a task as done
    Done(TaskIdentifier),

    /// Push branch, create PR, merge, mark task done
    Finish,

    /// Create _tasky symlink in project directory
    Link(LinkArgs),

    /// Show daily summary
    Day,

    /// Show weekly summary
    Week,

    /// Pomodoro timer
    Pomodoro(PomodoroArgs),
}

#[derive(Args)]
pub struct NewArgs {
    /// Task title
    pub title: Option<String>,

    /// Task description (body content)
    #[arg(short, long)]
    pub description: Option<String>,

    /// Link to a GitHub issue number
    #[arg(short, long)]
    pub issue: Option<u64>,

    /// Time estimate in minutes
    #[arg(short, long)]
    pub estimate: Option<u64>,

    /// Target a specific project (default: auto-detect)
    #[arg(short, long)]
    pub project: Option<String>,
}

#[derive(Args)]
pub struct ListArgs {
    /// Show all tasks including done
    #[arg(long)]
    pub all: bool,

    /// Show only done tasks
    #[arg(long)]
    pub done: bool,

    /// Filter by project
    #[arg(long)]
    pub project: Option<String>,

    /// Sort field (duration, created, status)
    #[arg(long)]
    pub sort: Option<String>,
}

#[derive(Args)]
pub struct TaskIdentifier {
    /// Task title (fuzzy match) or issue number
    pub query: String,
}

#[derive(Args)]
pub struct LinkArgs {
    /// Target a specific project
    #[arg(long)]
    pub project: Option<String>,
}

#[derive(Args)]
pub struct PomodoroArgs {
    #[command(subcommand)]
    pub command: PomodoroCommands,
}

#[derive(Subcommand)]
pub enum PomodoroCommands {
    /// Start a pomodoro timer
    Start {
        /// Target a specific task
        #[arg(long)]
        task: Option<String>,
    },

    /// Stop the current pomodoro
    Stop,

    /// Show current pomodoro status
    Status,

    /// Configure pomodoro settings
    Configure,
}
