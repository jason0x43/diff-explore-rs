use std::{cell::RefCell, cmp::min};

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::{
    console::console_log,
    git::{git_diff, CommitRange, Stat},
    widget::WidgetWithBlock,
};

#[derive(Debug, Clone)]
pub struct Diff {
    list: ListState,
    height: usize,
    offset: usize,
    lines: Vec<String>,
    range: CommitRange,
    stat: Stat,
}

impl Diff {
    pub fn new(stat: &Stat, range: &CommitRange) -> Diff {
        let lines = git_diff(&range, &stat, None);

        Diff {
            lines,
            height: 0,
            offset: 0,
            list: ListState::default(),
            stat: stat.clone(),
            range: range.clone(),
        }
    }

    pub fn refresh(&mut self) {
        self.lines = git_diff(&self.range, &self.stat, None);
    }

    pub fn scroll_up(&mut self) {
        let delta = min(1, self.offset);
        self.offset -= delta;
    }

    pub fn page_up(&mut self) {
        let delta = min(self.height, self.offset);
        self.offset -= delta;
    }

    pub fn scroll_down(&mut self) {
        if self.lines.len() - self.offset > self.height {
            let limit = self.lines.len() - self.offset - self.height;
            let delta = min(limit, 1);
            self.offset += delta;
        }
    }

    pub fn page_down(&mut self) {
        if self.lines.len() - self.offset > self.height {
            let limit = self.lines.len() - self.offset - self.height;
            let delta = min(limit, self.height);
            self.offset += delta;
        }
    }
}

/// The Widget used to render Stats
pub struct DiffView<'a> {
    diff: &'a RefCell<Diff>,
    block: Option<Block<'a>>,
}

impl<'a> DiffView<'a> {
    pub fn new(diff: &RefCell<Diff>) -> DiffView {
        DiffView { diff, block: None }
    }
}

impl<'a> WidgetWithBlock<'a> for DiffView<'a> {
    fn block(&mut self, block: Block<'a>) {
        self.block = Some(block);
    }
}

impl<'a> Widget for DiffView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut diff = self.diff.borrow_mut();

        diff.height = area.height as usize;

        console_log(&format!("rendering with offset {}", diff.offset));

        let items: Vec<ListItem> = diff.lines[diff.offset..]
            .iter()
            .map(|c| {
                if c.len() > 0 {
                    let style = match c.chars().nth(0) {
                        Some('+') => Style::default().fg(Color::Indexed(2)),
                        Some('-') => Style::default().fg(Color::Indexed(1)),
                        _ => Style::default(),
                    };
                    ListItem::new(Spans::from(vec![Span::styled(
                        c.clone(),
                        style,
                    )]))
                } else {
                    ListItem::new(Spans::from(vec![Span::from("")]))
                }
            })
            .collect();

        let list = List::new(items).block(self.block.unwrap());
        StatefulWidget::render(list, area, buf, &mut diff.list);
    }
}
