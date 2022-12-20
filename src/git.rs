use core::fmt;
use serde::Deserialize;
use serde_json::Result;
use std::{
    fmt::Display,
    process::{Command, Output},
};

#[derive(Debug, Clone, Deserialize)]
pub struct Commit {
    pub commit: String,
    pub decoration: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: u64,
    pub subject: String,
}

#[derive(Debug, Clone, Default)]
pub struct CommitRange {
    pub start: String,
    pub end: Option<String>,
}

impl CommitRange {
    pub fn to_string(&self) -> String {
        match &self.end {
            Option::Some(e) => {
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
            Option::Some(e) => &e[..8],
            _ => "<index>",
        };
        write!(f, "CommitRange({}..{})", start, end)
    }
}

#[derive(Debug, Clone)]
pub struct Stat {
    pub adds: u32,
    pub deletes: u32,
    pub path: String,
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

pub fn git_log() -> Result<Vec<Commit>> {
    let output = Command::new("git")
        .arg("log")
        .arg("--date=iso8601-strict")
        .arg("--decorate")
        .arg(
            "--pretty=format:{\
			  \"commit\":\"%H\",\
			  \"decoration\":\"%d\",\
			  \"author_name\":\"%aN\",\
			  \"author_email\":\"%aE\",\
			  \"timestamp\":%at,\
			  \"subject\":\"%s\"\
			},",
        )
        .output()
        .expect("unable to read git log");
    let out_str =
        String::from_utf8(output.stdout).expect("invalid output string");
    let out_clean = out_str.trim_end_matches("\n").trim_end_matches(",");
    let out_array = format!("[{}]", out_clean);
    serde_json::from_str(&out_array)
}

const RENAME_THRESHOLD: u16 = 50;

fn to_stats(output: Output) -> Vec<Stat> {
    let out_str =
        String::from_utf8(output.stdout).expect("invalid output string");
    out_str
        .trim_end_matches("\n")
        .split("\n")
        .filter(|x| x.len() > 0)
        .map(|x| Stat::new(x))
        .collect()
}

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
