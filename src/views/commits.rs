use std::collections::HashMap;

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

use crate::{
    git::{git_log, Commit, CommitRange, Decoration},
    graph::{CommitRow, Track},
    time::RelativeTime,
    views::statusline::Status,
};
use crate::{graph::CommitGraph, widget::WidgetWithBlock};

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

const SPACE_CHAR: &str = " ";
const BULLET_CHAR: &str = "•";
const BIG_BULLET_CHAR: &str = "●";
const RIGHT_UP_CHAR: &str = "╯";
const RIGHT_DOWN_CHAR: &str = "╮";
const TEE_DOWN_CHAR: &str = "┬";
const TEE_UP_CHAR: &str = "┴";
const VLINE_CHAR: &str = "│";
const UP_RIGHT_CHAR: &str = "╭";
const HALF_HLINE_CHAR: &str = "╶";
const HLINE_CHAR: &str = "─";

fn get_commit_color<'a>(
    hash: &String,
    colors: &'a mut HashMap<String, Color>,
) -> Color {
    if !colors.contains_key(hash) {
        colors
            .insert(hash.clone(), Color::Indexed(1 + (colors.len() % 6) as u8));
    }
    *colors.get(hash).unwrap()
}

fn cell<'a>(
    hash: &String,
    char: &'a str,
    colors: &mut HashMap<String, Color>,
) -> Span<'a> {
    Span::styled(char, Style::default().fg(get_commit_color(hash, colors)))
}

fn draw_graph_node<'a>(
    node: CommitRow,
    colors: &mut HashMap<String, Color>,
) -> Vec<Span<'a>> {
    let mut graph: Vec<Span> = vec![];

    // set to the commit hash of the target when a horizontal line should be
    // drawn
    let mut draw_hline: Option<&String> = None;

    if node.tracks.len() == 0 {
        return graph;
    }

    let track = &node.tracks[0];

    // render the first track by itself to simplify look-backs when
    // processing the remaining tracks
    match node.tracks[0].track {
        Track::Continue => {
            graph.push(cell(&track.related, VLINE_CHAR, colors));
        }
        Track::Node => {
            // if there's a merge after this node, draw a horizontal line
            // from this node to the merge
            draw_hline = if let Some(t) = node
                .tracks
                .iter()
                .find(|t| matches!(t.track, Track::Merge | Track::Branch))
            {
                Some(&t.related)
            } else {
                None
            };

            if node
                .tracks
                .iter()
                .find(|t| t.track == Track::Merge)
                .is_some()
            {
                graph.push(Span::from(BIG_BULLET_CHAR));
            } else {
                graph.push(Span::from(BULLET_CHAR));
            }
        }
        _ => {}
    }

    for i in 1..node.tracks.len() {
        let track = &node.tracks[i];

        match track.track {
            Track::Continue => {
                if let Some(h) = draw_hline {
                    if node.tracks[i - 1].track == Track::Node {
                        graph.push(cell(&h, HALF_HLINE_CHAR, colors));
                    } else {
                        graph.push(cell(&h, HLINE_CHAR, colors));
                    }
                } else if matches!(
                    node.tracks[i - 1].track,
                    Track::Merge
                        | Track::Branch
                        | Track::Continue
                        | Track::Node
                ) {
                    graph.push(Span::from(SPACE_CHAR));
                }

                graph.push(cell(&track.related, VLINE_CHAR, colors));
            }

            Track::ContinueRight => {
                if node.tracks[i - 1].track == Track::ContinueRight {
                    // this is an intermediate ContinueRight
                    let hash = &node.tracks[i - 1].related;
                    graph.push(cell(&hash, HLINE_CHAR, colors));
                    graph.push(cell(&hash, HLINE_CHAR, colors));
                } else {
                    // this is the initial ContinueRight
                    graph.push(Span::from(SPACE_CHAR));
                    graph.push(cell(&track.related, UP_RIGHT_CHAR, colors));
                }
            }

            Track::ContinueUp => {
                graph.push(cell(&track.related, HLINE_CHAR, colors));
                graph.push(cell(&track.related, RIGHT_UP_CHAR, colors));
            }

            Track::Node => {
                if matches!(
                    node.tracks[i - 1].track,
                    Track::Merge
                        | Track::Branch
                        | Track::Continue
                        | Track::Node
                ) {
                    graph.push(Span::from(SPACE_CHAR));
                }

                // enable draw_hline if there's a merge later in the track
                draw_hline = if let Some(t) =
                    node.tracks.iter().skip(i + 1).find(|t| {
                        matches!(t.track, Track::Merge | Track::Branch)
                    }) {
                    Some(&t.related)
                } else {
                    None
                };

                if node
                    .tracks
                    .iter()
                    .skip(i + 1)
                    .find(|t| t.track == Track::Merge)
                    .is_some()
                {
                    graph.push(Span::from(BIG_BULLET_CHAR));
                } else {
                    graph.push(Span::from(BULLET_CHAR));
                }
            }

            Track::Branch | Track::Merge => {
                if node.tracks[i - 1].track == Track::Node {
                    graph.push(cell(&track.related, HALF_HLINE_CHAR, colors));
                } else {
                    graph.push(cell(&track.related, HLINE_CHAR, colors));
                }

                let (tee_char, corner_char) =
                    if node.tracks[i].track == Track::Branch {
                        (TEE_UP_CHAR, RIGHT_UP_CHAR)
                    } else {
                        (TEE_DOWN_CHAR, RIGHT_DOWN_CHAR)
                    };

                // There may be several Merges or Braches in a row, in
                // which case the intermediate ones should be Ts, and the
                // last one should be a corner
                if node.tracks.len() > i + 1
                    && node.tracks[i + 1].track == node.tracks[i].track
                {
                    graph.push(cell(&track.related, tee_char, colors));
                } else {
                    graph.push(cell(&track.related, corner_char, colors));
                }

                // a merge ends an hline
                draw_hline = None;
            }
        }
    }

    graph
}

impl<'a> WidgetWithBlock<'a> for CommitsView<'a> {
    fn block(&mut self, block: Block<'a>) {
        self.block = Some(block);
    }
}

static COMMIT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\w+(\(\w+\))?!?:.").unwrap());

impl<'a> Widget for CommitsView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut colors: HashMap<String, Color> = HashMap::new();
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

        let time_width = self
            .commits
            .commits
            .iter()
            .max_by(|x, y| {
                x.relative_time().len().cmp(&y.relative_time().len())
            })
            .unwrap()
            .relative_time()
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

                let age = c.relative_time();
                let author =
                    format!("{:width$}", c.author_name, width = author_width);

                // draw the graph
                let graph = draw_graph_node(
                    self.commits.graph.graph[i].clone(),
                    &mut colors,
                );

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
                ];

                item.extend(graph);
                item.push(Span::from(" "));

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
