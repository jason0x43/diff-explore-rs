use core::fmt;
use std::{
    fmt::Display,
    process::{Command, Output},
};

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub parent_hashes: Vec<String>,
    pub decoration: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: u64,
    pub subject: String,
}

impl Commit {
    fn from_log_line(line: &str) -> Commit {
        let parts: Vec<&str> = line.splitn(7, '|').collect();
        Commit {
            hash: parts[0].into(),
            parent_hashes: if parts[1].len() > 0 {
                parts[1].split(" ").map(|p| p.into()).collect()
            } else {
                vec![]
            },
            decoration: parts[2].into(),
            author_name: parts[3].into(),
            author_email: parts[4].into(),
            timestamp: parts[5].parse().unwrap(),
            subject: parts[6].into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommitRange {
    pub start: String,
    pub end: Option<String>,
}

impl CommitRange {
    pub fn to_string(&self) -> String {
        match &self.end {
            Some(e) => {
                format!("{}..{}", self.start, e)
            }
            _ => self.start.clone(),
        }
    }
}

impl Display for CommitRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = &self.start[..8];
        let end = match &self.end {
            Some(e) => &e[..8],
            _ => "<index>",
        };
        write!(f, "CommitRange({}..{})", start, end)
    }
}

#[derive(Debug, Clone)]
pub struct Decoration {
    pub branches: Vec<String>,
    pub tags: Vec<String>,
    pub refs: Vec<String>,
    pub head: Option<String>,
}

impl Decoration {
    pub fn from_commit(commit: &Commit) -> Decoration {
        let mut branches: Vec<String> = vec![];
        let mut tags: Vec<String> = vec![];
        let mut refs: Vec<String> = vec![];
        let mut head: Option<String> = None;

        let deco_str = commit.decoration.trim();
        if deco_str.len() > 0 && deco_str.chars().nth(0) == Some('(') {
            deco_str[1..deco_str.len() - 1].split(", ").for_each(|d| {
                if d.contains(" -> ") {
                    // branch has format "HEAD -> name"
                    head = Some(d.split(" -> ").last().unwrap().into());
                } else if d.starts_with("tag: ") {
                    let tag = d.splitn(2, ": ").last().unwrap();
                    tags.push(tag.into());
                } else if d.contains("/") {
                    refs.push(d.into());
                } else {
                    branches.push(d.into());
                }
            });
        }

        Decoration {
            head,
            branches,
            tags,
            refs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Stat {
    /// Number of added lines
    pub adds: u32,
    /// Number of deleted lines
    pub deletes: u32,
    /// Path of the modified file
    pub path: String,
    /// Original path of the modified file (if renamed)
    pub old_path: String,
}

impl Stat {
    pub fn new(stat_line: &str) -> Stat {
        let parts: Vec<&str> = stat_line.split("\t").collect();
        let adds: u32 = parts[0].parse().unwrap();
        let deletes: u32 = parts[1].parse().unwrap();
        let (path, old_path) = if parts[2].contains(" => ") {
            let path_parts: Vec<&str> = parts[2].split(" => ").collect();
            (path_parts[0].into(), path_parts[1].into())
        } else {
            (parts[2].into(), "".into())
        };

        Stat {
            adds,
            deletes,
            path,
            old_path,
        }
    }
}

/// Return the absolute root directory of the current repo
pub fn git_root() -> String {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .expect("output of rev-parse should be string");
    let out_str = String::from_utf8(output.stdout)
        .expect("output should be a UTF8 string");
    out_str.trim_end_matches("\n").trim_end_matches(",").into()
}

/// Return a git commit log for the current repo
pub fn git_log() -> Vec<Commit> {
    let output = Command::new("git")
        .arg("log")
        .arg("--all")
        .arg("--date=iso8601-strict")
        .arg("--decorate")
        // commit|decoration|author_name|author_email|timestamp|subject
        .arg("--pretty=format:%h|%p|%d|%aN|%aE|%at|%s")
        .output()
        .expect("unable to read git log");
    let out_str =
        String::from_utf8(output.stdout).expect("invalid output string");
    out_str
        .split("\n")
        .map(|line| Commit::from_log_line(line))
        .collect()
}

const RENAME_THRESHOLD: u16 = 50;

fn to_stats(output: Output) -> Vec<Stat> {
    let out_str = String::from_utf8(output.stdout)
        .expect("output should be a UTF8 string");
    out_str
        .trim_end_matches("\n")
        .split("\n")
        .filter(|x| x.len() > 0)
        .map(|x| Stat::new(x))
        .collect()
}

/// Return file diff stats between two commits
pub fn git_diff_stat(range: &CommitRange) -> Vec<Stat> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--numstat")
        .arg(format!("--find-renames={}", RENAME_THRESHOLD))
        .arg(range.to_string())
        .output()
        .expect("unable to get diff stat");
    to_stats(output)
}

#[derive(Default)]
pub struct GitDiffOpts {
    ignore_whitespace: bool,
}

#[derive(Debug, Clone)]
struct ChunkInfo {
    old: u32,
    new: u32,
}

impl ChunkInfo {
    fn new(line: &str) -> ChunkInfo {
        let parts: Vec<&str> = line.split(' ').collect();
        let old: Vec<&str> = parts[1][1..].split(',').collect();
        let new: Vec<&str> = parts[2][1..].split(',').collect();
        let old_start: u32 = old[0].parse().unwrap();
        let new_start: u32 = new[0].parse().unwrap();
        ChunkInfo {
            old: old_start,
            new: new_start,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiffLineNrs {
    pub old: u32,
    pub new: u32,
}

#[derive(Debug, Clone)]
pub enum DiffLine {
    Add(DiffLineNrs),
    Del(DiffLineNrs),
    Same(DiffLineNrs),
    Hunk,
    Start,
    None,
}

impl DiffLine {
    fn new_line(line: &str, old: u32, new: u32) -> DiffLine {
        match line.chars().nth(0) {
            Some('+') => DiffLine::Add(DiffLineNrs { old, new }),
            Some('-') => DiffLine::Del(DiffLineNrs { old, new }),
            _ => DiffLine::Same(DiffLineNrs { old, new }),
        }
    }

    fn new_meta(line: &str) -> DiffLine {
        match line.chars().nth(0) {
            Some('d') => DiffLine::Start,
            Some('@') => DiffLine::Hunk,
            _ => DiffLine::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiffFile {
    /// file path
    pub path: String,
    /// old path (if renamed)
    pub old_path: String,
    /// commit range for diff
    pub range: CommitRange,
    /// raw diff lines
    pub lines: Vec<String>,
    /// metadata about each line
    pub line_meta: Vec<DiffLine>,
}

impl DiffFile {
    fn new(text: &str, range: &CommitRange) -> DiffFile {
        let mut chunk_info: Option<ChunkInfo> = None;
        let mut path: &str = "";
        let mut old_path: &str = "";
        let lines: Vec<String> = text.lines().map(|s| s.into()).collect();
        let line_meta: Vec<DiffLine> = lines
            .iter()
            .map(|s| {
                if s.starts_with("diff ") {
                    chunk_info = None;
                    DiffLine::new_meta(&s)
                } else if s.starts_with("@@") {
                    chunk_info = Some(ChunkInfo::new(&s));
                    DiffLine::new_meta(&s)
                } else if let Some(info) = &mut chunk_info {
                    let old = info.old;
                    let new = info.new;
                    match s.chars().nth(0) {
                        Some('+') => info.new += 1,
                        Some('-') => info.old += 1,
                        _ => {
                            info.new += 1;
                            info.old += 1;
                        }
                    }
                    DiffLine::new_line(&s, old, new)
                } else if s.starts_with("---") {
                    old_path = s[4..].into();
                    DiffLine::new_meta(&s)
                } else if s.starts_with("+++") {
                    path = s[4..].into();
                    DiffLine::new_meta(&s)
                } else {
                    DiffLine::new_meta(&s)
                }
            })
            .collect();

        DiffFile {
            path: path.into(),
            old_path: old_path.into(),
            range: range.clone(),
            lines,
            line_meta,
        }
    }
}

/// Return a diff for a specific file between two commits
pub fn git_diff_file<'a>(
    path: &str,
    old_path: &str,
    range: &CommitRange,
    opts: Option<GitDiffOpts>,
) -> DiffFile {
    let cmd = match &range.end {
        Some(v) => {
            if *v == range.start {
                "show"
            } else {
                "diff-tree"
            }
        }
        _ => "diff-index",
    };
    let opts = match opts {
        Some(o) => o,
        _ => GitDiffOpts::default(),
    };

    let root = git_root();
    let mut command = Command::new("git");
    command
        .current_dir(root)
        .arg(cmd)
        .arg("--patience")
        .arg(format!("--find-renames={}", RENAME_THRESHOLD))
        .arg("-p");

    if opts.ignore_whitespace {
        command.arg("-w");
    }

    command.arg(range.to_string()).arg("--").arg(path.clone());

    if old_path.len() > 0 {
        command.arg(old_path.clone());
    }

    let output = command.output().expect("unable to get diff");
    let output =
        String::from_utf8(output.stdout).expect("invalid output string");
    DiffFile::new(&output, range)
}
