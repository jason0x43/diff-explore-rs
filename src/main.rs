use crate::git::git_log;

mod git;

fn main() {
    println!("{}", git_log());
}