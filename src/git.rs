use std::process::Command;

pub fn git_log() -> &'static str {
    Command::new("git")

}