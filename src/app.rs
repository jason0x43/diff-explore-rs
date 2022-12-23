use std::{cell::RefCell, collections::LinkedList};

use crate::{
    commits::Commits, console, diff::Diff, events::Key, git::git_log,
    stack::Stack, stats::Stats,
};

use list_helper_core::ListCursor;

pub enum View {
    Commits(RefCell<Commits>),
    Stats(RefCell<Stats>),
    Diff(RefCell<Diff>),
}

pub struct App {
    pub views: LinkedList<View>,
    pub cursor: u32,
    pub height: u32,
    pub history: Vec<String>,
    pub width: u32,
    should_quit: bool,
    show_console: bool,
}

impl App {
    pub fn new() -> Self {
        let commits = git_log();
        let mut views = LinkedList::new();
        views.push(View::Commits(RefCell::new(Commits::new(commits))));

        Self {
            views,
            cursor: 0,
            height: 0,
            history: vec![],
            should_quit: false,
            show_console: false,
            width: 0,
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
                    v.borrow_mut().cursor_mark();
                }
                Some(View::Diff(v)) => {
                    v.borrow_mut().page_down();
                }
                _ => {}
            },
            Key::Up | Key::Char('k') => match self.views.top() {
                Some(View::Commits(v)) => {
                    v.borrow_mut().cursor_up();
                }
                Some(View::Stats(v)) => {
                    v.borrow_mut().cursor_up();
                }
                Some(View::Diff(v)) => {
                    v.borrow_mut().scroll_up();
                }
                _ => {}
            },
            Key::Down | Key::Char('j') => match self.views.top() {
                Some(View::Commits(v)) => {
                    v.borrow_mut().cursor_down();
                }
                Some(View::Stats(v)) => {
                    v.borrow_mut().cursor_down();
                }
                Some(View::Diff(v)) => {
                    v.borrow_mut().scroll_down();
                }
                _ => {}
            },
            Key::Ctrl('u') => match self.views.top() {
                Some(View::Commits(v)) => {
                    v.borrow_mut().cursor_page_up();
                }
                Some(View::Stats(v)) => {
                    v.borrow_mut().cursor_page_up();
                }
                Some(View::Diff(v)) => {
                    v.borrow_mut().page_up();
                }
                _ => {}
            },
            Key::Ctrl('f') => match self.views.top() {
                Some(View::Commits(v)) => {
                    v.borrow_mut().cursor_page_down();
                }
                Some(View::Stats(v)) => {
                    v.borrow_mut().cursor_page_down();
                }
                Some(View::Diff(v)) => {
                    v.borrow_mut().page_down();
                }
                _ => {}
            },
            Key::Ctrl('c') => self.quit(),
            Key::Enter => match self.views.top() {
                Some(View::Commits(v)) => {
                    let range = v.borrow_mut().get_range();
                    self.views
                        .push(View::Stats(RefCell::new(Stats::new(range))));
                }
                Some(View::Stats(v)) => {
                    let stat = v.borrow().current_stat().clone();
                    let range = v.borrow().commit_range().clone();
                    self.views.push(View::Diff(RefCell::new(Diff::new(
                        &stat, &range,
                    ))));
                }
                _ => {}
            },
            _ => {
                console!("Unhandled: {}", key);
            }
        }
    }
}
