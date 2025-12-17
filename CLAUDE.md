# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Tasky** is a command-line task manager written in Go that integrates with Obsidian vaults and GitHub issues. It allows developers to create, track, and manage tasks directly from the terminal with optional GitHub integration and Pomodoro timer functionality.

## Build & Development Commands

```bash
# Build the binary
go build -o tasky

# Run the application
./tasky [command] [args]

# Run a specific command for testing
./tasky new "Test Task"
./tasky list
./tasky pomodoro start
```

## Architecture & Code Structure

### High-Level Architecture

Tasky follows a layered CLI architecture using the `urfave/cli/v2` framework:

```
main.go (entry point)
  ↓
app/ (CLI framework setup)
  ├─ app.go (creates the CLI app)
  └─ commands.go (registers all commands)
      ↓
cmd/ (command handlers - execution layer)
  ├─ new.go (create tasks)
  ├─ list.go (list tasks)
  ├─ done.go (mark complete)
  ├─ start.go (mark in-progress)
  ├─ finish.go (complete PR workflow)
  ├─ pomodoro.go (timer commands)
  └─ link.go (symlink creation)
      ↓
task/ (business logic layer - task operations)
  ├─ task.go (read/write/query tasks)
  ├─ create.go (task creation logic)
  ├─ start.go (start task logic)
  └─ finish.go (finish workflow logic)
      ↓
utils/ (infrastructure layer)
  ├─ storage.go (file I/O for Obsidian vault)
  ├─ git.go (Git operations)
  ├─ common.go (utilities)
  └─ audio.go (sound playback)
      ↓
config/ (configuration & data models)
  ├─ config.go (TOML config loading/saving)
  └─ frontmatter_keys.go (YAML frontmatter constants)

pomodoro/ (timer implementation)
  ├─ pomodoro.go (timer logic)
  └─ pomodoro_anim.go (terminal animation)
```

### Key Data Flow

1. **Task Storage**: Tasks are stored as Markdown files with YAML frontmatter in the Obsidian vault at `$VAULT_PATH/$PROJECT_NAME/Tasky/`
2. **Configuration**: User config is stored in `~/.config/tasky/config.toml` with vault path, Pomodoro settings, and audio files
3. **Git Integration**: Commands can create branches, check remotes, and extract issue numbers from branch names
4. **Active Task Tracking**: The current git branch name is parsed to extract the issue number (e.g., `123-feature-name` → issue #123)

### Core Packages

**config/** - Configuration and data models
- `Config` struct: Contains vault path, Pomodoro settings, audio files
- `Frontmatter` struct: YAML data for each task (title, status, dates, pomodoro count, duration, issue number)
- Task status constants: `StatusTodo`, `StatusInProgress`, `StatusDone`
- `LoadConfig()` / `SaveConfig()`: Handle TOML configuration persistence

**task/** - Business logic for task operations
- `ReadTaskFile()` / `WriteTaskFile()`: YAML frontmatter parsing and serialization
- `GetTasks()`: Recursively scan vault for tasks, optional project filtering
- `MarkTaskDone()` / `MarkTaskInProgress*()`: Update task status and timestamps
- `IncrementPomodoroCountForActiveTask()`: Increment pomodoro count based on current git branch issue

**utils/** - Infrastructure and system interaction
- `GetTaskyDir()` / `CreateTaskyFile()` / `WriteToTaskyFile()` / `ReadFromTaskyFile()`: File I/O operations in vault
- `IsGitRepository()` / `HasGitHubRemote()`: Git repository detection
- `GetProjectName()`: Extract project name from git remote or directory
- `GetCurrentBranchName()`: Get current git branch
- `ExtractIssueNumberFromBranch()`: Parse issue number from branch naming pattern

**cmd/** - CLI command handlers
- Each command file implements a command handler (e.g., `NewCommand()`, `ListCommand()`)
- Uses `urfave/cli` Action callbacks for execution
- Delegates business logic to `task/` package

**pomodoro/** - Timer implementation
- `Pomodoro` struct: Timer state with work/break durations
- `pomodoro_anim.go`: Terminal animations for visual feedback

### Important Patterns

**YAML Frontmatter Format**:
```yaml
---
title: "Task Title"
status: "todo"  # or "in progress", "done"
created_date: "2024-06-10 10:00:00"
start_date: "2024-06-10 10:05:00"
done_date: ""
pomodoro_count: 0
duration: 0
issue: 123
---
```

**Project Name Detection** (in order of precedence):
1. Git remote URL repository name (GitHub only)
2. Git repository root directory name
3. Current working directory name
4. "unknown_project" fallback

**Issue Number Extraction**: Branch names follow pattern `<number>-<slug>` (e.g., `123-fix-login`), parsed with regex `^([0-9]+)-`

## Code Quality & Style

- **English-only code**: All identifiers, comments, and documentation must be in English (see `.cursor/rules/`)
- **Cursor Rule**: Link to `english-only-code.mdc` (symlinked from global rules)
- **Error Handling**: Use wrapped errors with `fmt.Errorf()` for context
- **CLI Framework**: `urfave/cli/v2` for command structure and argument parsing

## Dependencies

- `gopkg.in/yaml.v3`: YAML parsing for task frontmatter
- `github.com/urfave/cli/v2`: CLI framework and command routing
- `github.com/BurntSushi/toml`: TOML config parsing
- Standard library: `os`, `exec`, `filepath`, `regexp`, `bufio`, etc.

## Important Implementation Notes

**File I/O**: All task file operations go through `utils/storage.go` functions to maintain consistent paths. Never construct paths directly.

**Git Operations**: Executed via `exec.Command()`. Always handle errors gracefully as git commands may fail if not in a repo.

**Task Lookup**: Multiple functions walk the entire task directory tree (e.g., `MarkTaskDone`, `MarkTaskInProgress`). Consider performance if the vault grows very large.

**Pomodoro Duration**: Loaded from config, with a default of 25 minutes. The `IncrementPomodoroCountForActiveTask()` function reads the config fresh to ensure it has the latest pomodoro duration.

**YAML Parsing**: Uses inline struct embedding (`Frontmatter` embedded in `Task`). When marshaling, ensure the struct tags are correct for YAML output format.
