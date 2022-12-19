use std::cmp::min;

use tui::widgets::ListState;

pub trait ListCursor {
    fn cursor(&self) -> usize;
    fn mut_list(&mut self) -> &mut ListData;

    fn cursor_down(&mut self) {
        self.mut_list().cursor_down()
    }

    fn cursor_page_down(&mut self) {
        self.mut_list().cursor_page_down()
    }

    fn cursor_up(&mut self) {
        self.mut_list().cursor_up()
    }

    fn cursor_page_up(&mut self) {
        self.mut_list().cursor_page_up()
    }
}

#[derive(Debug, Clone)]
pub struct ListData {
    state: ListState,
    height: usize,
    count: usize,
}

impl ListData {
    pub fn new(count: usize) -> ListData {
        let mut state = ListState::default();
        state.select(Some(0));

        ListData {
            state,
            height: 0,
            count,
        }
   }

    pub fn mut_state(&mut self) -> &mut ListState {
        &mut self.state
    }

    pub fn set_count(&mut self, count: usize) {
        self.count = count
    }

    pub fn set_height(&mut self, height: usize) {
        self.height = height
    }
}

impl ListCursor for ListData {
    fn cursor(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }

    fn mut_list(&mut self) -> &mut ListData {
        self
    }

    fn cursor_down(&mut self) {
        let cursor = min(self.cursor() + 1, self.count - 1);
        self.state.select(Some(cursor));
    }

    fn cursor_page_down(&mut self) {
        let cursor = min(self.cursor() + self.height, self.count - 1);
        self.state.select(Some(cursor));
    }

    fn cursor_up(&mut self) {
        let cursor = self.cursor();
        let delta = min(self.cursor(), 1);
        self.state.select(Some(cursor - delta));
    }

    fn cursor_page_up(&mut self) {
        let cursor = self.cursor();
        let delta = min(cursor, self.height);
        self.state.select(Some(cursor - delta));
    }
}
