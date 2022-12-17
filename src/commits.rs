use std::collections::HashSet;

use tui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Cell, Row, Table},
};

use crate::{git::Commit, messages::Message};

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

    pub fn to_widget<'a>(&mut self, r: Rect) -> Table<'a> {
        let height = r.height as usize;
        let width = r.width as usize;

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
                ">"
            } else {
                " "
            };

            let mut subject = c.subject.clone();
            let last_col = width - 11;
            if subject.len() > last_col - 1 {
                subject = format!("{}...", &subject[0..last_col - 5]);
            }

            let mut row = Row::new(vec![
                Cell::from(prefix),
                Cell::from(c.commit.clone())
                    .style(Style::default().fg(Color::Indexed(5))),
                Cell::from(subject),
            ]);

            if real_i == self.cursor {
                row = row.style(Style::default().bg(Color::Indexed(0)));
            }

            row
        }))
        .widths(&[
            Constraint::Length(1),
            Constraint::Length(8),
            Constraint::Percentage(100),
        ])
    }
}
