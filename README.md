# Tasky

CLI task manager for developers — integrates with Obsidian vaults and GitHub Issues.

## Setup

```bash
cargo build
./target/debug/tasky --help
```

## Commands

| Command | Description |
|---------|-------------|
| `tasky init` | Initialize configuration (vault path, pomodoro settings) |
| `tasky new "Title"` | Create a new task |
| `tasky list` | List tasks for current project |
| `tasky start <query>` | Start a task (in-progress + time tracking) |
| `tasky stop` | Pause time tracking |
| `tasky done <query>` | Mark a task as done |
| `tasky finish` | Push branch, create PR, merge, mark done |
| `tasky link` | Create `_tasky` symlink in project directory |
| `tasky day` | Show daily summary |
| `tasky week` | Show weekly summary |
| `tasky pomodoro start` | Start a pomodoro timer |

## Architecture

```
src/
  main.rs          — entry point, command routing
  cmd/             — CLI command handlers (stubs)
  config.rs        — TOML config loading/saving
  task.rs          — Task data model (frontmatter structs)
  utils.rs         — Project detection, git helpers
  pomodoro.rs      — Pomodoro timer (stub)
```

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

## License

MIT
