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
    fn new(deco: &String) -> Decoration {
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
    pub gref: GitRef,
    pub parent_grefs: Vec<GitRef>,
    pub decoration: Decoration,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: Option<NaiveDateTime>,
    pub subject: String,
}

impl Commit {
    pub fn from_log_line(line: &str) -> Commit {
        let parts: Vec<&str> = line.splitn(7, '|').collect();
        Commit {
            gref: GitRef::new(parts[0].into()),
            parent_grefs: if parts[1].len() > 0 {
                parts[1].split(" ").map(|p| p.into()).collect()
            } else {
                vec![]
            },
            decoration: Decoration::new(&parts[2].into()),
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
