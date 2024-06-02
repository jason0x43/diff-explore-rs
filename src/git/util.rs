use std::process::Command;

use super::{
    commit::Commit,
    commits::{GitRef, Target},
    diff::FileDiff,
    stat::Stat,
    DiffAction,
};

const RENAME_THRESHOLD: u16 = 50;

pub trait Stdout {
    fn stdout_str(&mut self) -> String;
}

impl Stdout for Command {
    fn stdout_str(&mut self) -> String {
        let output =
            self.output().expect("output of command should be a string");
        let out_str = String::from_utf8(output.stdout)
            .expect("output should be a UTF8 string");
        out_str.trim().into()
    }
}

/// Return the absolute root directory of the current repo
pub fn is_git_repo() -> bool {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output();
    match output {
        Err(_) => false,
        Ok(output) => output.status.success(),
    }
}

/// Return the absolute root directory of the current repo
pub fn git_root() -> String {
    Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .stdout_str()
}

/// Return the commit hash of the current branch head
pub fn git_id() -> String {
    Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .stdout_str()
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
        .stdout_str();
    let mut log = output
        .lines()
        .map(Commit::from_log_line)
        .collect::<Vec<Commit>>();

    let hash_len = if let Some(c) = log.first() {
        c.commit_ref.len()
    } else {
        6
    };
    let head = String::from(&git_id()[..hash_len]);

    if has_changes(&DiffAction::staged()) {
        log.insert(
            0,
            Commit::new(
                GitRef::staged(hash_len),
                vec![GitRef::new(head.clone())],
                "",
                "".into(),
                "".into(),
                None,
                "Staged changes".into(),
            ),
        );
    }

    if has_changes(&DiffAction::unstaged()) {
        log.insert(
            0,
            Commit::new(
                GitRef::unstaged(hash_len),
                vec![GitRef::new(head)],
                "",
                "".into(),
                "".into(),
                None,
                "Unstaged changes".into(),
            ),
        );
    }

    log
}

/// Return the diff summary stats between two commits or between a commit and
/// the index or working tree
fn git_summary(commits: &DiffAction) -> String {
    let cmd = &mut Command::new("git");
    cmd.arg("diff").arg("--shortstat");

    if commits.has_staged() {
        cmd.arg("--staged");
    }

    if let Some(h) = &commits.anchor {
        cmd.arg(h);
    }

    if let Target::Ref(h) = &commits.target {
        cmd.arg(h);
    }

    cmd.stdout_str()
}

/// Return a git commit log message for the given commit
pub fn git_log_message(commit: &GitRef) -> String {
    Command::new("git")
        .arg("show")
        .arg("--shortstat")
        .arg(commit)
        .stdout_str()
}

#[derive(Default)]
pub struct GitDiffOpts {
    ignore_whitespace: bool,
}

/// Return file diff stats between two commits, or for a particular commit
/// (between that commit and its parent)
pub fn git_diff_stat(
    action: &DiffAction,
    opts: Option<GitDiffOpts>,
) -> Vec<Stat> {
    let opts = match opts {
        Some(o) => o,
        _ => GitDiffOpts::default(),
    };

    let cmd = &mut Command::new("git");

    if action.is_show() {
        if action.target == Target::Staged || action.target == Target::Unstaged
        {
            cmd.arg("diff");
        } else {
            cmd.arg("show").arg("--format=");
        }
    } else {
        cmd.arg("diff");
    }

    if action.has_staged() {
        cmd.arg("--cached");
    }

    cmd.arg("--numstat");
    cmd.arg(format!("--find-renames={}", RENAME_THRESHOLD));

    if opts.ignore_whitespace {
        cmd.arg("-w");
    }

    match &action.target {
        Target::Staged | Target::Unstaged => {}
        Target::Ref(h) => {
            cmd.arg(h);
        }
    }

    cmd.stdout_str()
        .lines()
        .filter(|x| !x.is_empty())
        .map(Stat::new)
        .collect()
}

/// Return a diff for a specific file between two commits
pub fn git_diff_file(
    path: &str,
    old_path: &str,
    action: &DiffAction,
    opts: Option<GitDiffOpts>,
) -> FileDiff {
    let opts = match opts {
        Some(o) => o,
        _ => GitDiffOpts::default(),
    };

    let command = &mut Command::new("git");
    command.current_dir(git_root());

    if action.is_show() {
        if action.target == Target::Staged || action.target == Target::Unstaged
        {
            command.arg("diff");
        } else {
            command.arg("show");
        }
    } else {
        command.arg("diff");
    }

    if action.has_staged() {
        command.arg("--cached");
    }

    command
        .arg("--patience")
        .arg("--format=")
        .arg(format!("--find-renames={}", RENAME_THRESHOLD))
        .arg("-p");

    if opts.ignore_whitespace {
        command.arg("-w");
    }

    if let Some(h) = &action.anchor {
        command.arg(h);
    }

    if let Target::Ref(h) = &action.target {
        command.arg(h);
    }

    command.arg("--").arg(path);

    if !old_path.is_empty() {
        command.arg(old_path);
    }

    let output = command.stdout_str();
    crate::log!("got {} lines of output", output.lines().count());
    FileDiff::new(&output, action)
}

/// Return true if there are changes across a range
fn has_changes(commits: &DiffAction) -> bool {
    !git_summary(commits).is_empty()
}
