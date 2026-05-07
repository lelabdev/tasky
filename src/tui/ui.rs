use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use super::app::{App, State, SETTINGS_FIELDS};
use crate::task::TaskStatus;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    match app.state() {
        State::Loading => draw_loading(f, size),
        State::ProjectPicker => draw_project_picker(f, app, size),
        State::NewProjectInput => draw_new_project_input(f, app, size),
        State::List => draw_list(f, app, size),
        State::Detail => draw_detail(f, app, size),
        State::Confirm => draw_confirm(f, app, size),
        State::Settings => draw_settings(f, app, size),
        State::SettingsEdit(_) => draw_settings_edit(f, app, size),
        State::Done => draw_done(f, app, size),
    }
}

fn draw_loading(f: &mut Frame, size: Rect) {
    let text = vec![
        Line::from("Loading issues..."),
        Line::from(""),
        Line::from("Press q to quit"),
    ];
    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Cyan));
    f.render_widget(paragraph, size);
}

// ── Project Picker ──────────────────────────────────────────────────

fn draw_project_picker(f: &mut Frame, app: &App, size: Rect) {
    let projects = app.projects();

    let mut items: Vec<ListItem> = projects
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.project_selected() {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(
                format!("  📁 {}", name),
                style,
            )))
        })
        .collect();

    // Add "New project..." option
    let new_style = if app.project_selected() == projects.len() {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    items.push(ListItem::new(Line::from(Span::styled(
        "  ➕ New project...",
        new_style,
    ))));

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Select a Project (j/k: navigate, Enter: select, n: new, q: quit) "),
    );

    let mut state = ListState::default();
    state.select(Some(app.project_selected()));
    f.render_stateful_widget(list, size, &mut state);
}

fn draw_new_project_input(f: &mut Frame, app: &App, size: Rect) {
    // Draw project picker in background
    draw_project_picker(f, app, size);

    // Draw input dialog on top
    let dialog_width = 50.min(size.width - 4);
    let dialog_height = 5;
    let dialog = Rect {
        x: (size.width.saturating_sub(dialog_width)) / 2,
        y: (size.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    f.render_widget(Clear, dialog);

    let input = app.new_project_input();

    let text = vec![
        Line::from(Span::styled(
            "Create New Project",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Name: "),
            Span::styled(
                format!("{}█", input),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(Span::styled(
            "  Enter: create  Esc: cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, dialog);
}

// ── Issue / Task List ───────────────────────────────────────────────

fn status_icon(status: &TaskStatus) -> (&'static str, Color) {
    match status {
        TaskStatus::Todo => ("○", Color::Blue),
        TaskStatus::InProgress => ("◐", Color::Yellow),
        TaskStatus::Done => ("●", Color::Green),
    }
}

fn draw_list(f: &mut Frame, app: &App, size: Rect) {
    let local_tasks = app.local_tasks();
    let issues = app.issues();

    let mut items: Vec<ListItem> = Vec::new();

    // ── Local tasks section ──
    if !local_tasks.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "── Local Tasks ──",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        ))));

        for (i, task) in local_tasks.iter().enumerate() {
            let (icon, icon_color) = status_icon(&task.frontmatter.status);
            let duration = if task.frontmatter.duration > 0 {
                format!(" {}min", task.frontmatter.duration)
            } else {
                String::new()
            };

            let style = if i == app.selected() {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(icon_color)),
                Span::styled(format!("{}", task.frontmatter.title), style),
                Span::styled(duration, Style::default().fg(Color::DarkGray)),
            ]);
            items.push(ListItem::new(line));
        }
    }

    // ── GitHub issues section ──
    if !issues.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "── GitHub Issues ──",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))));

        for (i, issue) in issues.iter().enumerate() {
            let idx = local_tasks.len() + i;
            let labels: String = issue
                .labels
                .iter()
                .map(|l| format!("[{}]", l.name))
                .collect::<Vec<_>>()
                .join(" ");

            let style = if idx == app.selected() {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = if labels.is_empty() {
                Line::from(Span::styled(
                    format!("#{}  {}", issue.number, issue.title),
                    style,
                ))
            } else {
                Line::from(vec![
                    Span::styled(format!("#{}  ", issue.number), style),
                    Span::styled(format!("{} ", issue.title), style),
                    Span::styled(labels, Style::default().fg(Color::DarkGray)),
                ])
            };

            items.push(ListItem::new(line));
        }
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from("No tasks or issues found.")));
    }

    let project_name = app.selected_project().unwrap_or("unknown");
    let title = format!(
        " {} — Tasks (j/k: navigate, Enter: view, Esc: back, s: settings, q: quit) ",
        project_name
    );

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(app.selected()));
    f.render_stateful_widget(list, size, &mut state);
}

fn draw_detail(f: &mut Frame, app: &App, size: Rect) {
    let local_tasks = app.local_tasks();
    let issues = app.issues();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(5),    // body
            Constraint::Length(1), // footer
        ])
        .split(size);

    // Build header based on whether it's a local task or a GitHub issue
    let header_line = if app.selected() < local_tasks.len() {
        let task = &local_tasks[app.selected()];
        let (icon, icon_color) = status_icon(&task.frontmatter.status);
        Line::from(vec![
            Span::styled(
                format!("{} ", icon),
                Style::default().fg(icon_color),
            ),
            Span::styled(
                &task.frontmatter.title,
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        let issue_idx = app.selected() - local_tasks.len();
        if issue_idx < issues.len() {
            let issue = &issues[issue_idx];
            Line::from(vec![
                Span::styled(
                    format!("#{} ", issue.number),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &issue.title,
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ])
        } else {
            Line::from("(unknown)")
        }
    };

    let header = Paragraph::new(header_line)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    // Body
    let body_text = if app.detail_body().is_empty() {
        "(no description)".to_string()
    } else {
        app.detail_body().to_string()
    };

    let body = Paragraph::new(body_text)
        .block(
            Block::default()
                .title(" Description ")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll(), 0));
    f.render_widget(body, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(Span::styled(
        "Enter: work on this  |  s: settings  |  Esc: back  |  j/k: scroll",
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(footer, chunks[2]);
}

fn draw_confirm(f: &mut Frame, app: &App, size: Rect) {
    // Draw detail in background
    draw_detail(f, app, size);

    // Draw confirm dialog on top
    let dialog_width = 50.min(size.width - 4);
    let dialog_height = 5;
    let dialog = Rect {
        x: (size.width.saturating_sub(dialog_width)) / 2,
        y: (size.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    f.render_widget(Clear, dialog);

    let local_tasks = app.local_tasks();
    let issues = app.issues();

    let (label, sublabel) = if app.selected() < local_tasks.len() {
        let task = &local_tasks[app.selected()];
        ("Open this task?".to_string(), task.frontmatter.title.clone())
    } else {
        let issue_idx = app.selected() - local_tasks.len();
        if issue_idx < issues.len() {
            let issue = &issues[issue_idx];
            (
                format!("Work on #{}?", issue.number),
                issue.title.clone(),
            )
        } else {
            ("?".to_string(), String::new())
        }
    };

    let text = vec![
        Line::from(Span::styled(
            label,
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            sublabel,
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from("  [y] Create branch  [n] Cancel"),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, dialog);
}

fn draw_settings(f: &mut Frame, app: &App, size: Rect) {
    let values = app.settings_values();
    let idx = app.settings_idx();

    let units = ["min", "min", "min", "pomodoros"];

    let items: Vec<ListItem> = SETTINGS_FIELDS
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let style = if i == idx {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(format!("  {} ", field), style),
                Span::styled(
                    format!("{} {}", values[i], units[i]),
                    Style::default().fg(Color::Cyan),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let mut all_items = items;
    all_items.push(ListItem::new(Line::from("")));
    all_items.push(ListItem::new(Line::from(Span::styled(
        "  Press Enter to edit, Esc to go back",
        Style::default().fg(Color::DarkGray),
    ))));

    if app.settings_saved() {
        all_items.push(ListItem::new(Line::from("")));
        all_items.push(ListItem::new(Line::from(Span::styled(
            "  ✓ Saved!",
            Style::default().fg(Color::Green),
        ))));
    }

    let list = List::new(all_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .title(" 🍅 Pomodoro Settings (j/k: navigate) "),
    );

    let mut state = ListState::default();
    state.select(Some(idx));
    f.render_stateful_widget(list, size, &mut state);
}

fn draw_settings_edit(f: &mut Frame, app: &App, size: Rect) {
    // Draw settings in background
    draw_settings(f, app, size);

    // Draw input dialog on top
    let dialog_width = 50.min(size.width - 4);
    let dialog_height = 5;
    let dialog = Rect {
        x: (size.width.saturating_sub(dialog_width)) / 2,
        y: (size.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    f.render_widget(Clear, dialog);

    let field = SETTINGS_FIELDS[app.settings_idx()];
    let current = app.settings_values()[app.settings_idx()];
    let input = app.settings_input();

    let text = vec![
        Line::from(Span::styled(
            format!("Edit: {}", field),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("Current: {}min", current),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  New value: "),
            Span::styled(
                format!("{}█", input),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(Span::styled(
            "  Enter: save  Esc: cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, dialog);
}

fn draw_done(f: &mut Frame, app: &App, size: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    if let Some(branch) = app.branch_created() {
        lines.push(Line::from(Span::styled(
            "✓ Branch created!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(format!("  {branch}")));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Use `tasky start <issue>` to begin tracking.",
            Style::default().fg(Color::DarkGray),
        )));
    } else if let Some(err) = app.error() {
        lines.push(Line::from(Span::styled(
            "✗ Error",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(err.to_string()));
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, size);
}
