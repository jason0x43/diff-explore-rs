use std::collections::LinkedList;
use std::time::{Duration, Instant};

use crate::error::AppError;
use crate::events::{AppEvent, Events};
use crate::git::DiffAction;
use crate::list::{ListCursor, ListScroll};
use crate::search::Search;
use crate::ui::Ui;
use crate::{
    events::Key,
    stack::Stack,
    views::{
        commitlog::CommitLog,
        diff::Diff,
        stats::Stats,
        statusline::{Status, StatusLine},
    },
};

pub enum View {
    CommitLog(CommitLog),
    Stats(Stats),
    Diff(Diff),
}

pub struct App {
    pub views: LinkedList<View>,
    pub statusline: StatusLine,
    pub tab_width: u8,
    pub search: Option<String>,
    typing_search: bool,
    events: Events,
    should_quit: bool,
    pending_keys: Vec<Key>,
    pending_key_timeout: Duration,
    last_key_time: Instant,
}

impl App {
    pub fn new() -> Result<Self, AppError> {
        let mut views = LinkedList::new();
        let commits = CommitLog::new();
        let status = commits.status();
        views.push(View::CommitLog(commits));

        Ok(Self {
            views,
            should_quit: false,
            statusline: StatusLine::new(status, None),
            tab_width: 4,
            pending_keys: vec![],
            pending_key_timeout: Duration::from_millis(500),
            last_key_time: Instant::now(),
            events: Events::new()?,
            search: None,
            typing_search: false,
        })
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn entering_search(&self) -> Option<String> {
        if self.typing_search {
            self.search.clone()
        } else {
            None
        }
    }

    pub fn do_action(&mut self, key: Key) {
        if self.last_key_time.elapsed() > self.pending_key_timeout {
            self.pending_keys.clear();
        }

        self.last_key_time = Instant::now();

        if let Key::Ctrl('c') = key {
            self.quit();
            return;
        }

        if self.typing_search {
            match key {
                Key::Enter => {
                    self.typing_search = false;
                    match self.views.top_mut() {
                        Some(View::CommitLog(v)) => v.search_next(),
                        Some(View::Stats(v)) => v.search_next(),
                        Some(View::Diff(v)) => v.search_next(),
                        _ => {}
                    }
                }

                Key::Char(c) => {
                    let q = self.search.clone().unwrap_or(String::from(""));
                    self.search = Some(format!("{}{}", q, c));
                }

                Key::Backspace => {
                    if let Some(q) = &mut self.search {
                        if !q.is_empty() {
                            q.truncate(q.len() - 1);
                            self.search = Some(q.clone());
                        }
                    }
                }

                Key::Escape => {
                    self.search = None;
                    self.typing_search = false;
                }

                Key::Ctrl('n') => match self.views.top_mut() {
                    Some(View::CommitLog(v)) => v.search_next(),
                    Some(View::Stats(v)) => v.search_next(),
                    Some(View::Diff(v)) => v.search_next(),
                    _ => {}
                },

                Key::Ctrl('p') => match self.views.top_mut() {
                    Some(View::CommitLog(v)) => v.search_prev(),
                    Some(View::Stats(v)) => v.search_prev(),
                    Some(View::Diff(v)) => v.search_prev(),
                    _ => {}
                },
                _ => {}
            }
        } else if !self.pending_keys.is_empty() {
            let last_key = self.pending_keys.last().unwrap();
            match key {
                Key::Char('G') => match last_key {
                    Key::Char('1') => match self.views.top_mut() {
                        Some(View::CommitLog(v)) => {
                            v.cursor_to_top();
                            self.pending_keys.clear();
                        }

                        Some(View::Diff(v)) => {
                            v.scroll_top();
                            self.pending_keys.clear();
                        }

                        Some(View::Stats(v)) => {
                            v.cursor_to_top();
                            self.pending_keys.clear();
                        }

                        _ => self.pending_keys.clear(),
                    },

                    _ => self.pending_keys.clear(),
                },

                _ => self.pending_keys.clear(),
            }
        } else {
            match key {
                Key::Escape => self.search = None,
                Key::Char('1') => self.pending_keys.push(key),

                Key::Char('l') => {
                    if let Some(View::CommitLog(v)) = self.views.top_mut() {
                        v.toggle_show_details();
                    }
                }

                Key::Char('q') => match self.views.top() {
                    Some(View::CommitLog(_v)) => self.quit(),
                    Some(View::Stats(_v)) => {
                        self.views.pop();
                    }
                    Some(View::Diff(v)) => {
                        if let Ok(p) = v.path() {
                            if let Err(e) = self.events.unwatch_file(&p) {
                                tracing::warn!(
                                    "Error unwatching {:?}: {}",
                                    v.path(),
                                    e
                                )
                            }
                        }
                        self.views.pop();
                    }
                    _ => {}
                },

                Key::Char('/') => {
                    self.search = Some("".into());
                    self.typing_search = true;
                }

                Key::Char('G') => match self.views.top_mut() {
                    Some(View::CommitLog(v)) => v.cursor_to_bottom(),
                    Some(View::Diff(v)) => v.scroll_bottom(),
                    Some(View::Stats(v)) => v.cursor_to_bottom(),
                    _ => {}
                },

                Key::Char('n') => {
                    if self.search.is_some() {
                        match self.views.top_mut() {
                            Some(View::CommitLog(v)) => v.search_next(),
                            Some(View::Diff(v)) => v.search_next(),
                            Some(View::Stats(v)) => v.search_next(),
                            _ => {}
                        }
                    } else if let Some(View::Diff(v)) = self.views.top_mut() {
                        v.toggle_show_line_numbers();
                    }
                }

                Key::Char('N') => {
                    if self.search.is_some() {
                        match self.views.top_mut() {
                            Some(View::CommitLog(v)) => v.search_prev(),
                            Some(View::Diff(v)) => v.search_prev(),
                            Some(View::Stats(v)) => v.search_next(),
                            _ => {}
                        }
                    }
                }

                Key::Char(' ') => match self.views.top_mut() {
                    Some(View::CommitLog(v)) => v.cursor_mark(),
                    Some(View::Diff(v)) => v.page_down(),
                    _ => {}
                },

                Key::Up | Key::Char('k') => match self.views.top_mut() {
                    Some(View::CommitLog(v)) => v.cursor_up(),
                    Some(View::Stats(v)) => v.cursor_up(),
                    Some(View::Diff(v)) => v.scroll_up(),
                    _ => {}
                },

                Key::Down | Key::Char('j') => match self.views.top_mut() {
                    Some(View::CommitLog(v)) => v.cursor_down(),
                    Some(View::Stats(v)) => v.cursor_down(),
                    Some(View::Diff(v)) => v.scroll_down(),
                    _ => {}
                },

                Key::Ctrl('u') => match self.views.top_mut() {
                    Some(View::CommitLog(v)) => v.cursor_page_up(),
                    Some(View::Stats(v)) => v.cursor_page_up(),
                    Some(View::Diff(v)) => v.page_up(),
                    _ => {}
                },

                Key::Ctrl('f') => match self.views.top_mut() {
                    Some(View::CommitLog(v)) => v.cursor_page_down(),
                    Some(View::Stats(v)) => v.cursor_page_down(),
                    Some(View::Diff(v)) => v.page_down(),
                    _ => {}
                },

                Key::Ctrl('c') => self.quit(),

                Key::Char('d') => {
                    if let Some(View::CommitLog(v)) = self.views.top() {
                        let selected = v.get_selected();
                        let marked = v.get_marked();
                        let action = DiffAction::diff(selected, marked);
                        self.views.push(View::Stats(Stats::new(action)));
                    }
                }

                Key::Enter => match self.views.top() {
                    Some(View::CommitLog(v)) => {
                        let selected = v.get_selected();
                        let commits = DiffAction::show(selected);
                        self.views.push(View::Stats(Stats::new(commits)));
                    }

                    Some(View::Stats(v)) => {
                        let stat = v.current_stat().clone();
                        let commits = v.commits().clone();
                        self.views.push(View::Diff(Diff::new(&stat, &commits)));
                        if let Ok(p) = stat.path() {
                            if let Err(e) = self.events.watch_file(&p) {
                                tracing::warn!(
                                    "Error watching {:?}: {}",
                                    stat.path(),
                                    e
                                )
                            } else {
                                tracing::info!("Watching {:?}", stat.path())
                            }
                        } else {
                            tracing::warn!("Error watching {:?}", stat)
                        }
                    }

                    _ => {}
                },

                _ => tracing::debug!("Unhandled: {}", key),
            }
        }
    }

    pub fn start(&mut self) {
        self.events.start();

        let mut ui = Ui::new();

        loop {
            ui.update(self);

            match self.events.next().unwrap() {
                AppEvent::Input(key) => self.do_action(key),
                AppEvent::Resize => {}
                AppEvent::FilesChanged(_) => {
                    if let Some(View::Diff(v)) = self.views.top_mut() {
                        v.refresh();
                    }
                }
            };

            if self.should_quit() {
                break;
            }
        }

        ui.stop();
    }
}
