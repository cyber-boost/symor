#[derive(Debug, Clone)]
pub enum FileAction {
    View,
    Restore,
    Delete,
    Watch,
    Unwatch,
}
pub struct NavigationHandler {
    pub current_index: usize,
    pub page_size: usize,
}
impl NavigationHandler {
    pub fn new() -> Self {
        Self {
            current_index: 0,
            page_size: 20,
        }
    }
    pub fn next(&mut self, max_items: usize) {
        if self.current_index < max_items.saturating_sub(1) {
            self.current_index += 1;
        }
    }
    pub fn previous(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
    }
    pub fn page_up(&mut self, max_items: usize) {
        self.current_index = self
            .current_index
            .saturating_sub(self.page_size)
            .min(max_items.saturating_sub(1));
    }
    pub fn page_down(&mut self, max_items: usize) {
        self.current_index = (self.current_index + self.page_size)
            .min(max_items.saturating_sub(1));
    }
}
pub struct InputHandler {
    pub buffer: String,
    pub cursor_position: usize,
}
impl InputHandler {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor_position: 0,
        }
    }
    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.buffer.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.buffer.len() {
            self.cursor_position += 1;
        }
    }
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor_position = 0;
    }
}