use std::cell::RefCell;

use list_helper_core::{HasListCount, ListCursor, ListData};
use list_helper_macro::ListCursor;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::{
    git::{git_diff_stat, CommitRange, Stat},
    widget::WidgetWithBlock,
};

#[derive(Debug, Clone, ListCursor)]
pub struct Stats {
    list: ListData,
    range: CommitRange,
    stats: Vec<Stat>,
}

impl HasListCount for Stats {
    fn list_count(&self) -> usize {
        self.stats.len()
    }
}

impl Stats {
    pub fn new(range: CommitRange) -> Stats {
        Stats {
            list: ListData::new(),
            stats: git_diff_stat(&range),
            range,
        }
    }

    pub fn commit_range(&self) -> &CommitRange {
        &self.range
    }

    pub fn current_stat(&self) -> &Stat {
        let cursor = self.cursor();
        &self.stats[cursor]
    }
}

/// The Widget used to render Stats
pub struct StatsView<'a> {
    stats: &'a RefCell<Stats>,
    block: Option<Block<'a>>,
}

impl<'a> StatsView<'a> {
    pub fn new(stats: &'a RefCell<Stats>) -> StatsView {
        StatsView { stats, block: None }
    }
}

impl<'a> WidgetWithBlock<'a> for StatsView<'a> {
    fn block(&mut self, block: Block<'a>) {
        self.block = Some(block);
    }
}

impl<'a> Widget for StatsView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let height = area.height as usize;
        let mut stats = self.stats.borrow_mut();

        stats.set_list_height(height);

        let adds_width = stats
            .stats
            .iter()
            .map(|s| s.adds.to_string().len())
            .max()
            .unwrap_or(0);
        let dels_width = stats
            .stats
            .iter()
            .map(|s| s.deletes.to_string().len())
            .max()
            .unwrap_or(0);

        let items: Vec<ListItem> = stats
            .stats
            .iter()
            .map(|c| {
                let row = ListItem::new(Spans::from(vec![
                    Span::styled(
                        format!(
                            "{:>width$}",
                            c.adds.to_string(),
                            width = adds_width,
                        ),
                        Style::default().fg(Color::Indexed(2)),
                    ),
                    Span::from(" "),
                    Span::styled(
                        format!(
                            "{:>width$}",
                            c.deletes.to_string(),
                            width = dels_width,
                        ),
                        Style::default().fg(Color::Indexed(1)),
                    ),
                    Span::from(" "),
                    Span::from(c.path.clone()),
                    Span::from(" "),
                ]));

                row
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().bg(Color::Indexed(0)))
            .block(self.block.unwrap());
        StatefulWidget::render(
            list,
            area,
            buf,
            &mut stats.list_state(),
        );
    }
}
