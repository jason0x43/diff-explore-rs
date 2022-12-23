use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
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
    Frame, Terminal,
};

use crate::{
    app::{App, View},
    commits::CommitsView,
    console::ConsoleView,
    diff::DiffView,
    events::{Events, InputEvent},
    stack::Stack,
    stats::StatsView,
    widget::RenderBorderedWidget,
};

/// Draw the UI
pub fn draw(f: &mut Frame<CrosstermBackend<Stdout>>, app: &mut App) {
    let constraints = if app.should_show_console() {
        [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref()
    } else {
        [Constraint::Percentage(100)].as_ref()
    };

    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.size());

    let content_rect = parts[0];

    match app.views.top() {
        Some(View::Commits(v)) => {
            let w = CommitsView::new(v);
            f.draw_widget(w, "Commits", content_rect);
        }
        Some(View::Stats(v)) => {
            let w = StatsView::new(v);
            f.draw_widget(w, "Stats", content_rect);
        }
        Some(View::Diff(v)) => {
            let w = DiffView::new(v);
            f.draw_widget(w, "Diff", content_rect);
        }
        _ => {}
    };

    if app.should_show_console() {
        let console = ConsoleView::new(&mut app.console);
        f.render_widget(console, parts[1]);
    }
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
    let events = Events::new();

    loop {
        term.draw(|f| draw(f, &mut app))?;

        match events.next().unwrap() {
            InputEvent::Input(key) => app.do_action(key),
            InputEvent::Resize => {}
            InputEvent::FileChange(event) => match app.views.top() {
                Some(View::Diff(v)) => match event.kind {
                    notify::event::EventKind::Modify(modify_kind) => {
                        match modify_kind {
                            notify::event::ModifyKind::Data(_change) => {
                                if v.is_in_list(&event.paths) {
                                    v.refresh();
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
        };

        if app.should_quit() {
            break;
        }
    }

    restore_term(term)?;

    Ok(())
}
