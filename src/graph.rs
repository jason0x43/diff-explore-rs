use crate::git::{Commit, GitRef};

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
    pub parent: Option<GitRef>,

    /// the commit that this cell is related to; transient cells (Merges,
    /// ContinueRights, etc.) will have `related` but not `hash`; any cell with
    /// a hash value should have `hash` == `related`
    pub related: GitRef,

    /// how the cell relates to the next row
    pub track: Track,
}

impl CommitCell {
    fn new(parent: Option<&GitRef>, related: GitRef, track: Track) -> CommitCell {
        CommitCell {
            parent: parent.cloned(),
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
    pub fn new(commits: &[Commit]) -> CommitGraph {
        // use a vector of open branches to preserve order
        let mut tracks: Vec<CommitCell> = vec![];
        let mut prev_tracks: Vec<CommitCell> = vec![];

        CommitGraph {
            graph: commits
                .iter()
                .map(|c| {
                    // used to walk through parent commits
                    let mut parent_hash_iter = c.parent_refs.iter();

                    // initialize the current row of tracks with all the
                    // non-None tracks from the previous row
                    let temp_tracks: Vec<CommitCell> = tracks
                        .iter()
                        .filter(|t| t.parent.is_some()).cloned()
                        .collect();

                    tracks = vec![];

                    let mut offset = 0;

                    // add connections to the tracks in the previous row if
                    // things have shifted
                    (0..temp_tracks.len()).for_each(|i| {
                        let pi = i + offset;

                        if let Some(prev_track) = prev_tracks.get(pi) {
                            // there's a track corresponding to this one in the
                            // previous tracks list

                            let parent = &temp_tracks[i].parent;

                            if prev_track.parent == *parent {
                                // the parent in the previous track is the same
                                // as this track -- it's a continue
                                tracks.push(CommitCell {
                                    track: Track::Continue,
                                    ..prev_track.clone()
                                });
                            } else if let Some(x) =
                                prev_tracks.iter().skip(pi).position(|p| {
                                    p.parent.is_some() && p.parent == *parent
                                })
                            {
                                // this track's parent is in a later track (x)
                                // in the previous tracks list; mark any
                                // tracks between i and x as continuations of
                                // x's parent, adding new tracks as necessary

                                // the search started at pi
                                let x = x + pi;

                                // push a continue cell to parent
                                tracks.push(CommitCell::new(
                                    parent.as_ref(),
                                    parent.clone().unwrap(),
                                    Track::ContinueRight,
                                ));

                                // push some connector cells to get to the
                                // target track
                                for _ in pi + 1..x {
                                    tracks.push(CommitCell::new(
                                        None,
                                        parent.clone().unwrap(),
                                        Track::ContinueRight,
                                    ));
                                }

                                // push a connector cell to the target track
                                tracks.push(CommitCell::new(
                                    None,
                                    parent.clone().unwrap(),
                                    Track::ContinueUp,
                                ));

                                offset += x - i;
                            }
                        }
                    });

                    if let Some(x) = tracks
                        .iter()
                        .position(|t| t.parent == Some(c.commit_ref.clone()))
                    {
                        // this commit's hash is in the track list, so its node
                        // will be inserted into the track list at the commit
                        // hash's first occurrence

                        // the hash of this commit's first parent (if it has
                        // parents) will become the ancestor hash of this
                        // commit's track
                        if let Some(parent_hash) = parent_hash_iter.next() {
                            tracks[x].parent = Some(parent_hash.clone());
                            tracks[x].related =
                                tracks[x].parent.clone().unwrap();
                            tracks[x].track = Track::Node;
                        }

                        // clear out any other instances of this commit's hash
                        // in the track list
                        for y in x + 1..tracks.len() {
                            if tracks[y].parent == Some(c.commit_ref.clone()) {
                                tracks[y].related = c.commit_ref.clone();
                                tracks[y].parent = None;

                                if tracks[y].track == Track::ContinueRight {
                                    // this is a continuation cell added during
                                    // dead track removal -- find the end of the
                                    // continuation run and replace that with a
                                    // Branch
                                    if let Some(t) =
                                        tracks.iter().skip(y + 1).position(
                                            |t| t.track == Track::ContinueUp,
                                        )
                                    {
                                        tracks[y + 1 + t].track = Track::Branch;
                                    }
                                } else {
                                    tracks[y].track = Track::Branch;
                                }
                            }
                        }
                    } else {
                        // this commit's hash isn't in the tracks list -- create
                        // a new track for it
                        let hash = parent_hash_iter.next();
                        if let Some(hash) = hash {
                            tracks.push(CommitCell::new(
                                Some(hash),
                                hash.clone(),
                                Track::Node,
                            ));
                        }
                    }

                    // create tracks for all this commit's remaining parents
                    for p in parent_hash_iter {
                        tracks.push(CommitCell::new(
                            Some(p),
                            p.clone(),
                            Track::Merge,
                        ));
                    }

                    prev_tracks.clone_from(&tracks);

                    CommitRow {
                        tracks: tracks.clone(),
                    }
                })
                .collect::<Vec<CommitRow>>(),
        }
    }
}
