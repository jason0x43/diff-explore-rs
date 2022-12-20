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
    console::console_log,
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
    pub fn new() -> Stats {
        Stats {
            list: ListData::new(),
            range: CommitRange::default(),
            stats: vec![],
        }
    }

    pub fn set_range(&mut self, range: CommitRange) {
        console_log(&format!("set range to {}", range));
        self.stats = git_diff_stat(&range);
        self.range = range;
    }
}

/// The Widget used to render Stats
pub struct StatsList<'a> {
    stats: &'a mut Stats,
    block: Option<Block<'a>>,
}

impl<'a> StatsList<'a> {
    pub fn new(stats: &mut Stats) -> StatsList {
        StatsList { stats, block: None }
    }
}

impl<'a> WidgetWithBlock<'a> for StatsList<'a> {
    fn block(&mut self, block: Block<'a>) {
        self.block = Some(block);
    }
}

impl<'a> Widget for StatsList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let height = area.height as usize;

        self.stats.set_list_height(height);

        let stats: &Vec<Stat> = &self.stats.stats;

        let adds_width = stats
            .iter()
            .map(|s| s.adds.to_string().len())
            .max()
            .unwrap_or(0);
        let dels_width = stats
            .iter()
            .map(|s| s.deletes.to_string().len())
            .max()
            .unwrap_or(0);

        let items: Vec<ListItem> = stats
            .iter()
            .enumerate()
            .map(|(_i, c)| {
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
        StatefulWidget::render(list, area, buf, &mut self.stats.list_state());
    }
}
