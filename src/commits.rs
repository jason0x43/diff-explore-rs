use chrono::{Datelike, NaiveDateTime, Timelike, Utc};
use list_helper_core::{HasListCount, ListCursor, ListData};
use list_helper_macro::ListCursor;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::git::{Commit, CommitRange, Decoration};
use crate::{util::Truncatable, widget::WidgetWithBlock};

#[derive(Debug, Clone, ListCursor)]
pub struct Commits {
    list: ListData,
    commits: Vec<Commit>,
    mark: Option<usize>,
}

impl HasListCount for Commits {
    fn list_count(&self) -> usize {
        self.commits.len()
    }
}

impl Commits {
    pub fn new(commits: Vec<Commit>) -> Commits {
        Commits {
            list: ListData::new(),
            mark: None,
            commits,
        }
    }

    pub fn cursor_mark(&mut self) {
        let cursor = self.cursor();
        match self.mark {
            None => {
                self.mark = Some(cursor);
            }
            _ => {
                self.mark = None;
            }
        }
    }

    pub fn get_range(&self) -> CommitRange {
        let cursor = self.cursor();
        let c = &self.commits;
        let start = match self.mark {
            Some(mark) => {
                if mark > cursor {
                    c[mark].commit.clone()
                } else {
                    c[cursor].commit.clone()
                }
            }
            _ => c[cursor].commit.clone(),
        };
        let end = match self.mark {
            Some(mark) => {
                if mark > cursor {
                    Some(c[cursor].commit.clone())
                } else {
                    Some(c[mark].commit.clone())
                }
            }
            _ => None,
        };

        CommitRange { start, end }
    }
}

/// The Widget used to render Commits
pub struct CommitsView<'a> {
    commits: &'a mut Commits,
    block: Option<Block<'a>>,
}

impl<'a> CommitsView<'a> {
    pub fn new(commits: &'a mut Commits) -> CommitsView<'a> {
        CommitsView {
            commits,
            block: None,
        }
    }
}

impl<'a> WidgetWithBlock<'a> for CommitsView<'a> {
    fn block(&mut self, block: Block<'a>) {
        self.block = Some(block);
    }
}

fn relative_time(c: &Commit) -> String {
    let ctime = c.timestamp;
    let time = NaiveDateTime::from_timestamp_opt(ctime as i64, 0).unwrap();
    let now = Utc::now().naive_utc();
    if time.year() != now.year() {
        format!("{}Y", now.year() - time.year())
    } else if time.month() != now.month() {
        format!("{}M", now.month() - time.month())
    } else if time.day() != now.day() {
        format!("{}D", now.day() - time.day())
    } else if time.hour() != now.hour() {
        format!("{}h", now.hour() - time.hour())
    } else if time.minute() != now.minute() {
        format!("{}m", now.minute() - time.minute())
    } else {
        format!("{}s", now.second() - time.second())
    }
}

impl<'a> Widget for CommitsView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = area.width as usize;
        let height = area.height as usize;

        self.commits.set_list_height(height);

        let author_width = self
            .commits
            .commits
            .iter()
            .max_by(|x, y| x.author_name.len().cmp(&y.author_name.len()))
            .unwrap()
            .author_name
            .len();

        let time_width = relative_time(
            self.commits
                .commits
                .iter()
                .max_by(|x, y| {
                    relative_time(x).len().cmp(&relative_time(y).len())
                })
                .unwrap(),
        )
        .len();

        let items: Vec<ListItem> = self
            .commits
            .commits
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let prefix = match self.commits.mark {
                    Some(mark) => {
                        if mark == i {
                            "▶"
                        } else {
                            " "
                        }
                    }
                    _ => " ",
                };

                // Determine column widths
                let cols = [1, 8, time_width, author_width];
                let sized_cols: usize = cols.iter().sum();
                let all_gaps: usize = cols.len();
                let last_col = width - sized_cols - all_gaps;

                let age = relative_time(c);
                let author = c.author_name.ellipses(cols[3]);
                let subject = c.subject.ellipses(last_col);

                let mut item: Vec<Span> = vec![
                    Span::from(prefix),
                    Span::from(" "),
                    Span::styled(
                        format!("{}", &c.commit[..cols[1]]),
                        Style::default().fg(Color::Indexed(5)),
                    ),
                    Span::from(" "),
                    Span::styled(age, Style::default().fg(Color::Indexed(4))),
                    Span::from(" "),
                    Span::styled(
                        author,
                        Style::default().fg(Color::Indexed(2)),
                    ),
                    Span::from(" "),
                ];

                let deco = Decoration::from_commit(c);
                if let Some(head) = deco.head {
                    item.push(Span::styled(
                        format!("[{}]", head),
                        Style::default()
                            .fg(Color::Indexed(6))
                            .add_modifier(Modifier::BOLD),
                    ));
                    item.push(Span::from(" "));
                }
                deco.branches.iter().for_each(|b| {
                    item.push(Span::styled(
                        format!("[{}]", b),
                        Style::default().fg(Color::Indexed(6)),
                    ));
                    item.push(Span::from(" "));
                });
                deco.tags.iter().for_each(|t| {
                    item.push(Span::styled(
                        format!("<{}>", t),
                        Style::default().fg(Color::Indexed(5)),
                    ));
                    item.push(Span::from(" "));
                });
                deco.refs.iter().for_each(|r| {
                    item.push(Span::styled(
                        format!("<{}>", r),
                        Style::default().fg(Color::Indexed(3)),
                    ));
                    item.push(Span::from(" "));
                });

                item.push(Span::from(subject));

                ListItem::new(Spans::from(item))
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().bg(Color::Indexed(0)))
            .block(self.block.unwrap());

        StatefulWidget::render(list, area, buf, self.commits.list_state());
    }
}
