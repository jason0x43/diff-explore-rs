mod app;
mod commits;
mod console;
mod cursor;
mod diff;
mod events;
mod git;
mod stack;
mod stats;
mod statusline;
mod ui;
mod util;
mod widget;

use app::App;
use std::{
    env::{self, set_current_dir},
    io,
};
use ui::start;

fn main() -> Result<(), io::Error> {
    // Process command line arg
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        set_current_dir(args[1].clone())?;
    }

    // Initialize the app
    let app = App::new();

    // Run the app
    start(app)?;

    Ok(())
}
