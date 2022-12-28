mod app;
mod events;
mod git;
mod graph;
mod stack;
mod time;
mod ui;
mod views;
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
