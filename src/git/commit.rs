use std::num::ParseIntError;

use chrono::NaiveDateTime;

use super::commits::GitRef;
use crate::time::RelativeTime;

#[derive(Debug, Clone)]
pub struct Decoration {
    pub branches: Vec<String>,
    pub tags: Vec<String>,
    pub refs: Vec<String>,
    pub head: Option<String>,
}

impl Decoration {
    fn new(deco: &str) -> Decoration {
        let mut branches: Vec<String> = vec![];
        let mut tags: Vec<String> = vec![];
        let mut refs: Vec<String> = vec![];
        let mut head: Option<String> = None;

        let deco_str = deco.trim();
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
pub struct Commit {
    pub commit_ref: GitRef,
    pub parent_refs: Vec<GitRef>,
    pub decoration: Decoration,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: Option<NaiveDateTime>,
    pub subject: String,
}

impl Commit {
    pub fn new(
        commit_ref: GitRef,
        parent_refs: Vec<GitRef>,
        decoration: &str,
        author_name: String,
        author_email: String,
        timestamp: Option<&str>,
        subject: String,
    ) -> Commit {
        let ts = if let Some(t) = timestamp {
            NaiveDateTime::from_timestamp_opt(t.parse().unwrap(), 0)
        } else {
            None
        };

        Commit {
            commit_ref,
            parent_refs,
            decoration: Decoration::new(decoration),
            author_name,
            author_email,
            subject,
            timestamp: ts,
        }
    }

    pub fn from_log_line(line: &str) -> Commit {
        let parts: Vec<&str> = line.splitn(7, '|').collect();
        let time: Result<u64, ParseIntError> = parts[5].parse();
        if time.is_err() {
            panic!("Invalid time '{}' in [{}]", parts[5], line);
        }

        Commit {
            commit_ref: GitRef::new(parts[0]),
            parent_refs: if parts[1].len() > 0 {
                GitRef::from_strs(
                    parts[1].split(" ").collect::<Vec<&str>>().as_slice(),
                )
            } else {
                vec![]
            },
            decoration: Decoration::new(&parts[2]),
            author_name: parts[3].into(),
            author_email: parts[4].into(),
            timestamp: if parts[5].len() > 0 {
                NaiveDateTime::from_timestamp_opt(parts[5].parse().unwrap(), 0)
            } else {
                None
            },
            subject: parts[6].into(),
        }
    }
}

impl RelativeTime for Commit {
    fn relative_time(&self) -> String {
        match self.timestamp {
            Some(ts) => ts.relative_time(),
            _ => "".into(),
        }
    }
}
