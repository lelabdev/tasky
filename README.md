# Tasky

Tasky is a command-line task tracker for developers, integrating with Obsidian and GitHub.

## Version 1.0.0

## Features

- **Task Creation**: Create tasks as Markdown files in your Obsidian vault.
- **Obsidian Integration**: Tasks are organized by project in your vault.
- **GitHub Integration**: Optionally create GitHub issues for tasks.
- **YAML Frontmatter**: Each task file includes metadata (title, status, pomodoro count, duration, dates, issue number).
- **Task Listing**: List all tasks or filter by project.
- **Mark Task Done**: Mark a task as complete, update status and completion date.
- **Finish Task**: Push branch, create and merge a GitHub PR, mark the task as done.
- **Start Development**: Start work on a GitHub issue, update task status.
- **Pomodoro Timer**: Visual terminal animation, configurable duration.
- **Pomodoro Tracking**: Each finished Pomodoro increments the task’s pomodoro count and total duration (in minutes).
- **Smart Configuration**: Prompts for Obsidian vault path and Pomodoro settings on first run.
- **Unique Filenames**: Prevents overwriting existing tasks.

## Commands

- `tasky --help` or `tasky -h`
  Show all available commands.

- `tasky list [-all | <project_name>]`
  List tasks for all projects or a specific project. (Alias: `tasky view`)

- `tasky new ["<title>"] ["<description>"]`
  Create a new task. If `<title>` is omitted, you will be prompted to enter it. You will be asked if you want to create a GitHub issue. After creation, proposes to start the task (which will also call `gh issue develop` if a GitHub issue was created) and a Pomodoro (default: yes).

- `tasky done "<task_title>"`
  Mark a task as done.

- `tasky finish`
  Push branch, create and merge a PR, and mark the task as done.

- `tasky start <issue_number>`
  Start development on a GitHub issue, update task status.

- `tasky pomodoro --configure` or `tasky po -c`
  Configure Pomodoro timer settings.

- `tasky pomodoro start`
  Start a Pomodoro timer. At the end, increments the pomodoro count and duration (in minutes) for the active task (based on the current branch).

- `tasky link`
  Create a symbolic link named `tasky` in the current directory, pointing to the project's task directory. You will be asked if you want to add `tasky/` to your project's `.gitignore` (default: yes).

## YAML Frontmatter Example

```yaml
---
title: "My Task"
status: "in progress"
created_date: "2024-06-10 10:00:00"
done_date: ""
start_date: "2024-06-10 10:05:00"
pomodoro_count: 2
duration: 50
issue: 123
---
```

- `pomodoro_count`: Number of Pomodoros completed for this task.
- `duration`: Total time spent (in minutes).

## Configuration

The config file is stored in `~/.config/tasky/config.toml`:

```toml
vault_path = "/home/user/Documents/Obsidian/"
pomodoro_duration = 25
short_break_duration = 5
long_break_duration = 15
long_break_interval = 4
```
