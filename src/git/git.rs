use std::process::Command;

use super::{
    commit::Commit,
    commits::{GitRef, Target},
    diff::FileDiff,
    stat::Stat,
    DiffAction,
};

const RENAME_THRESHOLD: u16 = 50;

fn run(cmd: &mut Command) -> String {
    crate::log!("running {:?}", cmd);
    let output = cmd.output().expect("output of rev-parse should be string");
    let out_str = String::from_utf8(output.stdout)
        .expect("output should be a UTF8 string");
    out_str.trim().into()
}

/// Return the absolute root directory of the current repo
pub fn git_root() -> String {
    run(Command::new("git").arg("rev-parse").arg("--show-toplevel"))
}

/// Return the commit hash of the current branch head
pub fn git_id() -> String {
    run(Command::new("git").arg("rev-parse").arg("HEAD"))
}

/// Return a git commit log for the current repo
pub fn git_log() -> Vec<Commit> {
    let output = run(Command::new("git")
        .arg("log")
        .arg("--all")
        .arg("--date=iso8601-strict")
        .arg("--decorate")
        // commit|decoration|author_name|author_email|timestamp|subject
        .arg("--pretty=format:%h|%p|%d|%aN|%aE|%at|%s"));
    let mut log = output
        .lines()
        .map(|line| Commit::from_log_line(line))
        .collect::<Vec<Commit>>();

    let hash_len = if let Some(c) = log.get(0) {
        c.gref.len()
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
                vec![GitRef::new(head.clone())],
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

    match &commits.anchor {
        Some(h) => {
            cmd.arg(h);
        }
        _ => {}
    }

    match &commits.target {
        Target::REF(h) => {
            cmd.arg(h);
        }
        _ => {}
    }

    run(cmd)
}

/// Return a git commit log message for the given commit
pub fn git_log_message(commit: &GitRef) -> String {
    run(Command::new("git")
        .arg("show")
        .arg("--shortstat")
        .arg(commit))
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
        if action.target == Target::STAGED || action.target == Target::UNSTAGED
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
        Target::STAGED | Target::UNSTAGED => {}
        Target::REF(h) => {
            cmd.arg(h);
        }
    }

    run(cmd)
        .lines()
        .filter(|x| x.len() > 0)
        .map(|x| Stat::new(x))
        .collect()
}

/// Return a diff for a specific file between two commits
pub fn git_diff_file<'a>(
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
        if action.target == Target::STAGED || action.target == Target::UNSTAGED
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

    match &action.anchor {
        Some(h) => {
            command.arg(h);
        }
        _ => {}
    }

    match &action.target {
        Target::REF(h) => {
            command.arg(h);
        }
        _ => {}
    }

    command.arg("--").arg(path.clone());

    if old_path.len() > 0 {
        command.arg(old_path.clone());
    }

    let output = run(command);
    crate::log!("got {} lines of output", output.lines().count());
    FileDiff::new(&output, action)
}

/// Return true if there are changes across a range
fn has_changes(commits: &DiffAction) -> bool {
    git_summary(commits).len() > 0
}
