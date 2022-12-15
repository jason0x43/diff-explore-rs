use crate::{commits::Commits, git::git_log};

pub enum View {
    Commits,
    // Stats,
    // Diff
}

pub struct App {
    pub view: View,
    pub commits: Commits,
    pub messages: Vec<String>,
    pub cursor: u32,
    pub height: u32,
    pub history: Vec<String>,
    pub width: u32,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            view: View::Commits,
            commits: Commits::new(vec![]),
            cursor: 0,
            height: 0,
            history: vec![],
            messages: vec![],
            should_quit: false,
            width: 0,
        }
    }

    pub fn load_commits(&mut self) {
        let entries = git_log().expect("unable to load git log");
        let entry_list = entries;
        for entry in entry_list {
            self.commits.add(entry.subject.clone())
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}
