use tui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Paragraph, Widget},
};

pub trait HasStatus {
    fn status(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct Location {
    pos: usize,
    total: usize,
}

impl Location {
    fn max_width(&self) -> u16 {
        (self.total.to_string().len() * 2 + 1) as u16
    }

    fn to_string(&self) -> String {
        format!("{}/{}", self.pos, self.total)
    }
}

pub struct StatusLine {
    status: String,
    location: Option<Location>,
}

impl StatusLine {
    pub fn new(status: String, location: Option<Location>) -> StatusLine {
        StatusLine { status, location }
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn set_location(&mut self, pos: usize, total: usize) {
        self.location = Some(Location { pos, total });
    }
}

pub struct StatusLineView<'a> {
    statusline: &'a StatusLine,
}

impl<'a> StatusLineView<'a> {
    pub fn new(statusline: &'a StatusLine) -> StatusLineView<'a> {
        StatusLineView { statusline }
    }
}

impl<'a> Widget for StatusLineView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let status_width = match &self.statusline.location {
            Some(loc) => area.width - (loc.max_width() + 2),
            _ => area.width,
        };

        let status = Paragraph::new(format!(" {}", self.statusline.status))
            .style(Style::default().bg(Color::Indexed(8)));
        Widget::render(
            status,
            Rect {
                width: status_width,
                ..area
            },
            buf,
        );

        if let Some(loc) = &self.statusline.location {
            let location = Paragraph::new(format!("{} ", loc.to_string()))
                .alignment(Alignment::Right)
                .style(
                    Style::default()
                        .bg(Color::Indexed(4))
                        .fg(Color::Indexed(0)),
                );
            Widget::render(
                location,
                Rect {
                    x: status_width,
                    width: loc.max_width() + 2,
                    ..area
                },
                buf,
            );
        }
    }
}
