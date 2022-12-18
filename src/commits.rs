use std::collections::HashSet;

use chrono::{Datelike, NaiveDateTime, Timelike, Utc};
use tui::{
    layout::Constraint,
    style::{Color, Style},
    widgets::{Cell, Row, Table},
};

use crate::util::Truncatable;
use crate::{git::Commit, messages::Message, util::Dimensions};

#[derive(Debug, Clone)]
pub struct Commits {
    commits: Vec<Commit>,
    cursor: usize,
    offset: usize,
    marks: HashSet<usize>,
    pub messages: Vec<Message>,
}

impl Commits {
    pub fn new(commits: Vec<Commit>) -> Commits {
        Commits {
            commits,
            cursor: 0,
            offset: 0,
            marks: HashSet::new(),
            messages: vec![],
        }
    }

    pub fn add(&mut self, commit: Commit) {
        self.commits.push(commit);
    }

    pub fn cursor_down(&mut self) {
        if self.cursor + 1 < self.commits.len() {
            self.cursor += 1;
        }
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_mark(&mut self) {
        if self.marks.contains(&self.cursor) {
            self.marks.remove(&self.cursor);
        } else {
            self.marks.insert(self.cursor);
        }
    }

    pub fn to_widget<'a>(&mut self, d: Dimensions) -> Table<'a> {
        let height = d.height as usize;
        let width = d.width as usize;

        if self.cursor > self.offset {
            if self.cursor - self.offset > height - 1 {
                self.offset = self.cursor - height + 1
            }
        } else {
            self.offset = self.cursor
        }

        let start = if self.cursor > self.offset {
            self.offset
        } else {
            self.cursor
        };

        let end = if start + height < self.commits.len() {
            start + height
        } else {
            self.commits.len()
        };

        Table::new(self.commits[start..end].iter().enumerate().map(|(i, c)| {
            let real_i = i + start;

            let prefix = if self.marks.contains(&real_i) {
                "▶"
            } else {
                " "
            };

            let ctime = c.timestamp;
            let time =
                NaiveDateTime::from_timestamp_opt(ctime as i64, 0).unwrap();
            let now = Utc::now().naive_utc();
            let age = if time.year() != now.year() {
                self.messages.push(Message::new(format!("time.year: {}", time.year())));
                self.messages.push(Message::new(format!("now.year: {}", now.year())));
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

            // Truncate the subject if it's longer than the available space, which is (width - (sum
            // of column widths) - (sum of column gaps))
            let last_col = width - (1 + 8 + 3) - (1 + 1 + 1);
            let subject = c.subject.ellipses(last_col);

            let mut row = Row::new(vec![
                Cell::from(prefix),
                Cell::from(c.commit.clone())
                    .style(Style::default().fg(Color::Indexed(5))),
                Cell::from(age)
                    .style(Style::default().fg(Color::Indexed(4))),
                Cell::from(subject),
            ]);

            // Highlight the cursor row
            if real_i == self.cursor {
                row = row.style(Style::default().bg(Color::Indexed(0)));
            }

            row
        }))
        .widths(&[
            Constraint::Length(1),
            Constraint::Length(8),
            Constraint::Length(3),
            // Percentage has a lower priority than Length, so Percentage(100) will consume any
            // remaining space
            Constraint::Percentage(100),
        ])
    }
}
