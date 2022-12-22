use core::fmt;
use std::{
    fmt::{Display, Error},
    process::{Command, Output},
};

#[derive(Debug, Clone)]
pub struct Commit {
    pub commit: String,
    pub decoration: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: u64,
    pub subject: String,
}

impl Commit {
    fn from_log_line(line: &str) -> Commit {
        let parts: Vec<&str> = line.splitn(6, '|').collect();
        Commit {
            commit: String::from(parts[0]),
            decoration: String::from(parts[1]),
            author_name: String::from(parts[2]),
            author_email: String::from(parts[3]),
            timestamp: String::from(parts[4]).parse().unwrap(),
            subject: String::from(parts[5]),
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
            (String::from(path_parts[0]), String::from(path_parts[1]))
        } else {
            (String::from(parts[2]), String::from(""))
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
    String::from(out_str.trim_end_matches("\n").trim_end_matches(","))
}

/// Return a git commit log for the current repo
pub fn git_log() -> Vec<Commit> {
    let output = Command::new("git")
        .arg("log")
        .arg("--date=iso8601-strict")
        .arg("--decorate")
        // commit|decoration|author_name|author_email|timestamp|subject
        .arg("--pretty=format:%H|%d|%aN|%aE|%at|%s")
        .output()
        .expect("unable to read git log");
    let out_str =
        String::from_utf8(output.stdout).expect("invalid output string");
    out_str.split("\n").map(|line| Commit::from_log_line(line)).collect()
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

/// Return a diff for a specific file between two commits
pub fn git_diff_file(
    path: &str,
    old_path: &str,
    range: &CommitRange,
    opts: Option<GitDiffOpts>,
) -> Vec<String> {
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

    let root = git_root().unwrap();
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

    let out_str =
        String::from_utf8(output.stdout).expect("invalid output string");

    out_str.split("\n").map(|s| String::from(s)).collect()
}
