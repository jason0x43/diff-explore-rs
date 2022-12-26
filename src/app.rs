use std::collections::LinkedList;

use crate::console;
use crate::statusline::{Status, StatusLine};
use crate::{
    commits::Commits, console::Console, diff::Diff, events::Key, stack::Stack,
    stats::Stats,
};

use list_helper_core::ListCursor;

pub enum View {
    Commits(Commits),
    Stats(Stats),
    Diff(Diff),
}

pub struct App {
    pub views: LinkedList<View>,
    pub console: Console,
    pub statusline: StatusLine,
    should_quit: bool,
    show_console: bool,
}

impl App {
    pub fn new() -> Self {
        let mut views = LinkedList::new();
        let commits = Commits::new();
        let status = commits.status();
        views.push(View::Commits(commits));

        Self {
            views,
            should_quit: false,
            console: Console::new(),
            show_console: false,
            statusline: StatusLine::new(status, None),
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
            Key::Char('q') => match self.views.top() {
                Some(View::Commits(_v)) => {
                    self.quit();
                }
                Some(View::Stats(_v)) => {
                    self.views.pop();
                }
                Some(View::Diff(_v)) => {
                    self.views.pop();
                }
                _ => {}
            },
            Key::Char('>') => self.toggle_console(),
            Key::Space => match self.views.top() {
                Some(View::Commits(v)) => {
                    v.cursor_mark();
                }
                Some(View::Diff(v)) => {
                    v.page_down();
                }
                _ => {}
            },
            Key::Up | Key::Char('k') => match self.views.top() {
                Some(View::Commits(v)) => {
                    v.cursor_up();
                }
                Some(View::Stats(v)) => {
                    v.cursor_up();
                }
                Some(View::Diff(v)) => {
                    v.scroll_up();
                }
                _ => {}
            },
            Key::Down | Key::Char('j') => match self.views.top() {
                Some(View::Commits(v)) => {
                    v.cursor_down();
                }
                Some(View::Stats(v)) => {
                    v.cursor_down();
                }
                Some(View::Diff(v)) => {
                    v.scroll_down();
                }
                _ => {}
            },
            Key::Ctrl('u') => match self.views.top() {
                Some(View::Commits(v)) => {
                    v.cursor_page_up();
                }
                Some(View::Stats(v)) => {
                    v.cursor_page_up();
                }
                Some(View::Diff(v)) => {
                    v.page_up();
                }
                _ => {}
            },
            Key::Ctrl('f') => match self.views.top() {
                Some(View::Commits(v)) => {
                    v.cursor_page_down();
                }
                Some(View::Stats(v)) => {
                    v.cursor_page_down();
                }
                Some(View::Diff(v)) => {
                    v.page_down();
                }
                _ => {}
            },
            Key::Ctrl('n') => {
                if self.show_console {
                    self.console.scroll_down();
                }
            }
            Key::Ctrl('p') => {
                if self.show_console {
                    self.console.scroll_up();
                }
            }
            Key::Ctrl('c') => self.quit(),
            Key::Enter => match self.views.top() {
                Some(View::Commits(v)) => {
                    let range = v.get_range();
                    self.views.push(View::Stats(Stats::new(range)));
                }
                Some(View::Stats(v)) => {
                    let stat = v.current_stat().clone();
                    let range = v.commit_range().clone();
                    self.views.push(View::Diff(Diff::new(&stat, &range)));
                }
                _ => {}
            },
            _ => {
                console!("Unhandled: {}", key);
            }
        }
    }
}
