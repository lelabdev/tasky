# Tasky

CLI task manager for developers — integrates with [Obsidian](https://obsidian.md) vaults and GitHub Issues.

Tasks are stored as Markdown files with YAML frontmatter inside your Obsidian vault, so they're always accessible from both the terminal and your notes.

## Install

```bash
git clone https://github.com/lelabdev/tasky.git
cd tasky
cargo install --path .
```

Or run directly during development:

```bash
cargo run -- <command>
```

## Quick Start

```bash
# First-time setup — configures vault path and preferences
tasky init

# Create a task
tasky new "Implement login page"
tasky new "Fix header overflow" --issue 55 --estimate 30

# List tasks
tasky list
tasky list --all
tasky list --sort duration --project myapp

# Work on a task
tasky start "login page"

# Pause tracking
tasky stop

# Mark complete
tasky done "login page"

# Full workflow: push + PR + merge + done
tasky finish

# Daily / weekly summary
tasky day
tasky week

# Time tracking report
tasky report
tasky report --project myapp --done

# Pull open GitHub issues as tasks
tasky pull

# Interactive TUI — browse issues, pick a project
tasky tui

# Pomodoro timer
tasky pomodoro start
```

## Commands

### `tasky init`

Initialize Tasky configuration. Prompts for your Obsidian vault path and saves settings to `~/.config/tasky/config.toml`.

```bash
tasky init
```

### `tasky new`

Create a new task. Opens in your default editor if no title is provided.

When run **outside a git repository**, prompts for project selection with a list of existing vault projects. Using `-p <name>` auto-creates the project directory if it doesn't exist.

```bash
tasky new "Task title"
tasky new "Fix bug" --issue 42 --estimate 60
tasky new "Refactor auth" --description "Move to JWT" --project backend
```

| Flag | Description |
|------|-------------|
| `-d`, `--description` | Task body content |
| `-i`, `--issue` | Link to a GitHub issue number |
| `-e`, `--estimate` | Time estimate in minutes |
| `-p`, `--project` | Target project (default: auto-detect from git) |

### `tasky list`

List tasks for the current project (auto-detected from git context).

```bash
tasky list                  # Active tasks (todo + in progress)
tasky list --all            # All tasks including done
tasky list --done           # Only completed tasks
tasky list --sort duration  # Sort by tracked duration
tasky list --project api    # Filter by project name
```

| Flag | Description |
|------|-------------|
| `--all` | Show all tasks including done |
| `--done` | Show only done tasks |
| `--project <name>` | Filter by project |
| `--sort <field>` | Sort by: `duration`, `created`, `status` |

### `tasky start`

Set a task to **in progress** and begin time tracking. Accepts a title substring or issue number.

```bash
tasky start "login"
tasky start 42
```

### `tasky stop`

Pause time tracking on the active task and display current tracking state.

```bash
tasky stop
```

### `tasky done`

Mark a task as done. Accepts a title substring or issue number.

```bash
tasky done "login"
tasky done 42
```

### `tasky finish`

Full completion workflow: pushes the current branch, creates a PR, merges it, and marks the linked task as done. Extracts the issue number from the branch name (e.g. `feat/42-fix-login` → issue #42).

```bash
tasky finish
```

### `tasky link`

Create a `_tasky` symlink in the current project directory pointing to the vault task folder. Useful for quick access to task files.

```bash
tasky link
tasky link --project myapp
```

| Flag | Description |
|------|-------------|
| `--project <name>` | Target project (default: auto-detect) |

### `tasky day`

Show a daily summary of tasks worked on, total tracked time, and completed tasks.

```bash
tasky day
```

### `tasky week`

Show a weekly summary across all projects — tasks completed, total time tracked, and pomodoro counts for the last 7 days.

```bash
tasky week
```

### `tasky pull`

Fetch open GitHub issues from the current repository and create local tasks for any that don't already exist. Uses `gh` CLI under the hood.

```bash
tasky pull
```

### `tasky report`

Show a time tracking report with totals, task breakdown, estimate vs actual comparison, and average duration per completed task.

```bash
tasky report                # All tasks across all projects
tasky report --project api  # Filter by project
tasky report --done         # Only completed tasks
```

| Flag | Description |
|------|-------------|
| `--project <name>` | Filter by project |
| `--done` | Show only done tasks |

### `tasky tui`

Launch an interactive terminal UI (ratatui) to browse projects, view GitHub issues and local tasks, and create branches. Starts with a **project picker** that lists all vault projects and allows creating new ones.

```bash
tasky tui
```

Keybindings: `j/k` or arrows to navigate, `Enter` to select, `s` for settings, `q`/`Esc` to go back.

### `tasky pomodoro`

Pomodoro timer with terminal progress bar and automatic task tracking.

```bash
tasky pomodoro start              # Start a 25-min pomodoro
tasky pomodoro start --task "Fix login"  # Start and link to a task
tasky pomodoro stop               # Stop the current timer
tasky pomodoro status             # Show pomodoro settings
tasky pomodoro configure          # Edit pomodoro settings
```

## Aliases

Short aliases are available for common commands:

| Alias | Command |
|-------|---------|
| `n` | `new` |
| `l`, `ls` | `list` |
| `s` | `start` |
| `d` | `done` |
| `f` | `finish` |
| `po` | `pomodoro` |
| `p` | `pull` |

## Configuration

Stored at `~/.config/tasky/config.toml`:

```toml
[vault]
path = "/home/user/obsidian"

[pomodoro]
work_duration = 25
short_break = 5
long_break = 15
long_break_interval = 4

[sounds]
start = ""
done = ""
break = ""
```

## Task File Format

Tasks are stored as Markdown files with YAML frontmatter in your vault:

```
~/obsidian/1_Projects/<project>/<slug>.md
```

Example task file:

```markdown
---
title: "Implement login page"
status: in progress
created: 2024-06-15
issue: 42
estimate: 120
duration: 45
pomodoro_count: 2
start_date: "2024-06-15T10:30:00"
tags:
  - frontend
  - auth
---

## Notes
- Use OAuth2 flow
- Add "Remember me" checkbox
```

### Task Statuses

| Status | Description |
|--------|-------------|
| `todo` | Not yet started |
| `in progress` | Currently being worked on |
| `done` | Completed |

## Project Detection

Tasky auto-detects the current project using the following priority:

1. **Git remote URL** — extracts repo name from `origin` (e.g. `https://github.com/lelabdev/tasky.git` → `tasky`)
2. **Git root directory** — uses the repository's top-level folder name
3. **Current directory** — falls back to the working directory name

When run **outside a git repository** (no git context), Tasky prompts for project selection, listing existing vault projects and allowing you to type a new name.

Using `-p <name>` or the interactive project picker **auto-creates** the project directory in the vault if it doesn't exist.

You can also override auto-detection with `--project <name>` on any command.

### Branch → Issue Extraction

Branch names following the `<type>/<number>-<slug>` pattern are parsed automatically:

```
feat/42-login-page   → issue #42
fix/99-padding-overflow → issue #99
```

## Architecture

```
src/
  main.rs            — Entry point, clap command routing
  cmd/
    mod.rs            — CLI definitions (clap derive structs & enums)
    init.rs           — tasky init (config setup wizard)
    new_cmd.rs        — tasky new (create task + file)
    list_cmd.rs       — tasky list (filters, sorting)
    start_cmd.rs      — tasky start (in-progress + time tracking)
    stop_cmd.rs       — tasky stop (pause + display tracking state)
    done_cmd.rs       — tasky done (mark complete)
    finish_cmd.rs     — tasky finish (push + PR + merge + done)
    link_cmd.rs       — tasky link (symlink _tasky directory)
    day_cmd.rs        — tasky day (daily summary)
    week_cmd.rs       — tasky week (weekly summary)
    pull_cmd.rs       — tasky pull (fetch open GitHub issues as tasks)
    report_cmd.rs     — tasky report (time tracking report)
    pomodoro_cmd.rs   — tasky pomodoro (timer + auto-track)
  tui/
    mod.rs            — TUI entry point (terminal setup/teardown)
    app.rs            — App state machine (project picker, list, detail, settings)
    ui.rs             — ratatui rendering
    gh.rs             — GitHub issue fetching via gh CLI
  config.rs          — TOML config load/save (vault path, pomodoro, sounds)
  storage.rs         — Read/write Markdown + YAML frontmatter, list_tasks, find_task, slugify
  task.rs            — Task, Frontmatter, TaskStatus data models
  utils.rs           — detect_project, get_current_branch, extract_issue_from_branch, is_git_repository
  pomodoro.rs        — Pomodoro timer with indicatif progress bar + break prompt
```

### Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` (derive) | CLI argument parsing |
| `serde` + `serde_yaml` + `toml` | Serialization (frontmatter, config) |
| `chrono` | Date/time handling |
| `crossterm` | Terminal control |
| `ratatui` | Terminal UI framework (TUI) |
| `indicatif` | Progress bars (pomodoro timer) |
| `dirs` | Platform config directories |
| `anyhow` + `thiserror` | Error handling |

## License

MIT
