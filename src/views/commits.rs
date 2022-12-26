use chrono::{Datelike, NaiveDateTime, Timelike, Utc};
use list_helper_core::{ListCursor, ListData, ListInfo};
use list_helper_macro::ListCursor;
use once_cell::sync::Lazy;
use regex::Regex;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::widget::WidgetWithBlock;
use crate::{
    git::{git_log, Commit, CommitRange, Decoration},
    views::statusline::Status,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Track {
    Node,
    Continue,
    MergeDown,
    MergeUp,
}

#[derive(Debug, Clone)]
pub struct CommitNode {
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone)]
pub struct CommitGraph {
    pub graph: Vec<CommitNode>,
}

impl CommitGraph {
    pub fn new(commits: &Vec<Commit>) -> CommitGraph {
        // use a vector of open branches to preserve order
        let mut tracks: Vec<String> = vec![];

        CommitGraph {
            graph: commits
                .iter()
                .map(|c| {
                    let mut node_tracks: Vec<Track> = vec![];

                    // used to walk through parent commits
                    let mut parent_hash_iter = c.parent_hashes.iter();

                    if let Some(x) = tracks.iter().position(|t| t == &c.hash) {
                        // this commit is in the track list, so its node will
                        // be inserted into the track list at the commit hash's
                        // first occurrence

                        for _ in 0..x {
                            node_tracks.push(Track::Continue);
                        }
                        node_tracks.push(Track::Node);

                        // replace this branch in the open branches list with
                        // its first parent (if it has parents)
                        if let Some(parent_hash) = parent_hash_iter.next() {
                            tracks[x] = parent_hash.clone();
                        }

                        // all other occurrences of this node are merge-downs
                        for i in x + 1..tracks.len() {
                            if tracks[i] == c.hash {
                                node_tracks.push(Track::MergeDown);
                            } else {
                                node_tracks.push(Track::Continue);
                            }
                        }

                        tracks = tracks
                            .iter()
                            .filter(|&t| t != &c.hash)
                            .map(|t| t.clone())
                            .collect();
                    } else {
                        // this commit isn't in the track list, so it will open
                        // a new track

                        for _ in 0..tracks.len() {
                            node_tracks.push(Track::Continue);
                        }
                        node_tracks.push(Track::Node);
                    }

                    // create tracks for all this commit's remaining parents
                    for p in parent_hash_iter {
                        // don't add renderable tracks if the commit only has
                        // one parent (a Node track will already have been added
                        // for that) or if the track list already contains the
                        // commit (in which case a new track isn't needed)
                        if c.parent_hashes.len() > 1 && !tracks.contains(p) {
                            node_tracks.push(Track::MergeUp);
                        }
                        tracks.push(p.clone());
                    }

                    CommitNode {
                        tracks: node_tracks,
                    }
                })
                .collect::<Vec<CommitNode>>(),
        }
    }
}

#[derive(Debug, Clone, ListCursor)]
pub struct Commits {
    list: ListData,
    commits: Vec<Commit>,
    mark: Option<usize>,
    graph: CommitGraph,
}

impl Commits {
    pub fn new() -> Commits {
        let commits = git_log();
        let graph = CommitGraph::new(&commits);

        Commits {
            list: ListData::new(),
            mark: None,
            commits,
            graph,
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
                    c[mark].hash.clone()
                } else {
                    c[cursor].hash.clone()
                }
            }
            _ => c[cursor].hash.clone(),
        };
        let end = match self.mark {
            Some(mark) => {
                if mark > cursor {
                    Some(c[cursor].hash.clone())
                } else {
                    Some(c[mark].hash.clone())
                }
            }
            _ => None,
        };

        CommitRange { start, end }
    }
}

impl ListInfo for Commits {
    fn list_count(&self) -> usize {
        self.commits.len()
    }

    fn list_pos(&self) -> usize {
        self.cursor()
    }
}

impl Status for Commits {
    fn status(&self) -> String {
        format!("{}", self.get_range())
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

const BULLET: char = '•';

fn draw_graph_node(node: &CommitNode) -> String {
    let mut graph = String::from("");

    for i in 0..node.tracks.len() {
        match node.tracks[i] {
            Track::Continue => {
                graph.push('│');
                graph.push(' ');
            }
            Track::Node => {
                graph.push(BULLET);
                if let Some(Track::Continue) = node.tracks.get(i + 1) {
                    graph.push(' ');
                }
            }
            Track::MergeDown => {
                match node.tracks.get(i - 1) {
                    Some(Track::Node) => {
                        graph.push('╶');
                    }
                    _ => {
                        graph.push('─');
                    }
                }
                match node.tracks.get(i + 1) {
                    Some(Track::MergeDown) => {
                        graph.push('┴');
                    }
                    _ => {
                        graph.push('╯');
                    }
                }
            }
            Track::MergeUp => {
                match node.tracks.get(i - 1) {
                    Some(Track::Node) => {
                        graph.push('╶');
                    }
                    _ => {
                        graph.push('─');
                    }
                }
                match node.tracks.get(i + 1) {
                    Some(Track::MergeUp) => {
                        graph.push('┬');
                    }
                    _ => {
                        graph.push('╮');
                    }
                }
            }
        }
    }

    graph
}

static COMMIT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\w+(\(\w+\))?!?:.").unwrap());

impl<'a> Widget for CommitsView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let height = area.height as usize;
        let cursor = self.commits.cursor();

        self.commits.set_list_height(height);

        let hash_width = self.commits.commits[0].hash.len();

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
                            if cursor > mark {
                                "┏"
                            } else if cursor < mark {
                                "┗"
                            } else {
                                "╺"
                            }
                        } else if cursor == i {
                            if cursor > mark {
                                "┗"
                            } else {
                                "┏"
                            }
                        } else if i < cursor && i > mark
                            || i > cursor && i < mark
                        {
                            "┃"
                        } else {
                            " "
                        }
                    }
                    _ => {
                        if i == cursor {
                            "┗"
                        } else if i < cursor {
                            "┃"
                        } else {
                            " "
                        }
                    }
                };

                let age = relative_time(c);
                let author =
                    format!("{:width$}", c.author_name, width = author_width);

                // draw the graph
                let graph = draw_graph_node(&self.commits.graph.graph[i]);

                let mut item: Vec<Span> = vec![
                    Span::from(prefix),
                    Span::styled(
                        format!("{}", &c.hash[..hash_width]),
                        Style::default().fg(Color::Indexed(5)),
                    ),
                    Span::from(" "),
                    Span::styled(
                        format!("{:>width$}", age, width = time_width),
                        Style::default().fg(Color::Indexed(4)),
                    ),
                    Span::from(" "),
                    Span::styled(
                        author,
                        Style::default().fg(Color::Indexed(2)),
                    ),
                    Span::from(" "),
                    Span::from(graph),
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

                if COMMIT_RE.is_match(&c.subject) {
                    let mut subj_type = c.subject.clone();
                    let colon_idx = c.subject.find(':').unwrap();
                    let subj_mesg = subj_type.split_off(colon_idx + 1);
                    item.push(Span::styled(
                        subj_type,
                        Style::default().add_modifier(Modifier::BOLD),
                    ));
                    item.push(Span::from(subj_mesg));
                } else {
                    item.push(Span::from(c.subject.clone()));
                }

                ListItem::new(Spans::from(item))
            })
            .collect();

        let mut list = List::new(items)
            .highlight_style(Style::default().bg(Color::Indexed(0)));

        if let Some(b) = self.block {
            list = list.block(b);
        }

        StatefulWidget::render(list, area, buf, self.commits.list_state());
    }
}
