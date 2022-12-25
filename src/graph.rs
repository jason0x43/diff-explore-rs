use crate::git::Commit;

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
