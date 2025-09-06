use ratatui::{
    layout::Rect, style::{Color, Modifier, Style},
    text::Span, widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
pub struct FileListView;
impl FileListView {
    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        items: &[crate::WatchedItem],
        selected: Option<usize>,
    ) {
        let items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if Some(i) == selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(
                    Span::styled(format!("{}: {}", item.id, item.path.display()), style),
                )
            })
            .collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Watched Files"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        f.render_widget(list, area);
    }
}
pub struct VersionHistoryView;
impl VersionHistoryView {
    pub fn render(&self, f: &mut Frame, area: Rect, versions: &[crate::FileVersion]) {
        let items: Vec<ListItem> = versions
            .iter()
            .map(|version| {
                ListItem::new(
                    format!(
                        "{}: {} bytes ({})", version.id, version.size, version.timestamp
                        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
                        .as_secs()
                    ),
                )
            })
            .collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Version History"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        f.render_widget(list, area);
    }
}
pub struct SettingsView;
impl SettingsView {
    pub fn render(&self, f: &mut Frame, area: Rect, config: &crate::SymorConfig) {
        let text = format!(
            "Home Directory: {}\n\
             Versioning Enabled: {}\n\
             Max Versions: {}\n\
             Compression Level: {}\n\
             Link Type: {}\n\
             Preserve Permissions: {}",
            config.home_dir.display(), config.versioning.enabled, config.versioning
            .max_versions, config.versioning.compression, config.linking.link_type,
            config.linking.preserve_permissions
        );
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Settings"));
        f.render_widget(paragraph, area);
    }
}
pub struct LogsView;
impl LogsView {
    pub fn render(&self, f: &mut Frame, area: Rect, logs: &[String]) {
        let items: Vec<ListItem> = logs
            .iter()
            .map(|log| ListItem::new(log.as_str()))
            .collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Logs"));
        f.render_widget(list, area);
    }
}
pub struct HelpView;
impl HelpView {
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let help_text = "Symor TUI Help\n\
                        ==============\n\
                        \n\
                        Navigation:\n\
                        h - Help\n\
                        f - File List\n\
                        v - Version History\n\
                        s - Settings\n\
                        l - Logs\n\
                        q - Quit\n\
                        \n\
                        Use arrow keys to navigate lists";
        let paragraph = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"));
        f.render_widget(paragraph, area);
    }
}