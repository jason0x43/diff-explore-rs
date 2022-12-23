use crate::git::Commit;

#[derive(Debug, Clone, Copy)]
pub struct CommitNode {
    /// how many branches are open at this commit
    pub num_open: u16,
    /// how many branches were closed at this commit
    pub num_closed: u16,
    /// the index of the graph branch this commit is connected to
    pub index: u16,
}

#[derive(Debug, Clone)]
pub struct CommitGraph {
    pub graph: Vec<CommitNode>,
}

#[derive(Debug, Clone)]
struct OpenBranch<'a> {
    pub hash: &'a str,
    pub count: usize,
}

impl CommitGraph {
    pub fn new(commits: &Vec<Commit>) -> CommitGraph {
        // use a vector of open branches to preserve order
        let mut open_branches: Vec<OpenBranch> = vec![];

        CommitGraph {
            graph: commits
                .iter()
                .map(|c| {
                    // the horizontal index of this commit's branch
                    let mut index: Option<usize> = None;
                    // number of branches closed by this commit
                    let mut closed_branches = 0;
                    let mut parent_hash_iter = c.parent_hashes.iter();

                    if let Some(x) =
                        open_branches.iter().position(|b| b.hash == &c.hash)
                    {
                        // this commit is a branch root; all of its open
                        // branches will be closed
                        closed_branches = open_branches[x].count as u16;

                        // replace this branch in the open branches list with
                        // its first parent (if it has parents)
                        if let Some(parent_hash) = parent_hash_iter.next() {
                            if let Some(y) = open_branches
                                .iter()
                                .position(|b| b.hash == parent_hash)
                            {
                                // the parent branch is already in the open
                                // branches list -- move it to the this
                                // branch's slot in the open branches list
                                // and increase the count
                                let old = open_branches.remove(y);
                                open_branches[x] = OpenBranch {
                                    count: old.count + 1,
                                    ..old
                                }
                            } else {
                                // the parent branch isn't in the open branches
                                // list -- replace this branch's slot with a new
                                // entry
                                open_branches[x] = OpenBranch {
                                    hash: &parent_hash,
                                    count: 1,
                                };
                            };

                            index = Some(x);
                        }
                    }

                    // push all this commit's parents onto the open branches
                    // list if they're not already in there; skip the first one,
                    // which was handled previously
                    for p in parent_hash_iter {
                        // if the parent hash isn't already assigned to an open
                        // branch, add it to the list
                        match open_branches.iter().position(|b| b.hash == p) {
                            None => {
                                open_branches
                                    .push(OpenBranch { hash: p, count: 1 });
                            }
                            Some(p) => {
                                open_branches[p].count += 1;
                            }
                        }
                    }

                    let num_open_branches =
                        open_branches.iter().fold(0, |acc, b| acc + b.count);

                    CommitNode {
                        num_open: num_open_branches as u16,
                        index: match index {
                            Some(b) => b as u16,
                            _ => num_open_branches as u16 - 1,
                        },
                        num_closed: closed_branches,
                    }
                })
                .collect::<Vec<CommitNode>>(),
        }
    }
}
