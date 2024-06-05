use std::cmp::min;

use ratatui::widgets::ListState;

#[derive(Debug, Clone)]
pub struct ListData {
    pub state: ListState,
    pub height: usize,
}

impl ListData {
    pub fn new() -> ListData {
        let mut state = ListState::default();
        state.select(Some(0));

        ListData { state, height: 0 }
    }
}

pub trait ListInfo {
    /// Return the number of list items
    fn list_count(&self) -> usize;

    /// Return the current position in the list
    fn list_pos(&self) -> usize;

    /// Update the position in the list
    fn set_list_pos(&mut self, pos: usize);
}

pub trait ListScroll: ListInfo {
    /// Return the visible height of the list
    fn height(&self) -> usize;

    /// Scroll one line up
    fn scroll_up(&mut self) {
        let offset = self.list_pos();
        let delta = min(1, offset);
        self.set_list_pos(offset - delta);
    }

    /// Scroll one visible page up
    fn page_up(&mut self) {
        let offset = self.list_pos();
        let delta = min(self.height() - 1, offset);
        self.set_list_pos(offset - delta);
    }

    /// Scroll one line down
    fn scroll_down(&mut self) {
        let count = self.list_count();
        let offset = self.list_pos();
        let height = self.height();
        if count - offset > height {
            let limit = count - offset - height;
            let delta = min(limit, 1);
            self.set_list_pos(offset + delta);
        }
    }

    /// Scroll one visible page down
    fn page_down(&mut self) {
        let offset = self.list_pos();
        let height = self.height();
        if self.list_count() - offset > height {
            let limit = self.list_count() - offset - height;
            let delta = min(limit, height - 1);
            self.set_list_pos(offset + delta);
        }
    }

    /// Scroll to a specific line
    fn scroll_to(&mut self, line: usize) {
        self.set_list_pos(line);
    }

    /// Scroll to the first line in the list
    fn scroll_top(&mut self) {
        self.set_list_pos(0);
    }

    /// Scroll to the last line in the list
    fn scroll_bottom(&mut self) {
        self.scroll_to(self.list_count() - self.height());
    }
}

/// A trait for widgets with a visible cursor
pub trait ListCursor: ListInfo + ListScroll {
    fn list_state(&self) -> &ListState;
    fn list_state_mut(&mut self) -> &mut ListState;

    fn cursor(&self) -> usize {
        self.list_state().selected().unwrap_or(0)
    }

    fn cursor_down(&mut self) {
        if self.list_count() == 0 {
            return;
        }
        let cursor = min(self.cursor() + 1, self.list_count() - 1);
        self.list_state_mut().select(Some(cursor));
    }

    fn cursor_page_down(&mut self) {
        if self.list_count() == 0 {
            return;
        }
        let cursor =
            min(self.cursor() + self.height() - 1, self.list_count() - 1);
        self.list_state_mut().select(Some(cursor));
    }

    fn cursor_up(&mut self) {
        if self.list_count() == 0 {
            return;
        }
        let cursor = self.cursor();
        let delta = min(self.cursor(), 1);
        self.list_state_mut().select(Some(cursor - delta));
    }

    fn cursor_page_up(&mut self) {
        if self.list_count() == 0 {
            return;
        }
        let cursor = self.cursor();
        let delta = min(cursor, self.height() - 1);
        self.list_state_mut().select(Some(cursor - delta));
    }

    fn cursor_to_bottom(&mut self) {
        let height = self.list_count();
        self.list_state_mut().select(Some(height - 1));
    }

    fn cursor_to_top(&mut self) {
        self.list_state_mut().select(Some(0));
    }

    fn cursor_to(&mut self, line: usize) {
        if line < self.list_count() {
            self.list_state_mut().select(Some(line));
        }
    }
}
