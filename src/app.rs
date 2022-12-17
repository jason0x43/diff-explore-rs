use crate::{commits::Commits, events::Key, git::git_log};

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
    show_console: bool,
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
            show_console: false,
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

    pub fn toggle_console(&mut self) {
        self.show_console = !self.show_console;
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn should_show_console(&self) -> bool {
        self.show_console
    }

    pub fn do_action(&mut self, key: Key) {
        match key {
            Key::Char('q') => self.quit(),
            Key::Char('>') => self.toggle_console(),
            Key::Space => match self.view {
                View::Commits => {
                    self.commits.cursor_mark();
                }
            },
            Key::Up | Key::Char('k') => match self.view {
                View::Commits => self.commits.cursor_up(),
            },
            Key::Down | Key::Char('j') => match self.view {
                View::Commits => self.commits.cursor_down(),
            },
            _ => {
                self.messages.push(format!("Unhandled: {}", key));
            }
        }
    }
}
