use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use super::app::{App, State, SETTINGS_FIELDS};
use super::gh;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    match app.state() {
        State::Loading => draw_loading(f, size),
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

fn draw_list(f: &mut Frame, app: &App, size: Rect) {
    let issues = app.issues();

    let items: Vec<ListItem> = issues
        .iter()
        .enumerate()
        .map(|(i, issue)| {
            let labels: String = issue
                .labels
                .iter()
                .map(|l| format!("[{}]", l.name))
                .collect::<Vec<_>>()
                .join(" ");

            let style = if i == app.selected() {
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

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Issues (j/k: navigate, Enter: view, s: settings, q: quit) "),
        )
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
    let issue = &app.issues()[app.selected()];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(5),    // body
            Constraint::Length(1), // footer
        ])
        .split(size);

    // Header: issue number + title
    let header = Paragraph::new(Line::from(vec![
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
    ]))
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
    let issue = &app.issues()[app.selected()];

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

    let text = vec![
        Line::from(Span::styled(
            format!("Work on #{}?", issue.number),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            &issue.title,
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
    let cursor = if input.is_empty() { " " } else { "" };

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
