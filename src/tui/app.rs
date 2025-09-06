use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal, Frame, prelude::Rect};
use std::{io, time::Duration};
#[derive(Debug, Clone)]
pub struct AppState {
    pub watched_items: Vec<crate::WatchedItem>,
    pub current_view: ViewType,
    pub selected_item: Option<usize>,
    pub filter: String,
    pub running: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewType {
    FileList,
    VersionHistory,
    Settings,
    Logs,
    Help,
}
pub struct SymorTUI {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: AppState,
}
impl SymorTUI {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let state = AppState {
            watched_items: Vec::new(),
            current_view: ViewType::FileList,
            selected_item: None,
            filter: String::new(),
            running: true,
        };
        Ok(Self { terminal, state })
    }
    pub fn run(&mut self) -> Result<()> {
        while self.state.running {
            self.draw()?;
            self.handle_events()?;
        }
        Ok(())
    }
    fn draw(&mut self) -> Result<()> {
        let current_view = self.state.current_view.clone();
        let watched_items = self.state.watched_items.clone();
        let selected_item = self.state.selected_item;
        self.terminal
            .draw(|f| {
                use ratatui::layout::{Constraint, Direction, Layout};
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(1),
                        Constraint::Length(1),
                    ])
                    .split(size);
                let header = ratatui::widgets::Paragraph::new(
                        "Symor TUI - File Mirroring & Version Control",
                    )
                    .style(
                        ratatui::style::Style::default()
                            .fg(ratatui::style::Color::Cyan)
                            .add_modifier(ratatui::style::Modifier::BOLD),
                    )
                    .block(
                        ratatui::widgets::Block::default()
                            .borders(ratatui::widgets::Borders::ALL)
                            .title("Symor"),
                    );
                f.render_widget(header, chunks[0]);
                match current_view {
                    ViewType::FileList => {
                        Self::draw_file_list_static(
                            f,
                            chunks[1],
                            &watched_items,
                            selected_item,
                        )
                    }
                    ViewType::VersionHistory => {
                        Self::draw_version_history_static(f, chunks[1])
                    }
                    ViewType::Settings => Self::draw_settings_static(f, chunks[1]),
                    ViewType::Logs => Self::draw_logs_static(f, chunks[1]),
                    ViewType::Help => Self::draw_help_static(f, chunks[1]),
                }
                let footer_text = match current_view {
                    ViewType::FileList => {
                        "↑↓ Navigate | Enter Select | h Help | q Quit"
                    }
                    ViewType::VersionHistory => {
                        "↑↓ Navigate | Enter Restore | h Help | q Quit"
                    }
                    ViewType::Settings => "h Help | q Quit",
                    ViewType::Logs => "↑↓ Scroll | h Help | q Quit",
                    ViewType::Help => "q Quit",
                };
                let footer = ratatui::widgets::Paragraph::new(footer_text)
                    .style(
                        ratatui::style::Style::default().fg(ratatui::style::Color::White),
                    );
                f.render_widget(footer, chunks[2]);
            })?;
        Ok(())
    }
    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.state.running = false;
                    }
                    KeyCode::Char('h') => {
                        self.state.current_view = ViewType::Help;
                    }
                    KeyCode::Char('f') => {
                        self.state.current_view = ViewType::FileList;
                    }
                    KeyCode::Char('v') => {
                        self.state.current_view = ViewType::VersionHistory;
                    }
                    KeyCode::Char('s') => {
                        self.state.current_view = ViewType::Settings;
                    }
                    KeyCode::Char('l') => {
                        self.state.current_view = ViewType::Logs;
                    }
                    KeyCode::Up => {
                        self.handle_navigation(-1);
                    }
                    KeyCode::Down => {
                        self.handle_navigation(1);
                    }
                    KeyCode::Enter => {
                        self.handle_selection();
                    }
                    KeyCode::PageUp => {
                        self.handle_page_navigation(-10);
                    }
                    KeyCode::PageDown => {
                        self.handle_page_navigation(10);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    fn handle_navigation(&mut self, direction: i32) {
        let max_items = match self.state.current_view {
            ViewType::FileList => self.state.watched_items.len(),
            _ => 0,
        };
        if max_items > 0 {
            let current = self.state.selected_item.unwrap_or(0) as i32;
            let new_index = (current + direction).max(0).min(max_items as i32 - 1)
                as usize;
            self.state.selected_item = Some(new_index);
        }
    }
    fn handle_page_navigation(&mut self, direction: i32) {
        let page_size = 10;
        let max_items = match self.state.current_view {
            ViewType::FileList => self.state.watched_items.len(),
            _ => 0,
        };
        if max_items > 0 {
            let current = self.state.selected_item.unwrap_or(0) as i32;
            let new_index = (current + direction * page_size)
                .max(0)
                .min(max_items as i32 - 1) as usize;
            self.state.selected_item = Some(new_index);
        }
    }
    fn handle_selection(&mut self) {
        match self.state.current_view {
            ViewType::FileList => {
                if let Some(index) = self.state.selected_item {
                    if index < self.state.watched_items.len() {
                        self.state.current_view = ViewType::VersionHistory;
                    }
                }
            }
            ViewType::VersionHistory => {}
            _ => {}
        }
    }
    pub fn shutdown(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
    pub fn get_state(&self) -> &AppState {
        &self.state
    }
    pub fn update_state<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut AppState),
    {
        updater(&mut self.state);
    }
    fn draw_file_list_static(
        f: &mut Frame,
        area: Rect,
        watched_items: &[crate::WatchedItem],
        selected_item: Option<usize>,
    ) {
        use crate::tui::views::FileListView;
        let view = FileListView;
        view.render(f, area, watched_items, selected_item);
    }
    fn draw_version_history_static(f: &mut Frame, area: Rect) {
        use crate::tui::views::VersionHistoryView;
        let view = VersionHistoryView;
        let versions: Vec<crate::FileVersion> = Vec::new();
        view.render(f, area, &versions);
    }
    fn draw_settings_static(f: &mut Frame, area: Rect) {
        use crate::tui::views::SettingsView;
        let view = SettingsView;
        let config = crate::SymorConfig::default();
        view.render(f, area, &config);
    }
    fn draw_logs_static(f: &mut Frame, area: Rect) {
        use crate::tui::views::LogsView;
        let view = LogsView;
        let logs: Vec<String> = vec!["TUI initialized".to_string()];
        view.render(f, area, &logs);
    }
    fn draw_help_static(f: &mut Frame, area: Rect) {
        use crate::tui::views::HelpView;
        let view = HelpView;
        view.render(f, area);
    }
}
impl Drop for SymorTUI {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_app_state() {
        let state = AppState {
            watched_items: Vec::new(),
            current_view: ViewType::FileList,
            selected_item: None,
            filter: String::new(),
            running: true,
        };
        assert_eq!(state.current_view, ViewType::FileList);
        assert!(state.running);
    }
}