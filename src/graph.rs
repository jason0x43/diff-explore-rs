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
    /// the direct ancestor commit of the cell
    pub hash: Option<String>,
    /// the commit that this cell is related to; transient cells (Merges,
    /// ContinueRights, etc.) will have `related` but not `hash`; any cell with
    /// a hash value should have `hash` == `related`
    pub related: String,
    /// how the cell relates to the next row
    pub track: Track,
}

impl CommitCell {
    fn new(hash: Option<&String>, related: String, track: Track) -> CommitCell {
        CommitCell {
            hash: match hash {
                Some(s) => Some(s.clone()),
                _ => None,
            },
            related,
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
                            } else if let Some(x) =
                                prev_tracks.iter().position(|p| {
                                    p.hash.is_some() && p.hash == tracks[i].hash
                                })
                            {
                                for y in i..x {
                                    if y < tracks.len() {
                                        tracks[y].track = Track::ContinueRight;
                                    } else {
                                        tracks.push(CommitCell::new(
                                            None,
                                            tracks[i].hash.clone().unwrap(),
                                            Track::ContinueRight,
                                        ));
                                    }
                                }
                                if x < tracks.len() {
                                    tracks[x].track = Track::ContinueUp;
                                } else {
                                    tracks.push(CommitCell::new(
                                        None,
                                        tracks[i].hash.clone().unwrap(),
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
                            tracks[x].related = tracks[x].hash.clone().unwrap();
                            tracks[x].track = Track::Node;
                        }

                        // clear out any other instances of this commit's hash
                        // in the track list
                        for y in x + 1..tracks.len() {
                            if tracks[y].hash == Some(c.hash.clone()) {
                                tracks[y].related =
                                    tracks[y].hash.clone().unwrap();
                                tracks[y].hash = None;
                                tracks[y].track = Track::Branch;
                            }
                        }
                    } else {
                        // this commit's hash isn't in the tracks list -- create
                        // a new track for it
                        let hash = parent_hash_iter.next();
                        tracks.push(CommitCell::new(
                            hash,
                            String::from(hash.unwrap()),
                            Track::Node,
                        ));
                    }

                    // create tracks for all this commit's remaining parents
                    for p in parent_hash_iter {
                        tracks.push(CommitCell::new(
                            Some(p),
                            p.clone(),
                            Track::Merge,
                        ));
                    }

                    prev_tracks = tracks.clone();

                    CommitRow {
                        tracks: tracks.clone(),
                    }
                })
                .collect::<Vec<CommitRow>>(),
        }
    }
}
