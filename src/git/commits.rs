use std::{
    ffi::OsStr,
    fmt::{self, Display, Error, Formatter},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GitRef(String);

impl GitRef {
    pub fn new(ref_str: impl Into<String>) -> Self {
        Self(ref_str.into())
    }

    pub fn from_strs(refs: &[&str]) -> Vec<GitRef> {
        refs.iter().map(|r| GitRef::new(*r)).collect()
    }

    pub fn unstaged(len: usize) -> GitRef {
        Self::new("0".repeat(len))
    }

    pub fn staged(len: usize) -> GitRef {
        Self::new("S".repeat(len))
    }

    pub fn is_staged(&self) -> bool {
        self.0.starts_with('S')
    }

    pub fn is_unstaged(&self) -> bool {
        self.0 == "0".repeat(self.0.len())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn contains(&self, query: &str) -> bool {
        self.0.contains(query)
    }
}

impl From<GitRef> for String {
    fn from(value: GitRef) -> Self {
        value.0
    }
}

impl Display for GitRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<OsStr> for GitRef {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl From<&str> for GitRef {
    fn from(value: &str) -> Self {
        GitRef::new(String::from(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Staged,
    Unstaged,
    Ref(GitRef),
}

impl Display for Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Staged => write!(f, "STAGED"),
            Self::Unstaged => write!(f, "UNSTAGED"),
            Self::Ref(h) => write!(f, "{}", h),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DiffType {
    Diff,
    Show,
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
        let mut diff_type = DiffType::Diff;
        if let Some(a) = &anchor {
            if let Target::Ref(t) = &target {
                if t == a {
                    diff_type = DiffType::Show;
                }
            }
        };

        DiffAction {
            target,
            anchor,
            diff_type,
        }
    }

    /// Describe a show for the given target commit
    pub fn show(target: Target) -> DiffAction {
        DiffAction {
            target,
            anchor: None,
            diff_type: DiffType::Show,
        }
    }

    /// Describe a diff of the unstaged changes against the working directory
    pub fn unstaged() -> DiffAction {
        DiffAction::diff(Target::Unstaged, None)
    }

    /// Describe a diff of the staged changes against HEAD
    pub fn staged() -> DiffAction {
        DiffAction::diff(Target::Staged, None)
    }

    /// This action involves the staging area
    pub fn has_staged(&self) -> bool {
        self.target == Target::Staged
    }

    /// Is this a show (vs a diff) action
    pub fn is_show(&self) -> bool {
        self.diff_type == DiffType::Show
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
