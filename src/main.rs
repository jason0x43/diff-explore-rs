mod app;
mod git;
mod ui;
mod cursor;
mod commits;

use crate::app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::{io::{self, Stdout}, env::{self, set_current_dir}};
use tui::{backend::CrosstermBackend, Terminal};

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    app.should_quit = true;
                }
                _ => {}
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn run() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut app = App::new();
    app.load_commits();

    let res = run_app(&mut term, app);

    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        set_current_dir(args[1].clone())?;
    }

    run()?;
    Ok(())
}
