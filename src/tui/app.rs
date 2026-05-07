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
    Done,
}

pub struct App {
    state: State,
    issues: Vec<gh::Issue>,
    selected: usize,
    scroll: u16,
    detail_body: String,
    branch_created: Option<String>,
    error: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: State::Loading,
            issues: Vec::new(),
            selected: 0,
            scroll: 0,
            detail_body: String::new(),
            branch_created: None,
            error: None,
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
                // Still show one frame so user sees the error
                terminal.draw(|f| super::ui::draw(f, self))?;
                // Wait for any key
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
                // Create branch for this issue
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
}
