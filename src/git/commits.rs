use std::{
    ffi::OsStr,
    fmt::{Display, Error, Formatter},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRef {
    ref_str: String,
}

impl GitRef {
    pub fn new(ref_str: impl Into<String>) -> GitRef {
        GitRef { ref_str: ref_str.into() }
    }

    pub fn from_strs(refs: &[&str]) -> Vec<GitRef> {
        let mut v = vec![];
        for i in refs {
            v.push(GitRef::new(*i));
        }
        v
    }

    pub fn unstaged(len: usize) -> GitRef {
        Self::new("0".repeat(len))
    }

    pub fn staged(len: usize) -> GitRef {
        Self::new("S".repeat(len))
    }

    pub fn is_staged(&self) -> bool {
        self.ref_str.starts_with("S")
    }

    pub fn is_unstaged(&self) -> bool {
        self.ref_str == "0".repeat(self.ref_str.len())
    }

    pub fn len(&self) -> usize {
        self.ref_str.len()
    }

    pub fn contains(&self, query: &str) -> bool {
        self.ref_str.contains(query)
    }
}

impl std::hash::Hash for GitRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ref_str.hash(state)
    }
}

impl Display for GitRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.ref_str)
    }
}

impl AsRef<OsStr> for GitRef {
    fn as_ref(&self) -> &OsStr {
        self.ref_str.as_ref()
    }
}

impl From<&str> for GitRef {
    fn from(value: &str) -> Self {
        GitRef::new(String::from(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    STAGED,
    UNSTAGED,
    REF(GitRef),
}

impl Display for Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::STAGED => {
                write!(f, "STAGED")
            }
            Self::UNSTAGED => {
                write!(f, "UNSTAGED")
            }
            Self::REF(h) => {
                write!(f, "{}", h)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DiffType {
    DIFF,
    SHOW,
}

/// A set of commits that will be diffed or shown.
///
/// The target commit is the end state. The anchor commit is the base state. By
/// default, the anchor is HEAD.
#[derive(Debug, Clone)]
pub struct DiffAction {
    /// The target commit -- the selected commit
    pub target: Target,
    /// The anchor commit -- the currently marked commit
    pub anchor: Option<GitRef>,
    diff_type: DiffType,
}

impl DiffAction {
    /// Describe a diff between two commits, or between the target and HEAD
    pub fn diff(target: Target, anchor: Option<GitRef>) -> DiffAction {
        let mut diff_type = DiffType::DIFF;
        match &anchor {
            Some(a) => match &target {
                Target::REF(t) => {
                    if t == a {
                        diff_type = DiffType::SHOW;
                    }
                }
                _ => {}
            },
            _ => {}
        };
        DiffAction {
            target,
            anchor,
            diff_type
        }
    }

    /// Describe a show for the given target commit
    pub fn show(target: Target) -> DiffAction {
        DiffAction {
            target,
            anchor: None,
            diff_type: DiffType::SHOW,
        }
    }

    /// Describe a diff of the unstaged changes against the working directory
    pub fn unstaged() -> DiffAction {
        DiffAction::diff(Target::UNSTAGED, None)
    }

    /// Describe a diff of the staged changes against HEAD
    pub fn staged() -> DiffAction {
        DiffAction::diff(Target::STAGED, None)
    }

    /// This action involves the staging area
    pub fn has_staged(&self) -> bool {
        self.target == Target::STAGED
    }

    /// Is this a show (vs a diff) action
    pub fn is_show(&self) -> bool {
        self.diff_type == DiffType::SHOW
    }
}

/// Display the commits of the diff action
impl Display for DiffAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match &self.anchor {
            Some(h) => {
                write!(f, "{}", h)
            }
            None => {
                write!(f, "{}", self.target)
            }
        }
    }
}
