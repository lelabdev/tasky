use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};

use super::gh;
use crate::task::Task;

/// App state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Loading,
    ProjectPicker,
    NewProjectInput,
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
    // Project picker
    projects: Vec<String>,
    project_selected: usize,
    selected_project: Option<String>,
    new_project_input: String,
    // Issues
    issues: Vec<gh::Issue>,
    // Local tasks
    local_tasks: Vec<Task>,
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

/// List all vault projects from `~/obsidian/1_Projects/`
fn list_vault_projects(vault_path: &str) -> Vec<String> {
    let dir = std::path::Path::new(vault_path).join("1_Projects");
    std::fs::read_dir(&dir)
        .unwrap_or_else(|_| panic!("cannot read {}", dir.display()))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .filter(|s| !s.starts_with('_') && !s.starts_with('.'))
        .collect()
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

        // Load vault projects
        let vault_path = crate::config::Config::load()
            .map(|c| c.vault.path.clone())
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .map(|p| p.join("obsidian").display().to_string())
                    .unwrap_or_else(|| "~/obsidian".to_string())
            });

        let mut projects = list_vault_projects(&vault_path);
        projects.sort();

        Self {
            state: State::ProjectPicker,
            projects,
            project_selected: 0,
            selected_project: None,
            new_project_input: String::new(),
            issues: Vec::new(),
            local_tasks: Vec::new(),
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
                    State::ProjectPicker => self.handle_project_picker(key.code)?,
                    State::NewProjectInput => self.handle_new_project_input(key.code)?,
                    State::Loading => self.handle_loading(key.code)?,
                    State::List => self.handle_list(key.code)?,
                    State::Detail => self.handle_detail(key.code)?,
                    State::Confirm => self.handle_confirm(key.code)?,
                    State::Settings => self.handle_settings(key.code)?,
                    State::SettingsEdit(_) => self.handle_settings_edit(key.code)?,
                    State::Done => return Ok(()),
                }
            }
        }
    }

    fn handle_loading(&mut self, key: KeyCode) -> Result<()> {
        if matches!(key, KeyCode::Char('q') | KeyCode::Esc) {
            self.state = State::Done;
        }
        Ok(())
    }

    // ── Project Picker ──────────────────────────────────────────────

    fn handle_project_picker(&mut self, key: KeyCode) -> Result<()> {
        let total_items = self.projects.len() + 1; // +1 for "New project..."
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = State::Done;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.project_selected > 0 {
                    self.project_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.project_selected < total_items.saturating_sub(1) {
                    self.project_selected += 1;
                }
            }
            KeyCode::Char('n') => {
                self.new_project_input.clear();
                self.state = State::NewProjectInput;
            }
            KeyCode::Enter => {
                if self.project_selected < self.projects.len() {
                    // Selected an existing project
                    let project = self.projects[self.project_selected].clone();
                    self.load_project(&project)?;
                } else {
                    // Selected "New project..."
                    self.new_project_input.clear();
                    self.state = State::NewProjectInput;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_new_project_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc => {
                self.state = State::ProjectPicker;
            }
            KeyCode::Enter => {
                let name = self.new_project_input.trim().to_string();
                if !name.is_empty() {
                    // Create the project directory
                    let vault_path = crate::config::Config::load()
                        .map(|c| c.vault.path.clone())
                        .unwrap_or_else(|_| "~/obsidian".to_string());
                    let dir =
                        crate::utils::get_tasky_dir(&vault_path, &name);
                    if !dir.exists() {
                        std::fs::create_dir_all(&dir)?;
                    }
                    // Add to project list and load
                    self.load_project(&name)?;
                }
            }
            KeyCode::Backspace => {
                self.new_project_input.pop();
            }
            KeyCode::Char(c) => {
                self.new_project_input.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn load_project(&mut self, project: &str) -> Result<()> {
        self.selected_project = Some(project.to_string());

        // Load local tasks for this project
        let vault_path = crate::config::Config::load()
            .map(|c| c.vault.path.clone())
            .unwrap_or_else(|_| "~/obsidian".to_string());
        self.local_tasks = crate::storage::list_tasks(&vault_path, Some(project))
            .unwrap_or_default();

        // Fetch GitHub issues
        self.state = State::Loading;
        // We'll set state to Loading so the UI shows it, then transition to List
        // Issues are fetched inline here
        self.issues = match gh::fetch_issues() {
            Ok(issues) => issues,
            Err(_) => Vec::new(), // Don't fail if gh is not available
        };

        self.selected = 0;
        self.state = State::List;
        Ok(())
    }

    // ── Issue List ──────────────────────────────────────────────────

    fn handle_list(&mut self, key: KeyCode) -> Result<()> {
        let total_items = self.local_tasks.len() + self.issues.len();
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = State::ProjectPicker;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected < total_items.saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Enter => {
                if self.selected < self.local_tasks.len() {
                    // Local task selected — show detail
                    let task = &self.local_tasks[self.selected];
                    self.detail_body = task.body.clone();
                    self.scroll = 0;
                    self.state = State::Detail;
                } else if !self.issues.is_empty() {
                    let issue_idx = self.selected - self.local_tasks.len();
                    if issue_idx < self.issues.len() {
                        let issue_num = self.issues[issue_idx].number;
                        self.detail_body = gh::fetch_issue_body(issue_num).unwrap_or_default();
                        self.scroll = 0;
                        self.state = State::Detail;
                    }
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
                // Only create branch for GitHub issues
                if self.selected >= self.local_tasks.len() {
                    let issue_idx = self.selected - self.local_tasks.len();
                    if issue_idx < self.issues.len() {
                        let issue = &self.issues[issue_idx];
                        match gh::create_branch(issue.number, &issue.title) {
                            Ok(branch) => {
                                self.branch_created = Some(branch);
                            }
                            Err(e) => {
                                self.error = Some(format!("Failed to create branch: {e}"));
                            }
                        }
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

    pub fn projects(&self) -> &[String] {
        &self.projects
    }

    pub fn project_selected(&self) -> usize {
        self.project_selected
    }

    pub fn selected_project(&self) -> Option<&str> {
        self.selected_project.as_deref()
    }

    pub fn new_project_input(&self) -> &str {
        &self.new_project_input
    }

    pub fn issues(&self) -> &[gh::Issue] {
        &self.issues
    }

    pub fn local_tasks(&self) -> &[Task] {
        &self.local_tasks
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
