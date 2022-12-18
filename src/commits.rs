use std::{cmp::min, collections::HashSet};

use chrono::{Datelike, NaiveDateTime, Timelike, Utc};
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::util::Truncatable;
use crate::{git::Commit, messages::Message};

#[derive(Debug, Clone)]
pub struct Commits {
    commits: Vec<Commit>,
    // offset: usize,
    marks: HashSet<usize>,
    list_state: ListState,
    list_height: usize,
    pub messages: Vec<Message>,
}

impl Commits {
    pub fn new(commits: Vec<Commit>) -> Commits {
        let mut state = ListState::default();
        state.select(Some(0));

        Commits {
            commits,
            // offset: 0,
            marks: HashSet::new(),
            list_state: state,
            list_height: 0,
            messages: vec![],
        }
    }

    fn cursor(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    pub fn add(&mut self, commit: Commit) {
        self.commits.push(commit);
    }

    pub fn cursor_down(&mut self) {
        let cursor = min(self.cursor() + 1, self.commits.len() - 1);
        self.list_state.select(Some(cursor));
    }

    pub fn cursor_page_down(&mut self) {
        let cursor =
            min(self.cursor() + self.list_height, self.commits.len() - 1);
        self.list_state.select(Some(cursor));
    }

    pub fn cursor_up(&mut self) {
        let cursor = self.cursor();
        let delta = min(self.cursor(), 1);
        self.list_state.select(Some(cursor - delta));
    }

    pub fn cursor_page_up(&mut self) {
        let cursor = self.cursor();
        let delta = min(cursor, self.list_height);
        self.list_state.select(Some(cursor - delta));
    }

    pub fn cursor_mark(&mut self) {
        let cursor = self.cursor();
        if self.marks.contains(&cursor) {
            self.marks.remove(&cursor);
        } else {
            self.marks.insert(cursor);
        }
    }
}

/// The Widget used to render Commits
pub struct CommitsList<'a> {
    commits: &'a mut Commits,
    block: Option<Block<'a>>,
}

impl<'a> CommitsList<'a> {
    pub fn new(commits: &'a mut Commits) -> CommitsList<'a> {
        CommitsList {
            commits,
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> CommitsList<'a> {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for CommitsList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = area.width as usize;
        let height = area.height as usize;

        self.commits.list_height = height;

        let items: Vec<ListItem> = self
            .commits
            .commits
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let prefix = if self.commits.marks.contains(&i) {
                    "▶"
                } else {
                    " "
                };

                let ctime = c.timestamp;
                let time =
                    NaiveDateTime::from_timestamp_opt(ctime as i64, 0).unwrap();
                let now = Utc::now().naive_utc();
                let age = if time.year() != now.year() {
                    format!("{:>2}Y", now.year() - time.year())
                } else if time.month() != now.month() {
                    format!("{:>2}M", now.month() - time.month())
                } else if time.day() != now.day() {
                    format!("{:>2}D", now.day() - time.day())
                } else if time.hour() != now.hour() {
                    format!("{:>2}h", now.hour() - time.hour())
                } else if time.minute() != now.minute() {
                    format!("{:>2}m", now.minute() - time.minute())
                } else {
                    format!("{:>2}s", now.second() - time.second())
                };

                // Truncate the subject if it's longer than the available space, which is (width -
                // (sum of column widths) - (sum of column gaps))
                let last_col = width - (1 + 8 + 3) - (1 + 1 + 1);
                let subject = c.subject.ellipses(last_col);

                let row = ListItem::new(Spans::from(vec![
                    Span::from(prefix),
                    Span::from(" "),
                    Span::styled(
                        format!("{}", &c.commit[..8]),
                        Style::default().fg(Color::Indexed(5)),
                    ),
                    Span::from(" "),
                    Span::styled(age, Style::default().fg(Color::Indexed(4))),
                    Span::from(" "),
                    Span::from(subject),
                ]));

                row
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().bg(Color::Indexed(0)))
            .block(self.block.unwrap());
        StatefulWidget::render(list, area, buf, &mut self.commits.list_state);
    }
}
