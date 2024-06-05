# diff-explore

A terminal program to explore git diffs (a port of
https://github.com/jason0x43/diff-explore-go).

## Building

Clone the repo and run `cargo install --path .`.

## Using

Run `de` in a git repo, or `de ~/path/to/repo`.

The interface is similar to tig's, but de only does one thing: show diffs. Use
the arrow keys or j/k to select a commit, then press enter. De will switch to a
diff stat view, showing which files were updated between the current worktree
and the selected commit. Select a file, and de will show the diff for that
particular file. De watches the worktree and live-updates the diff when the
worktree changes.
