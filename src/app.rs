use std::collections::LinkedList;
use std::time::{Duration, Instant};

use crate::console;
use crate::events::{AppEvent, Events};
use crate::ui::Ui;
use crate::{
    events::Key,
    stack::Stack,
    views::{
        commits::Commits,
        console::Console,
        diff::Diff,
        stats::Stats,
        statusline::{Status, StatusLine},
    },
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
    pub tab_width: u8,
    events: Events,
    should_quit: bool,
    show_console: bool,
    pending_keys: Vec<Key>,
    pending_key_timeout: u64,
    last_key_time: Instant,
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
            tab_width: 4,
            pending_keys: vec![],
            pending_key_timeout: 500,
            last_key_time: Instant::now(),
            events: Events::new(),
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
        let now = std::time::Instant::now();
        if now - self.last_key_time
            > Duration::from_millis(self.pending_key_timeout)
        {
            self.pending_keys.clear();
        }

        self.last_key_time = now;

        if self.pending_keys.len() > 0 {
            let last_key = self.pending_keys.last().unwrap();
            match key {
                Key::Char('G') => match last_key {
                    Key::Char('1') => match self.views.top() {
                        Some(View::Commits(v)) => {
                            v.cursor_to_top();
                            self.pending_keys.clear();
                        }
                        Some(View::Diff(v)) => {
                            v.scroll_to_top();
                            self.pending_keys.clear();
                        }
                        Some(View::Stats(v)) => {
                            v.cursor_to_top();
                            self.pending_keys.clear();
                        }
                        _ => {
                            self.pending_keys.clear();
                        }
                    },
                    _ => {
                        self.pending_keys.clear();
                    }
                },
                _ => {
                    self.pending_keys.clear();
                }
            }
        } else {
            match key {
                Key::Char('1') => {
                    self.pending_keys.push(key);
                }
                Key::Char('q') => match self.views.top() {
                    Some(View::Commits(_v)) => {
                        self.quit();
                    }
                    Some(View::Stats(_v)) => {
                        self.views.pop();
                    }
                    Some(View::Diff(v)) => {
                        self.events.unwatch_file(&v.path());
                        self.views.pop();
                    }
                    _ => {}
                },
                Key::Char('G') => match self.views.top() {
                    Some(View::Commits(v)) => {
                        v.cursor_to_bottom();
                    }
                    Some(View::Diff(v)) => {
                        v.scroll_to_bottom();
                    }
                    Some(View::Stats(v)) => {
                        v.cursor_to_bottom();
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
                        self.events.watch_file(&stat.path());
                    }
                    _ => {}
                },
                _ => {
                    console!("Unhandled: {}", key);
                }
            }
        }
    }

    pub fn start(&mut self) {
        self.events.start();

        let mut ui = Ui::new();

        loop {
            ui.update(self);

            match self.events.next().unwrap() {
                AppEvent::Input(key) => {
                    self.do_action(key);
                }
                AppEvent::Resize => {}
                AppEvent::FilesChanged(_) => {
                    if let Some(View::Diff(v)) = self.views.top() {
                        v.refresh();
                    }
                }
            };

            if self.should_quit() {
                break;
            }
        }
    }
}
