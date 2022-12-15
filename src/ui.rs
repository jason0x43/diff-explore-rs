use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{self, Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use crate::app::{App, View};

/// Draw the UI
pub fn draw(f: &mut Frame<CrosstermBackend<Stdout>>, app: &App) {
    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
        )
        .split(f.size());

    let content_widget = match &app.view {
        View::Commits => app.commits.to_widget(),
    };
    let content_title = match &app.view {
        View::Commits => "Commits",
    };

    let content = content_widget
        .block(Block::default().borders(Borders::ALL).title(content_title));
    f.render_widget(content, parts[0]);

    let console = Paragraph::new(app.messages.join("\n"))
        .block(Block::default().title("Console").borders(Borders::ALL));
    f.render_widget(console, parts[1]);
}

fn setup_term() -> Result<Terminal<CrosstermBackend<Stdout>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_term(
    mut term: Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    Ok(())
}

/// Setup the terminal and start the render loop
pub fn start(mut app: App) -> Result<(), io::Error> {
    let mut term = setup_term()?;

    app.load_commits();

    loop {
        term.draw(|f| draw(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => app.quit(),
                KeyCode::Up => app.commits.cursor_up(),
                KeyCode::Down => app.commits.cursor_down(),
                _ => app
                    .messages
                    .push(format!("App has {} items", app.commits.len())),
            }
        }

        if app.should_quit() {
            break;
        }
    }

    restore_term(term)?;

    Ok(())
}
