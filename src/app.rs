use crate::{
    commits::Commits, console::console_log, events::Key, git::git_log,
    stats::Stats,
};

use list_helper_core::ListCursor;

pub enum View {
    Commits,
    Stats,
    // Diff
}

pub struct App {
    pub view: View,
    pub commits: Commits,
    pub stats: Stats,
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
            commits: Commits::new(),
            stats: Stats::new(),
            cursor: 0,
            height: 0,
            history: vec![],
            should_quit: false,
            show_console: false,
            width: 0,
        }
    }

    pub fn load_commits(&mut self) {
        let entries = git_log().expect("unable to load git log");
        let entry_list = entries;
        for entry in entry_list {
            self.commits.add(entry)
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
            Key::Char('q') => match self.view {
                View::Commits => self.quit(),
                View::Stats => {
                    self.view = View::Commits;
                }
            },
            Key::Char('>') => self.toggle_console(),
            Key::Space => match self.view {
                View::Commits => {
                    self.commits.cursor_mark();
                }
                _ => {}
            },
            Key::Up | Key::Char('k') => match self.view {
                View::Commits => self.commits.cursor_up(),
                View::Stats => self.stats.cursor_up(),
            },
            Key::Down | Key::Char('j') => match self.view {
                View::Commits => self.commits.cursor_down(),
                View::Stats => self.stats.cursor_down(),
            },
            Key::Ctrl('u') => match self.view {
                View::Commits => self.commits.cursor_page_up(),
                View::Stats => self.stats.cursor_page_up(),
            },
            Key::Ctrl('f') => match self.view {
                View::Commits => self.commits.cursor_page_down(),
                View::Stats => self.stats.cursor_page_down(),
            },
            Key::Ctrl('c') => self.quit(),
            Key::Enter => match self.view {
                View::Commits => {
                    self.stats.set_range(self.commits.get_range());
                    self.view = View::Stats;
                }
                _ => {}
            },
            _ => {
                console_log(&format!("Unhandled: {}", key));
            }
        }
    }
}
