use std::cmp::min;

use tui::widgets::ListState;

#[derive(Debug, Clone)]
pub struct ListData {
    pub state: ListState,
    pub height: usize,
    pub count: usize,
}

impl ListData {
    pub fn new() -> ListData {
        let mut state = ListState::default();
        state.select(Some(0));

        ListData {
            state,
            height: 0,
            count: 0,
        }
    }
}

pub trait ListInfo {
    /// Return the number of list items
    fn list_count(&self) -> usize;
    /// Return the current position in the list
    fn list_pos(&self) -> usize;
}

pub trait ListCursor: ListInfo {
    fn list_state(&mut self) -> &mut ListState;

    fn list_height(&self) -> usize;

    fn set_list_height(&mut self, height: usize);

    fn cursor(&self) -> usize;

    fn cursor_down(&mut self) {
        if self.list_count() == 0 {
            return
        }
        let cursor = min(self.cursor() + 1, self.list_count() - 1);
        self.list_state().select(Some(cursor));
    }

    fn cursor_page_down(&mut self) {
        if self.list_count() == 0 {
            return
        }
        let cursor =
            min(self.cursor() + self.list_height(), self.list_count() - 1);
        self.list_state().select(Some(cursor));
    }

    fn cursor_up(&mut self) {
        if self.list_count() == 0 {
            return
        }
        let cursor = self.cursor();
        let delta = min(self.cursor(), 1);
        self.list_state().select(Some(cursor - delta));
    }

    fn cursor_page_up(&mut self) {
        if self.list_count() == 0 {
            return
        }
        let cursor = self.cursor();
        let delta = min(cursor, self.list_height());
        self.list_state().select(Some(cursor - delta));
    }
}
