use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use list_helper_core::ListInfo;
use std::io::{self, Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame, Terminal,
};

use crate::{
    app::{App, View},
    events::{Events, InputEvent},
    stack::Stack,
    views::{
        commits::CommitsView,
        console::ConsoleView,
        diff::{DiffView, DiffViewOpts},
        stats::StatsView,
        statusline::{Status, StatusLineView},
    },
    widget::WidgetWithBlock,
};

/// Draw a widget with a border
fn draw_framed_widget<'a, W>(
    f: &mut Frame<CrosstermBackend<Stdout>>,
    mut w: W,
    t: &'a str,
    r: Rect,
) where
    W: WidgetWithBlock<'a>,
{
    w.block(Block::default().borders(Borders::ALL).title(t));
    f.render_widget(w, r);
}

/// Draw the UI
pub fn draw(f: &mut Frame<CrosstermBackend<Stdout>>, app: &mut App) {
    let constraints = if app.should_show_console() {
        [
            Constraint::Percentage(50),
            Constraint::Percentage(50),
            Constraint::Length(1),
        ]
        .as_ref()
    } else {
        [Constraint::Percentage(100), Constraint::Length(1)].as_ref()
    };

    let size = f.size();
    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(Rect {
            height: size.height - 1,
            ..size
        });

    let content_rect = parts[0];

    match app.views.top() {
        Some(View::Commits(v)) => {
            app.statusline.set_status(v.status());
            app.statusline.set_location(v.list_pos(), v.list_count());
            let w = CommitsView::new(v);
            f.render_widget(w, content_rect);
        }
        Some(View::Stats(v)) => {
            app.statusline.set_status(v.status());
            app.statusline.set_location(v.list_pos(), v.list_count());
            let w = StatsView::new(v);
            f.render_widget(w, content_rect);
        }
        Some(View::Diff(v)) => {
            app.statusline.set_status(v.status());
            app.statusline.set_location(v.list_pos(), v.list_count());
            let w = DiffView::new(
                v,
                Some(DiffViewOpts {
                    tab_width: app.tab_width,
                }),
            );
            f.render_widget(w, content_rect);
        }
        _ => {}
    };

    if app.should_show_console() {
        let console = ConsoleView::new(&mut app.console);
        draw_framed_widget(f, console, "Console", parts[1]);
    }

    let statusline = StatusLineView::new(&app.statusline);
    f.render_widget(
        statusline,
        Rect {
            x: 0,
            y: size.height - 1,
            width: size.width,
            height: 1,
        },
    );
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
