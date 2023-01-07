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
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Span,
    widgets::{Block, Borders},
    Frame, Terminal,
};

use crate::{
    app::{App, View},
    list::ListInfo,
    search::Search,
    stack::Stack,
    views::{
        commitlog::CommitsView,
        console::ConsoleView,
        diff::{DiffView, DiffViewOpts},
        stats::StatsView,
        statusline::{Status, StatusLineView},
    },
    widget::WidgetWithBlock,
};

/// Draw the UI
fn draw(f: &mut Frame<CrosstermBackend<Stdout>>, app: &mut App) {
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
    let search = app.entering_search();

    match app.views.top() {
        Some(View::CommitLog(v)) => {
            if let Some(s) = search {
                app.statusline.set_status(search_status(s));
            } else {
                app.statusline.set_status(v.status());
            }
            app.statusline.set_location(v.list_pos(), v.list_count());
            v.set_search(app.search.clone());
            f.render_widget(CommitsView::new(v), content_rect);
        }
        Some(View::Stats(v)) => {
            if let Some(s) = search {
                app.statusline.set_status(search_status(s));
            } else {
                app.statusline.set_status(v.status());
            }
            app.statusline.set_location(v.list_pos(), v.list_count());
            v.set_search(app.search.clone());
            f.render_widget(StatsView::new(v), content_rect);
        }
        Some(View::Diff(v)) => {
            if let Some(s) = search {
                app.statusline.set_status(search_status(s));
            } else {
                app.statusline.set_status(v.status());
            }
            app.statusline.set_location(v.list_pos(), v.list_count());
            v.set_search(app.search.clone());
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

pub fn highlight_spans<'a>(
    spans: Vec<Span<'a>>,
    hl_text: &String,
    hl_style: Style,
) -> Vec<Span<'a>> {
    let mut new_spans: Vec<Span> = vec![];
    let text = spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect::<Vec<&str>>()
        .join("");

    if text.contains(hl_text) {
        let mut styles: Vec<&Style> = vec![];
        for span in spans.iter() {
            for _ in 0..span.content.len() {
                styles.push(&span.style);
            }
        }

        let parts = text.split(&hl_text.clone()).collect::<Vec<&str>>();
        let mut offset = 0;
        for i in 0..parts.len() - 1 {
            offset += parts[i].len();
            for x in offset..offset + hl_text.len() {
                styles[x] = &hl_style;
            }
            offset += hl_text.len();
        }

        let mut start = 0;
        let mut style = styles[0];
        for i in 0..text.len() {
            if styles[i] != style {
                new_spans
                    .push(Span::styled(String::from(&text[start..i]), *style));
                start = i;
                style = styles[i];
            }
        }
        new_spans.push(Span::styled(
            String::from(&text[start..text.len()]),
            *styles[text.len() - 1],
        ));

        new_spans
    } else {
        spans.clone()
    }
}

fn search_status(query: String) -> String {
    format!("/{}", query)
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

    pub fn stop(&mut self) {
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
