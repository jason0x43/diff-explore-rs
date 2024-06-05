mod app;
mod error;
mod events;
mod git;
mod graph;
mod list;
mod logging;
mod search;
mod stack;
mod string;
mod time;
mod ui;
mod views;

use app::App;
use error::AppError;
use git::is_git_repo;
use std::{
    env::{self, set_current_dir},
    process::exit,
};

fn main() -> Result<(), AppError> {
    logging::initialize_logging()?;

    // Process command line arg
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        set_current_dir(args[1].clone())?;
    }

    // Verify that we are in a git repo
    if !is_git_repo() {
        println!("Not a git repo");
        exit(1);
    }

    // Initialize the app
    let mut app = App::new()?;

    tracing::info!("Starting app");

    // Run the app
    app.start();

    Ok(())
}
