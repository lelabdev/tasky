# AGENTS.md — Tasky

## Project

- **Name:** tasky
- **Repo:** [lelabdev/tasky](https://github.com/lelabdev/tasky)
- **Description:** CLI task manager for developers with Obsidian vault and GitHub Issue integration
- **Language:** Rust (edition 2024)

## Stack

- **CLI framework:** clap (derive)
- **Serialization:** serde, serde_yaml, toml
- **Error handling:** anyhow, thiserror
- **Date/time:** chrono
- **Terminal:** crossterm, indicatif
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
    pomodoro_cmd.rs   — tasky pomodoro
  config.rs          — TOML config load/save
  storage.rs         — Read/write frontmatter, list_tasks, find_task, slugify
  task.rs            — Task, Frontmatter, TaskStatus data models
  utils.rs           — detect_project, get_current_branch, extract_issue_from_branch
  pomodoro.rs        — Pomodoro timer with indicatif progress bar
```

### Data Flow

1. **Config** loaded from `~/.config/tasky/config.toml` (vault path, pomodoro settings)
2. **Project** auto-detected from git context (remote → root dir → cwd)
3. **Tasks** stored as Markdown + YAML frontmatter in `~/obsidian/1_Projects/<project>/`
4. **Task lookup** by issue number (exact) or title (case-insensitive substring)

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
