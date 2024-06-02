use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::{
    git::{git_diff_stat, DiffAction, Stat},
    list::{ListCursor, ListData, ListInfo, ListScroll},
    search::Search,
    ui::highlight_spans,
    views::statusline::Status,
};

#[derive(Debug, Clone)]
pub struct Stats {
    list: ListData,
    commits: DiffAction,
    stats: Vec<Stat>,
    search: Option<String>,
}

impl Stats {
    pub fn new(range: DiffAction) -> Stats {
        Stats {
            list: ListData::new(),
            stats: git_diff_stat(&range, None),
            commits: range,
            search: None,
        }
    }

    pub fn commits(&self) -> &DiffAction {
        &self.commits
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

    fn set_list_pos(&mut self, pos: usize) {
        self.cursor_to(pos);
    }
}

impl ListScroll for Stats {
    fn height(&self) -> usize {
        self.list.height
    }

    fn scroll_to(&mut self, line: usize) {
        self.cursor_to(line);
    }
}

impl ListCursor for Stats {
    fn list_state(&self) -> &ListState {
        &self.list.state
    }

    fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list.state
    }
}

impl Status for Stats {
    fn status(&self) -> String {
        format!("{}", self.commits)
    }
}

impl Search for Stats {
    fn set_search(&mut self, search: Option<String>) {
        self.search = search;
    }

    fn get_search(&self) -> Option<String> {
        self.search.clone()
    }

    fn is_match(&self, idx: usize) -> bool {
        match &self.search {
            Some(search) => {
                let stat = &self.stats[idx];
                stat.path.contains(search)
                    || stat.adds.to_string().contains(search)
                    || stat.deletes.to_string().contains(search)
            }
            _ => false,
        }
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

impl<'a> Widget for StatsView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.stats.list.height = area.height as usize;

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
                let mut spans = vec![
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
                ];

                if let Some(search) = &self.stats.search {
                    spans = highlight_spans(
                        spans.clone(),
                        search,
                        Style::default().add_modifier(Modifier::REVERSED),
                    )
                }

                let row = ListItem::new(Line::from(spans));

                row
            })
            .collect();

        let mut list = List::new(items)
            .highlight_style(Style::default().bg(Color::Indexed(0)));

        if let Some(b) = self.block {
            list = list.block(b);
        }

        StatefulWidget::render(list, area, buf, self.stats.list_state_mut());
    }
}
