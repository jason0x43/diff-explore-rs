use tui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

use crate::cursor::CursorView;

pub struct Commits {
    pub cursor: u32,
    pub commits: Vec<String>,
}

impl StatefulWidget for Commits {
    type State = Commits;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    }
}

impl CursorView for Commits {
    fn cursor_down(&mut self) {}
    fn cursor_up(&mut self) {}
}
