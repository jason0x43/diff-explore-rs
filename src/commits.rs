use std::collections::HashSet;

use tui::{
    style::{Color, Style},
    widgets::{List, ListItem},
};

#[derive(Debug, Clone)]
pub struct Commits {
    commits: Vec<String>,
    cursor: usize,
    marks: HashSet<usize>,
}

impl Commits {
    pub fn new(commits: Vec<String>) -> Commits {
        Commits {
            commits,
            cursor: 0,
            marks: HashSet::new(),
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

    pub fn to_widget<'a>(&self) -> List<'a> {
        let items: Vec<ListItem> = self
            .commits
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let style = if i == self.cursor {
                    Style::default().bg(Color::Indexed(0))
                } else {
                    Style::default()
                };
                let prefix = if self.marks.contains(&i) {
                    ">"
                } else {
                    " "
                };
                ListItem::new(format!("{}{}", prefix, c)).style(style)
            })
            .collect();
        List::new(items)
    }
}
