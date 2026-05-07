use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};

use super::gh;

/// App state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Loading,
    List,
    Detail,
    Confirm,
    Settings,
    SettingsEdit(usize),
    Done,
}

/// Pomodoro settings fields for editing
pub const SETTINGS_FIELDS: [&str; 4] = [
    "Work duration",
    "Short break",
    "Long break",
    "Long break interval",
];

pub struct App {
    state: State,
    issues: Vec<gh::Issue>,
    selected: usize,
    scroll: u16,
    detail_body: String,
    branch_created: Option<String>,
    error: Option<String>,
    // Settings
    settings_values: [u64; 4], // work, short, long, interval
    settings_idx: usize,
    settings_input: String,
    settings_saved: bool,
}

impl App {
    pub fn new() -> Self {
        // Load current pomodoro config
        let (work, short, long, interval) = match crate::config::Config::load() {
            Ok(cfg) => (
                cfg.pomodoro.work_duration,
                cfg.pomodoro.short_break,
                cfg.pomodoro.long_break,
                cfg.pomodoro.long_break_interval,
            ),
            Err(_) => (25, 5, 15, 4),
        };

        Self {
            state: State::Loading,
            issues: Vec::new(),
            selected: 0,
            scroll: 0,
            detail_body: String::new(),
            branch_created: None,
            error: None,
            settings_values: [work, short, long, interval],
            settings_idx: 0,
            settings_input: String::new(),
            settings_saved: false,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        // Fetch issues first
        self.issues = match gh::fetch_issues() {
            Ok(issues) => issues,
            Err(e) => {
                self.error = Some(format!("Failed to fetch issues: {e}"));
                self.state = State::Done;
                terminal.draw(|f| super::ui::draw(f, self))?;
                loop {
                    if event::poll(std::time::Duration::from_millis(100))? {
                        if let Event::Key(key) = event::read()? {
                            if key.kind == KeyEventKind::Press {
                                break;
                            }
                        }
                    }
                }
                return Ok(());
            }
        };
        self.state = State::List;

        // Main event loop
        loop {
            terminal.draw(|f| super::ui::draw(f, self))?;

            if !event::poll(std::time::Duration::from_millis(100))? {
                continue;
            }

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match self.state {
                    State::List => self.handle_list(key.code)?,
                    State::Detail => self.handle_detail(key.code)?,
                    State::Confirm => self.handle_confirm(key.code)?,
                    State::Settings => self.handle_settings(key.code)?,
                    State::SettingsEdit(_) => self.handle_settings_edit(key.code)?,
                    State::Done => return Ok(()),
                    State::Loading => {}
                }
            }
        }
    }

    fn handle_list(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = State::Done;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected < self.issues.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Enter => {
                if !self.issues.is_empty() {
                    let issue_num = self.issues[self.selected].number;
                    self.detail_body = gh::fetch_issue_body(issue_num).unwrap_or_default();
                    self.scroll = 0;
                    self.state = State::Detail;
                }
            }
            KeyCode::Char('s') => {
                self.settings_idx = 0;
                self.settings_saved = false;
                // Reload from config
                if let Ok(cfg) = crate::config::Config::load() {
                    self.settings_values = [
                        cfg.pomodoro.work_duration,
                        cfg.pomodoro.short_break,
                        cfg.pomodoro.long_break,
                        cfg.pomodoro.long_break_interval,
                    ];
                }
                self.state = State::Settings;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_detail(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = State::List;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll = self.scroll.saturating_add(1);
            }
            KeyCode::Enter => {
                self.state = State::Confirm;
            }
            KeyCode::Char('s') => {
                self.settings_idx = 0;
                self.settings_saved = false;
                self.state = State::Settings;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_confirm(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = State::Detail;
            }
            KeyCode::Char('y') | KeyCode::Enter => {
                let issue = &self.issues[self.selected];
                match gh::create_branch(issue.number, &issue.title) {
                    Ok(branch) => {
                        self.branch_created = Some(branch);
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to create branch: {e}"));
                    }
                }
                self.state = State::Done;
            }
            KeyCode::Char('n') => {
                self.state = State::Detail;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_settings(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = State::List;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.settings_idx > 0 {
                    self.settings_idx -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.settings_idx < SETTINGS_FIELDS.len() - 1 {
                    self.settings_idx += 1;
                }
            }
            KeyCode::Enter => {
                // Enter edit mode for selected field
                self.settings_input = self.settings_values[self.settings_idx].to_string();
                self.state = State::SettingsEdit(self.settings_idx);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_settings_edit(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc => {
                self.state = State::Settings;
            }
            KeyCode::Enter => {
                // Parse and save
                if let Ok(val) = self.settings_input.parse::<u64>() {
                    if val > 0 {
                        self.settings_values[self.settings_idx] = val;
                        self.save_settings()?;
                        self.settings_saved = true;
                    }
                }
                self.state = State::Settings;
            }
            KeyCode::Backspace => {
                self.settings_input.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.settings_input.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn save_settings(&self) -> Result<()> {
        let mut config = crate::config::Config::load()?;
        config.pomodoro.work_duration = self.settings_values[0];
        config.pomodoro.short_break = self.settings_values[1];
        config.pomodoro.long_break = self.settings_values[2];
        config.pomodoro.long_break_interval = self.settings_values[3];
        config.save()?;
        Ok(())
    }

    // Public getters for UI
    pub fn state(&self) -> State {
        self.state
    }

    pub fn issues(&self) -> &[gh::Issue] {
        &self.issues
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn scroll(&self) -> u16 {
        self.scroll
    }

    pub fn detail_body(&self) -> &str {
        &self.detail_body
    }

    pub fn branch_created(&self) -> Option<&str> {
        self.branch_created.as_deref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn settings_values(&self) -> &[u64; 4] {
        &self.settings_values
    }

    pub fn settings_idx(&self) -> usize {
        self.settings_idx
    }

    pub fn settings_input(&self) -> &str {
        &self.settings_input
    }

    pub fn settings_saved(&self) -> bool {
        self.settings_saved
    }
}
