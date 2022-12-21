use std::cell::RefCell;

use chrono::{Datelike, NaiveDateTime, Timelike, Utc};
use list_helper_core::{HasListCount, ListCursor, ListData};
use list_helper_macro::ListCursor;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::git::{Commit, CommitRange};
use crate::{util::Truncatable, widget::WidgetWithBlock};

#[derive(Debug, Clone, ListCursor)]
pub struct Commits {
    list: ListData,
    commits: Vec<Commit>,
    mark: Option<usize>,
}

impl HasListCount for Commits {
    fn list_count(&self) -> usize {
        self.commits.len()
    }
}

impl Commits {
    pub fn new(commits: Vec<Commit>) -> Commits {
        Commits {
            list: ListData::new(),
            mark: None,
            commits,
        }
    }

    pub fn cursor_mark(&mut self) {
        let cursor = self.cursor();
        match self.mark {
            None => {
                self.mark = Some(cursor);
            }
            _ => {
                self.mark = None;
            }
        }
    }

    pub fn get_range(&self) -> CommitRange {
        let cursor = self.cursor();
        let c = &self.commits;
        let start = match self.mark {
            Some(mark) => {
                if mark > cursor {
                    c[mark].commit.clone()
                } else {
                    c[cursor].commit.clone()
                }
            }
            _ => c[cursor].commit.clone(),
        };
        let end = match self.mark {
            Some(mark) => {
                if mark > cursor {
                    Some(c[cursor].commit.clone())
                } else {
                    Some(c[mark].commit.clone())
                }
            }
            _ => None,
        };

        CommitRange { start, end }
    }
}

/// The Widget used to render Commits
pub struct CommitsView<'a> {
    commits: &'a RefCell<Commits>,
    block: Option<Block<'a>>,
}

impl<'a> CommitsView<'a> {
    pub fn new(commits: &'a RefCell<Commits>) -> CommitsView<'a> {
        CommitsView {
            commits,
            block: None,
        }
    }
}

impl<'a> WidgetWithBlock<'a> for CommitsView<'a> {
    fn block(&mut self, block: Block<'a>) {
        self.block = Some(block);
    }
}

impl<'a> Widget for CommitsView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = area.width as usize;
        let height = area.height as usize;
        let mut commits = self.commits.borrow_mut();

        commits.set_list_height(height);

        let items: Vec<ListItem> = commits
            .commits
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let prefix = match commits.mark {
                    Some(mark) => {
                        if mark == i {
                            "▶"
                        } else {
                            " "
                        }
                    }
                    _ => " ",
                };

                // Determine column widths
                let cols = [1, 8, 3, 20];
                let sized_cols: usize = cols.iter().sum();
                let all_gaps: usize = cols.len();
                let last_col = width - sized_cols - all_gaps;

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

                let author = c.author_name.ellipses(cols[3]);

                // Truncate the subject if it's longer than the available space, which is (width -
                // (sum of column widths) - (sum of column gaps))
                let subject = c.subject.ellipses(last_col);

                let row = ListItem::new(Spans::from(vec![
                    Span::from(prefix),
                    Span::from(" "),
                    Span::styled(
                        format!("{}", &c.commit[..cols[1]]),
                        Style::default().fg(Color::Indexed(5)),
                    ),
                    Span::from(" "),
                    Span::styled(age, Style::default().fg(Color::Indexed(4))),
                    Span::from(" "),
                    Span::styled(
                        format!("{:<20}", author),
                        Style::default().fg(Color::Indexed(2)),
                    ),
                    Span::from(" "),
                    Span::from(subject),
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
            commits.list_state(),
        );
    }
}
