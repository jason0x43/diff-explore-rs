use crate::git::Commit;

#[derive(Debug, Clone, PartialEq)]
pub enum Track {
    Node,
    Continue,
    ContinueRight,
    ContinueUp,
    Branch,
    Merge,
}

#[derive(Debug, Clone)]
pub struct CommitCell {
    /// the direct ancestor hash of the cell
    pub hash: Option<String>,
    /// how the cell relates to the next row
    pub track: Track,
}

impl CommitCell {
    fn new(hash: Option<&String>, track: Track) -> CommitCell {
        CommitCell {
            hash: match hash {
                Some(s) => Some(s.clone()),
                _ => None,
            },
            track,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommitRow {
    pub tracks: Vec<CommitCell>,
}

#[derive(Debug, Clone)]
pub struct CommitGraph {
    pub graph: Vec<CommitRow>,
}

const SPACE_CHAR: char = ' ';
const BULLET_CHAR: char = '•';
const BIG_BULLET_CHAR: char = '●';
const RIGHT_UP_CHAR: char = '╯';
const RIGHT_DOWN_CHAR: char = '╮';
const TEE_DOWN_CHAR: char = '┬';
const TEE_UP_CHAR: char = '┴';
const VLINE_CHAR: char = '│';
const UP_RIGHT_CHAR: char = '╭';
const HALF_HLINE_CHAR: char = '╶';
const HLINE_CHAR: char = '─';

// Drawing rules
//
// 1. New head nodes will add new tracks
// 2. The first parent of a node will occupy its track
// 3. Additional parents of a node will occupy tracks to the right
// 4. A node will collapse all of its descendent tracks. Tracks to the right
//    will be able to move left on the next line.
//
// Example
//
//   ∙ Run CI tests on multipl
//   ∙ Fix circular references
//   │ ∙ [fix-circular-referen
//   │ ●─╮ Merge branch 'maste
//   ∙─│─╯ Add build step to c
//   ∙ │ Add missing eslintrc
//   ∙ │ Update digdug depende
//   ∙ │ Add GitHub Action CI
//   │ ∙ Fix circular referenc
//   │ │ ∙ [fix-test] Rework t
//   │ │ │ ∙ [fix-circular-ref
//   │ │ │ ∙ Fix LF
//   ∙─┴─╯ │ <4.10.0> Update d
//   ∙ ╭───╯ Create SECURITY.m
//   │ │ ∙ [intern-5] {origin/
//   ∙ │ │ [4.9] [4.x] {origin
//   ∙ │ │ Update dependencies
//   │ │ ∙ fix(leadfoot): hand
//   │ │ │ ∙ <4.9.0> Updating
//   ∙ │ │ │ Updating source v
//   │ ∙─│─╯ {origin/4.x} Add
//   │ │ │ ∙ [4.8] {origin/4.8
//   │ │ │ ∙ <4.8.8> Updating
//   │ │ ∙ │ fix(core): add mi
//   │ │ │ │ ∙ [fix-functional
//   │ │ │ ∙ │ fix: add missin
//   │ │ ∙─│─╯ [ctrl-c] test:
//   │ │ ∙ │ fix(leadfoot): im

impl CommitGraph {
    pub fn new(commits: &Vec<Commit>) -> CommitGraph {
        // use a vector of open branches to preserve order
        let mut tracks: Vec<CommitCell> = vec![];
        let mut prev_tracks: Vec<CommitCell> = vec![];

        CommitGraph {
            graph: commits
                .iter()
                .map(|c| {
                    // used to walk through parent commits
                    let mut parent_hash_iter = c.parent_hashes.iter();

                    tracks = tracks
                        .iter()
                        .filter(|t| t.hash.is_some())
                        .map(|t| t.clone())
                        .collect();

                    let num_tracks = tracks.len();
                    for i in 0..num_tracks {
                        if let Some(t) = prev_tracks.get(i) {
                            if t.hash == tracks[i].hash {
                                tracks[i].track = Track::Continue;
                            } else if let Some(x) = prev_tracks
                                .iter()
                                .position(|p| p.hash == tracks[i].hash)
                            {
                                for y in i..x {
                                    if y < tracks.len() {
                                        tracks[y].track = Track::ContinueRight;
                                    } else {
                                        tracks.push(CommitCell::new(
                                            None,
                                            Track::ContinueRight,
                                        ));
                                    }
                                }
                                if x < tracks.len() {
                                    tracks[x].track = Track::ContinueUp;
                                } else {
                                    tracks.push(CommitCell::new(
                                        None,
                                        Track::ContinueUp,
                                    ));
                                }
                            }
                        }
                    }

                    if let Some(x) = tracks
                        .iter()
                        .position(|t| t.hash == Some(c.hash.clone()))
                    {
                        // this commit's hash is in the track list, so its node
                        // will be inserted into the track list at the commit
                        // hash's first occurrence

                        // the hash of this commit's first parent (if it has
                        // parents) will become the ancestor hash of this
                        // commit's track
                        if let Some(parent_hash) = parent_hash_iter.next() {
                            tracks[x].hash = Some(parent_hash.clone());
                            tracks[x].track = Track::Node;
                        }

                        // clear out any other instances of this commit's hash
                        // in the track list
                        for y in x + 1..tracks.len() {
                            if tracks[y].hash == Some(c.hash.clone()) {
                                tracks[y].hash = None;
                                tracks[y].track = Track::Branch;
                            }
                        }
                    } else {
                        // this commit's hash isn't in the tracks list -- create
                        // a new track for it
                        tracks.push(CommitCell::new(
                            parent_hash_iter.next(),
                            Track::Node,
                        ));
                    }

                    // create tracks for all this commit's remaining parents
                    for p in parent_hash_iter {
                        tracks.push(CommitCell::new(Some(p), Track::Merge));
                    }

                    prev_tracks = tracks.clone();

                    CommitRow {
                        tracks: tracks.clone(),
                    }
                })
                .collect::<Vec<CommitRow>>(),
        }
    }

    pub fn draw_graph_node(&self, node: usize) -> String {
        let node = &self.graph[node];
        let mut graph = String::from("");

        // set to true when a horizontal line should be drawn
        let mut draw_hline = false;

        if node.tracks.len() == 0 {
            return String::from("");
        }

        // render the first track by itself to simplify look-backs when
        // processing the remaining tracks
        match node.tracks[0].track {
            Track::Continue => {
                graph.push(VLINE_CHAR);
            }
            Track::Node => {
                // if there's a merge after this node, draw a horizontal line
                // from this node to the merge
                draw_hline = node
                    .tracks
                    .iter()
                    .find(|t| matches!(t.track, Track::Merge | Track::Branch))
                    .is_some();

                if node
                    .tracks
                    .iter()
                    .find(|t| t.track == Track::Merge)
                    .is_some()
                {
                    graph.push(BIG_BULLET_CHAR);
                } else {
                    graph.push(BULLET_CHAR);
                }
            }
            _ => {}
        }

        for i in 1..node.tracks.len() {
            match node.tracks[i].track {
                Track::Continue => {
                    if draw_hline {
                        if node.tracks[i - 1].track == Track::Node {
                            graph.push(HALF_HLINE_CHAR);
                        } else {
                            graph.push(HLINE_CHAR);
                        }
                    } else if matches!(
                        node.tracks[i - 1].track,
                        Track::Merge
                            | Track::Branch
                            | Track::Continue
                            | Track::Node
                    ) {
                        graph.push(SPACE_CHAR);
                    }

                    graph.push(VLINE_CHAR);
                }

                Track::ContinueRight => {
                    if node.tracks[i - 1].track == Track::ContinueRight {
                        // this is an intermediate ContinueRight
                        graph.push(HLINE_CHAR);
                        graph.push(HLINE_CHAR);
                    } else {
                        // this is the initial ContinueRight
                        graph.push(SPACE_CHAR);
                        graph.push(UP_RIGHT_CHAR);
                    }
                }

                Track::ContinueUp => {
                    graph.push(HLINE_CHAR);
                    graph.push(RIGHT_UP_CHAR);
                }

                Track::Node => {
                    if matches!(
                        node.tracks[i - 1].track,
                        Track::Merge
                            | Track::Branch
                            | Track::Continue
                            | Track::Node
                    ) {
                        graph.push(SPACE_CHAR);
                    }

                    // enable draw_hline if there's a merge later in the track
                    draw_hline = node
                        .tracks
                        .iter()
                        .skip(i + 1)
                        .find(|t| {
                            matches!(t.track, Track::Merge | Track::Branch)
                        })
                        .is_some();

                    if node
                        .tracks
                        .iter()
                        .skip(i + 1)
                        .find(|t| t.track == Track::Merge)
                        .is_some()
                    {
                        graph.push(BIG_BULLET_CHAR);
                    } else {
                        graph.push(BULLET_CHAR);
                    }
                }

                Track::Branch | Track::Merge => {
                    if node.tracks[i - 1].track == Track::Node {
                        graph.push(HALF_HLINE_CHAR);
                    } else {
                        graph.push(HLINE_CHAR);
                    }

                    let (tee_char, corner_char) =
                        if node.tracks[i].track == Track::Branch {
                            (TEE_UP_CHAR, RIGHT_UP_CHAR)
                        } else {
                            (TEE_DOWN_CHAR, RIGHT_DOWN_CHAR)
                        };

                    // There may be serveral Merges or Braches in a row, in
                    // which case the intermediate ones should be Ts, and the
                    // last one should be a corner
                    if node.tracks.len() > i + 1
                        && node.tracks[i + 1].track == node.tracks[i].track
                    {
                        graph.push(tee_char);
                    } else {
                        graph.push(corner_char);
                    }

                    // a merge ends an hline
                    draw_hline = false;
                }
            }
        }

        graph
    }
}
