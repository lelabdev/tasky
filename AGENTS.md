# AGENTS.md — Tasky

## Project

- **Name:** tasky
- **Version:** 0.2.0
- **Repo:** [lelabdev/tasky](https://github.com/lelabdev/tasky)
- **Description:** CLI task manager for developers with Obsidian vault and GitHub Issue integration
- **Language:** Rust (edition 2024)

## Stack

- **CLI framework:** clap (derive)
- **Serialization:** serde, serde_yaml, toml
- **Error handling:** anyhow, thiserror
- **Date/time:** chrono
- **Terminal:** crossterm, indicatif, ratatui
- **Config:** dirs (platform config directory)

## Commands

```bash
# Build
cargo build
cargo build --release

# Run
cargo run -- <command>

# Test
cargo test

# Lint
cargo clippy
```

## CLI Commands & Aliases

| Command | Alias | Description |
|---------|-------|-------------|
| `tasky init` | — | Config setup (vault path) |
| `tasky new` | `n` | Create a task |
| `tasky list` | `l`, `ls` | List tasks |
| `tasky start` | `s` | Set in-progress + time tracking |
| `tasky stop` | — | Pause tracking |
| `tasky done` | `d` | Mark done |
| `tasky finish` | `f` | Push + PR + merge + done |
| `tasky link` | — | Create `_tasky` symlink |
| `tasky day` | — | Daily summary |
| `tasky week` | — | Weekly summary (all projects) |
| `tasky pull` | `p` | Fetch open GitHub issues as tasks |
| `tasky report` | — | Time tracking report |
| `tasky tui` | — | Interactive TUI (ratatui) |
| `tasky pomodoro start` | `po start` | Pomodoro timer |
| `tasky pomodoro stop` | `po stop` | Stop pomodoro |
| `tasky pomodoro status` | `po status` | Pomodoro settings |
| `tasky pomodoro configure` | `po configure` | Edit pomodoro durations |

## Configuration

- **Config file:** `~/.config/tasky/config.toml`
- **Task storage:** `~/obsidian/1_Projects/<project>/<slug>.md`
- Tasks are Markdown files with YAML frontmatter

## Architecture

```
src/
  main.rs            — Entry point, clap routing
  cmd/
    mod.rs            — CLI definitions (clap derive)
    init.rs           — tasky init
    new_cmd.rs        — tasky new
    list_cmd.rs       — tasky list
    start_cmd.rs      — tasky start
    stop_cmd.rs       — tasky stop
    done_cmd.rs       — tasky done
    finish_cmd.rs     — tasky finish
    link_cmd.rs       — tasky link
    day_cmd.rs        — tasky day
    week_cmd.rs       — tasky week
    pull_cmd.rs       — tasky pull
    report_cmd.rs     — tasky report
    pomodoro_cmd.rs   — tasky pomodoro
  tui/
    mod.rs            — TUI entry (terminal setup/teardown)
    app.rs            — App state machine (project picker, list, detail, settings)
    ui.rs             — ratatui rendering
    gh.rs             — GitHub issue fetching via gh CLI
  config.rs          — TOML config load/save
  storage.rs         — Read/write frontmatter, list_tasks, find_task, slugify
  task.rs            — Task, Frontmatter, TaskStatus data models
  utils.rs           — detect_project, get_current_branch, extract_issue_from_branch, is_git_repository
  pomodoro.rs        — Pomodoro timer with indicatif progress bar
```

### Data Flow

1. **Config** loaded from `~/.config/tasky/config.toml` (vault path, pomodoro settings)
2. **Project** auto-detected from git context (remote → root dir → cwd)
3. **Outside git** → prompts for project selection from vault projects
4. **Project auto-creation** → when using `-p <name>` or interactive picker, creates the vault directory if it doesn't exist
5. **Tasks** stored as Markdown + YAML frontmatter in `~/obsidian/1_Projects/<project>/`
6. **Task lookup** by issue number (exact) or title (case-insensitive substring)

### Key Types

- `Config` — vault path, pomodoro settings, sound paths
- `Frontmatter` — title, status, created, issue, estimate, duration, pomodoro_count, start_date, done_date, tags
- `Task` — Frontmatter + body + file_path + project
- `TaskStatus` — Todo, InProgress, Done

## Conventions

- **Language:** English for all code, comments, docs, and commit messages
- **Commits:** Conventional commits (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`)
- **Branches:** `type/issue-number-description` (e.g. `feat/42-login-page`)
- **Error handling:** `anyhow::Result` for commands, `thiserror` for domain errors
- **Naming:** snake_case for functions/variables, CamelCase for types

## Network

- No server — CLI tool only
- No ports exposed

## Legacy

- Original Go codebase archived in `_archive_go/` (reference only, not maintained)
