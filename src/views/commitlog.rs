use std::{cmp::min, collections::HashMap};

use once_cell::sync::Lazy;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget,
    },
};
use regex::Regex;

use crate::graph::CommitGraph;
use crate::{
    git::{git_log, git_log_message, Commit, GitRef, Target},
    graph::{CommitRow, Track},
    list::{ListCursor, ListData, ListInfo, ListScroll},
    search::Search,
    string::Ellipses,
    time::RelativeTime,
    ui::highlight_spans,
    views::statusline::Status,
};

/// Formatted fields that are used for searching and rendering
struct CommitFields {
    hash: GitRef,
    age: String,
    author: String,
    branches: Vec<String>,
    tags: Vec<String>,
    refs: Vec<String>,
    head: Option<String>,
    subject: String,
}

impl CommitFields {
    fn new(c: &Commit) -> CommitFields {
        let deco = &c.decoration;
        CommitFields {
            age: c.relative_time(),
            author: c.author_name.clone(),
            hash: c.commit_ref.clone(),
            branches: deco
                .branches
                .iter()
                .map(|b| format!("[{}]", b))
                .collect(),
            head: deco.head.as_ref().map(|h| format!("[{}]", h)),
            tags: deco.tags.iter().map(|b| format!("<{}>", b)).collect(),
            refs: deco.refs.iter().map(|b| format!("<{}>", b)).collect(),
            subject: c.subject.clone(),
        }
    }

    fn contains(&self, query: &String) -> bool {
        self.hash.contains(query)
            || self.age.contains(query)
            || self.author.contains(query)
            || self.branches.iter().any(|b| b.contains(query))
            || self.tags.iter().any(|b| b.contains(query))
            || self.refs.iter().any(|b| b.contains(query))
            || (self.head.is_some()
                && self.head.as_ref().unwrap().contains(query))
            || self.subject.contains(query)
    }
}

#[derive(Debug, Clone)]
pub struct CommitLog {
    list: ListData,
    commits: Vec<Commit>,
    mark: Option<usize>,
    graph: CommitGraph,
    query: Option<String>,
    show_details: bool,
}

impl CommitLog {
    pub fn new() -> CommitLog {
        let commits = git_log();
        let graph = CommitGraph::new(&commits);

        CommitLog {
            list: ListData::new(),
            mark: None,
            commits,
            graph,
            query: None,
            show_details: false,
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

    pub fn get_selected(&self) -> Target {
        let r = &self.commits[self.cursor()].commit_ref;
        if r.is_staged() {
            Target::Staged
        } else if r.is_unstaged() {
            Target::Unstaged
        } else {
            Target::Ref(r.clone())
        }
    }

    pub fn get_marked(&self) -> Option<GitRef> {
        self.mark.map(|m| self.commits[m].commit_ref.clone())
    }

    pub fn toggle_show_details(&mut self) {
        self.show_details = !self.show_details;
    }
}

impl ListInfo for CommitLog {
    fn list_count(&self) -> usize {
        self.commits.len()
    }

    fn list_pos(&self) -> usize {
        self.cursor()
    }

    fn set_list_pos(&mut self, pos: usize) {
        self.cursor_to(pos);
    }
}

impl ListScroll for CommitLog {
    fn height(&self) -> usize {
        self.list.height
    }

    fn scroll_to(&mut self, line: usize) {
        self.cursor_to(line);
    }
}

impl ListCursor for CommitLog {
    fn list_state(&self) -> &ListState {
        &self.list.state
    }

    fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list.state
    }
}

impl Status for CommitLog {
    fn status(&self) -> String {
        let marked = self.get_marked();
        let selected = self.get_selected();
        match marked {
            Some(m) => {
                format!("{}..{}", m, selected)
            }
            _ => {
                format!("{}", selected)
            }
        }
    }
}

impl Search for CommitLog {
    fn set_search(&mut self, query: Option<String>) {
        self.query = query;
    }

    fn get_search(&self) -> Option<String> {
        self.query.clone()
    }

    fn is_match(&self, idx: usize) -> bool {
        match &self.query {
            Some(query) => {
                CommitFields::new(&self.commits[idx]).contains(query)
            }
            _ => false,
        }
    }
}

/// The Widget used to render Commits
pub struct CommitsView<'a> {
    commits: &'a mut CommitLog,
    block: Option<Block<'a>>,
}

impl<'a> CommitsView<'a> {
    pub fn new(commits: &'a mut CommitLog) -> CommitsView<'a> {
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

/// Get the color to be used for continuation lines in the graph
fn get_commit_color(
    hash: &GitRef,
    colors: &mut HashMap<GitRef, Color>,
) -> Color {
    if !colors.contains_key(hash) {
        colors
            .insert(hash.clone(), Color::Indexed(1 + (colors.len() % 6) as u8));
    }
    *colors.get(hash).unwrap()
}

/// Render a cell in a track
fn draw_cell<'a>(
    hash: &GitRef,
    char: &'a str,
    colors: &mut HashMap<GitRef, Color>,
) -> Span<'a> {
    Span::styled(char, Style::default().fg(get_commit_color(hash, colors)))
}

/// Render the graph for a row
fn draw_graph<'a>(
    node: CommitRow,
    colors: &mut HashMap<GitRef, Color>,
) -> Vec<Span<'a>> {
    let mut graph: Vec<Span> = vec![];

    // set to the commit hash of the target when a horizontal line should be
    // drawn
    let mut draw_hline: Option<&GitRef> = None;

    if node.tracks.is_empty() {
        return graph;
    }

    let track = &node.tracks[0];

    // render the first track by itself to simplify look-backs when
    // processing the remaining tracks
    match node.tracks[0].track {
        Track::Continue => {
            graph.push(draw_cell(&track.related, VLINE_CHAR, colors));
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

            if node.tracks.iter().any(|t| t.track == Track::Merge) {
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
                        graph.push(draw_cell(h, HALF_HLINE_CHAR, colors));
                    } else {
                        graph.push(draw_cell(h, HLINE_CHAR, colors));
                    }
                } else if matches!(
                    node.tracks[i - 1].track,
                    Track::Merge
                        | Track::Branch
                        | Track::Continue
                        | Track::ContinueUp
                        | Track::Node
                ) {
                    graph.push(Span::from(SPACE_CHAR));
                }

                graph.push(draw_cell(&track.related, VLINE_CHAR, colors));
            }

            Track::ContinueRight => {
                if node.tracks[i - 1].track == Track::ContinueRight
                    || node.tracks.iter().skip(i).any(|t| {
                        t.parent == track.parent && t.track == Track::Branch
                    })
                {
                    // this is an intermediate ContinueRight
                    let hash = &track.related;
                    graph.push(draw_cell(hash, HLINE_CHAR, colors));
                    graph.push(draw_cell(hash, HLINE_CHAR, colors));
                } else {
                    if let Some(h) = draw_hline {
                        graph.push(draw_cell(h, HLINE_CHAR, colors));
                    } else {
                        graph.push(Span::from(SPACE_CHAR));
                    }
                    // this is the initial ContinueRight
                    graph.push(draw_cell(
                        &track.related,
                        UP_RIGHT_CHAR,
                        colors,
                    ));
                }
            }

            Track::ContinueUp => {
                if node.tracks[i - 1].track == Track::Node {
                    graph.push(draw_cell(
                        &track.related,
                        HALF_HLINE_CHAR,
                        colors,
                    ));
                } else {
                    graph.push(draw_cell(&track.related, HLINE_CHAR, colors));
                }
                graph.push(draw_cell(&track.related, RIGHT_UP_CHAR, colors));
            }

            Track::Node => {
                if matches!(
                    node.tracks[i - 1].track,
                    Track::Merge
                        | Track::Branch
                        | Track::Continue
                        | Track::ContinueUp
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
                    .any(|t| t.track == Track::Merge)
                {
                    graph.push(Span::from(BIG_BULLET_CHAR));
                } else {
                    graph.push(Span::from(BULLET_CHAR));
                }
            }

            Track::Branch | Track::Merge => {
                if node.tracks[i - 1].track == Track::Node {
                    graph.push(draw_cell(
                        &track.related,
                        HALF_HLINE_CHAR,
                        colors,
                    ));
                } else {
                    graph.push(draw_cell(&track.related, HLINE_CHAR, colors));
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
                if node.tracks.iter().skip(i + 1).any(|p| {
                    p.track == node.tracks[i].track
                        && p.related == node.tracks[i].related
                }) {
                    graph.push(draw_cell(&track.related, tee_char, colors));
                } else {
                    graph.push(draw_cell(&track.related, corner_char, colors));

                    // the corner of a merge ends an hline
                    draw_hline = None;
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
        let mut colors: HashMap<GitRef, Color> = HashMap::new();
        let constraints: Vec<Constraint> = if self.commits.show_details {
            vec![Constraint::Percentage(50), Constraint::Percentage(50)]
        } else {
            vec![Constraint::Percentage(100)]
        };
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        self.commits.list.height = layout[0].height as usize;

        let rows = self
            .commits
            .commits
            .iter()
            .map(CommitFields::new)
            .collect::<Vec<CommitFields>>();

        let author_width = min(
            20,
            rows.iter()
                .max_by(|x, y| x.author.len().cmp(&y.author.len()))
                .unwrap()
                .author
                .len(),
        );

        let time_width = rows
            .iter()
            .max_by(|x, y| x.age.len().cmp(&y.age.len()))
            .unwrap()
            .age
            .len();

        let items: Vec<ListItem> = rows
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let age = &f.age;
                let author = format!(
                    "{:width$}",
                    f.author.ellipses(author_width),
                    width = author_width
                );

                // draw the graph
                let graph = draw_graph(
                    self.commits.graph.graph[i].clone(),
                    &mut colors,
                );

                let mut spans: Vec<Span> = vec![
                    // commit hash
                    Span::styled(
                        format!("{}", f.hash),
                        Style::default().fg(Color::Indexed(5)),
                    ),
                    Span::from(" "),
                    // age
                    Span::styled(
                        format!("{:>width$}", age, width = time_width),
                        Style::default().fg(Color::Indexed(4)),
                    ),
                    Span::from(" "),
                    // author
                    Span::styled(
                        author,
                        Style::default().fg(Color::Indexed(2)),
                    ),
                    Span::from(" "),
                ];

                spans.extend(graph);
                spans.push(Span::from(" "));

                // subject
                if let Some(head) = &f.head {
                    spans.push(Span::styled(
                        head,
                        Style::default()
                            .fg(Color::Indexed(6))
                            .add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::from(" "));
                }
                f.branches.iter().for_each(|b| {
                    spans.push(Span::styled(
                        b,
                        Style::default().fg(Color::Indexed(6)),
                    ));
                    spans.push(Span::from(" "));
                });
                f.tags.iter().for_each(|t| {
                    spans.push(Span::styled(
                        t,
                        Style::default().fg(Color::Indexed(5)),
                    ));
                    spans.push(Span::from(" "));
                });
                f.refs.iter().for_each(|r| {
                    spans.push(Span::styled(
                        r,
                        Style::default().fg(Color::Indexed(3)),
                    ));
                    spans.push(Span::from(" "));
                });

                if COMMIT_RE.is_match(&f.subject) {
                    let mut subj_type = f.subject.clone();
                    let colon_idx = f.subject.find(':').unwrap();
                    let subj_mesg = subj_type.split_off(colon_idx + 1);
                    spans.push(Span::styled(
                        subj_type,
                        Style::default().add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::from(subj_mesg));
                } else {
                    spans.push(Span::from(f.subject.clone()));
                }

                if let Some(search) = &self.commits.query {
                    spans = highlight_spans(
                        spans.clone(),
                        search,
                        Style::default().add_modifier(Modifier::REVERSED),
                    )
                }

                let mut item = ListItem::new(Line::from(spans));

                if let Some(m) = self.commits.mark {
                    if m == i {
                        item =
                            item.style(Style::default().bg(Color::Indexed(8)));
                    }
                }

                item
            })
            .collect();

        let mut list = List::new(items)
            .highlight_style(Style::default().bg(Color::Indexed(0)));

        if let Some(b) = self.block {
            list = list.block(b);
        }

        StatefulWidget::render(
            list,
            layout[0],
            buf,
            self.commits.list_state_mut(),
        );

        if self.commits.show_details {
            let commit =
                &self.commits.commits[self.commits.cursor()].commit_ref;
            let message = git_log_message(commit);
            let log = Paragraph::new(message)
                .block(Block::default().borders(Borders::ALL));
            log.render(layout[1], buf);
        }
    }
}
