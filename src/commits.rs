use std::collections::HashSet;

use tui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{List, ListItem},
};

use crate::messages::Message;

#[derive(Debug, Clone)]
pub struct Commits {
    commits: Vec<String>,
    cursor: usize,
    offset: usize,
    marks: HashSet<usize>,
    pub messages: Vec<Message>,
}

impl Commits {
    pub fn new(commits: Vec<String>) -> Commits {
        Commits {
            commits,
            cursor: 0,
            offset: 0,
            marks: HashSet::new(),
            messages: vec![],
        }
    }

    pub fn add(&mut self, commit: String) {
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

    pub fn to_widget<'a>(&mut self, r: Rect) -> List<'a> {
        let height = r.height as usize;

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

        let items: Vec<ListItem> = self.commits[start..end]
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let real_i = i + start;
                let style = if real_i == self.cursor {
                    Style::default().bg(Color::Indexed(0))
                } else {
                    Style::default()
                };
                let prefix = if self.marks.contains(&real_i) { ">" } else { " " };
                ListItem::new(format!("{}{}", prefix, c)).style(style)
            })
            .collect();
        List::new(items)
    }
}
