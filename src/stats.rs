use list_helper_core::{ListCursor, ListData, ListInfo};
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
    statusline::Status,
    widget::WidgetWithBlock,
};

#[derive(Debug, Clone, ListCursor)]
pub struct Stats {
    list: ListData,
    range: CommitRange,
    stats: Vec<Stat>,
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

impl ListInfo for Stats {
    fn list_count(&self) -> usize {
        self.stats.len()
    }

    fn list_pos(&self) -> usize {
        self.cursor()
    }
}

impl Status for Stats {
    fn status(&self) -> String {
        format!("{}", self.range)
    }
}

/// The Widget used to render Stats
pub struct StatsView<'a> {
    stats: &'a mut Stats,
    block: Option<Block<'a>>,
}

impl<'a> StatsView<'a> {
    pub fn new(stats: &'a mut Stats) -> StatsView {
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

        self.stats.set_list_height(height);

        let adds_width = self
            .stats
            .stats
            .iter()
            .map(|s| s.adds.to_string().len())
            .max()
            .unwrap_or(0);
        let dels_width = self
            .stats
            .stats
            .iter()
            .map(|s| s.deletes.to_string().len())
            .max()
            .unwrap_or(0);

        let items: Vec<ListItem> = self
            .stats
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

        let mut list = List::new(items)
            .highlight_style(Style::default().bg(Color::Indexed(0)));

        if let Some(b) = self.block {
            list = list.block(b);
        }

        StatefulWidget::render(list, area, buf, self.stats.list_state());
    }
}
