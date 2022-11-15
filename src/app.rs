use crate::git::git_log;

pub struct App {
    pub height: u32,
    pub width: u32,
    pub history: Vec<String>,
    pub should_quit: bool,
    pub cursor: u32,
    pub commits: Vec<String>,
}

impl<'a> App {
    pub fn new() -> App {
        App {
            height: 0,
            width: 0,
            history: vec![],
            should_quit: false,
            commits: vec![],
            cursor: 0
        }
    }

    pub fn load_commits(&mut self) {
        let entries = git_log().expect("unable to load git log");
        let entry_list = entries;
        for entry in entry_list {
            self.commits.push(entry.subject.clone())
        }
    }

    pub fn move_cursor_down(&mut self) {
        self.cursor += 1;
    }
}
