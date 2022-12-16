#[derive(Debug, Clone, Default)]
pub struct CommitsState {
    // offset: usize,
    cursor: usize,
}

use tui::widgets::{List, ListItem};

#[derive(Debug, Clone)]
pub struct Commits {
    commits: Vec<String>,
    state: CommitsState,
}

impl Commits {
    pub fn new(commits: Vec<String>) -> Commits {
        Commits {
            commits,
            state: CommitsState::default(),
        }
    }

    pub fn add(&mut self, commit: String) {
        self.commits.push(commit);
    }

    pub fn cursor_down(&mut self) {
        if self.state.cursor + 1 < self.commits.len() {
            self.state.cursor += 1;
        }
    }

    pub fn cursor_up(&mut self) {
        if self.state.cursor > 0 {
            self.state.cursor -= 1;
        }
    }

    pub fn to_widget<'a>(&self) -> List<'a> {
        let items: Vec<ListItem> = self
            .commits
            .iter()
            .map(|c| ListItem::new(c.clone()))
            .collect();
        List::new(items)
    }
}
