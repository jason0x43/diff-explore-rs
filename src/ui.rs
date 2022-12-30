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
        let mut console = ConsoleView::new(&mut app.console);
        console.block(Block::default().borders(Borders::ALL).title("Console"));
        f.render_widget(console, parts[1]);
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

pub struct Ui {
    term: Terminal<CrosstermBackend<Stdout>>,
}

impl Ui {
    pub fn new() -> Ui {
        enable_raw_mode().unwrap();
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend).unwrap();

        Ui { term }
    }

    pub fn update(&mut self, app: &mut App) {
        self.term.draw(|f| draw(f, app)).unwrap();
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(
            self.term.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
        self.term.show_cursor().unwrap();
    }
}
