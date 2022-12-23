use std::{
    cell::RefCell,
    cmp::min,
    path::{Path, PathBuf},
};

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::{
    git::{git_diff_file, CommitRange, DiffFile, DiffLine, Stat},
    widget::WidgetWithBlock,
};

#[derive(Debug, Clone)]
pub struct Diff {
    list: ListState,
    height: usize,
    offset: usize,
    diff: DiffFile,
    range: CommitRange,
    stat: Stat,
}

impl Diff {
    pub fn new(stat: &Stat, range: &CommitRange) -> Diff {
        let diff = git_diff_file(&stat.path, &stat.old_path, &range, None);

        Diff {
            diff,
            height: 0,
            offset: 0,
            list: ListState::default(),
            stat: stat.clone(),
            range: range.clone(),
        }
    }

    pub fn is_in_list(&self, paths: &Vec<PathBuf>) -> bool {
        let buf = Path::new(&self.stat.path).canonicalize().unwrap();
        paths.contains(&buf)
    }

    pub fn refresh(&mut self) {
        self.diff = git_diff_file(
            &self.stat.path,
            &self.stat.old_path,
            &self.range,
            None,
        );
    }

    pub fn scroll_up(&mut self) {
        let delta = min(1, self.offset);
        self.offset -= delta;
    }

    pub fn page_up(&mut self) {
        let delta = min(self.height, self.offset);
        self.offset -= delta;
    }

    pub fn scroll_down(&mut self) {
        if self.diff.lines.len() - self.offset > self.height {
            let limit = self.diff.lines.len() - self.offset - self.height;
            let delta = min(limit, 1);
            self.offset += delta;
        }
    }

    pub fn page_down(&mut self) {
        if self.diff.lines.len() - self.offset > self.height {
            let limit = self.diff.lines.len() - self.offset - self.height;
            let delta = min(limit, self.height);
            self.offset += delta;
        }
    }
}

/// The Widget used to render Stats
pub struct DiffView<'a> {
    diff: &'a RefCell<Diff>,
    block: Option<Block<'a>>,
}

impl<'a> DiffView<'a> {
    pub fn new(diff: &RefCell<Diff>) -> DiffView {
        DiffView { diff, block: None }
    }
}

impl<'a> WidgetWithBlock<'a> for DiffView<'a> {
    fn block(&mut self, block: Block<'a>) {
        self.block = Some(block);
    }
}

fn line_spans<'a>(
    old_color: u8,
    new_color: u8,
    line_color: u8,
    old: u32,
    new: u32,
    line: &str,
) -> Vec<Span<'a>> {
    [
        Span::styled(
            old.to_string(),
            Style::default().fg(Color::Indexed(old_color)),
        ),
        Span::from(" "),
        Span::styled(
            new.to_string(),
            Style::default().fg(Color::Indexed(new_color)),
        ),
        Span::from(" "),
        Span::styled(
            String::from(&line[1..]),
            Style::default().fg(Color::Indexed(line_color)),
        ),
    ]
    .into()
}

impl<'a> Widget for DiffView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let list = {
            let diff = self.diff.borrow();
            let items: Vec<ListItem> = diff.diff.lines[diff.offset..]
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let line_nr = diff.offset + i;
                    if c.len() > 0 {
                        let spans: Vec<Span> =
                            match &diff.diff.line_meta[line_nr] {
                                DiffLine::Add(meta) => {
                                    line_spans(8, 7, 2, meta.old, meta.new, c)
                                }
                                DiffLine::Del(meta) => {
                                    line_spans(7, 8, 1, meta.old, meta.new, c)
                                }
                                DiffLine::Same(meta) => {
                                    line_spans(7, 7, 17, meta.old, meta.new, c)
                                }
                                DiffLine::Start => [Span::styled(
                                    c.clone(),
                                    Style::default().fg(Color::Indexed(3)),
                                )]
                                .into(),
                                DiffLine::Hunk => [Span::styled(
                                    c.clone(),
                                    Style::default().fg(Color::Indexed(6)),
                                )]
                                .into(),
                                _ => [Span::from(c.clone())].into(),
                            };
                        ListItem::new(Spans::from(spans))
                    } else {
                        ListItem::new(Spans::from(vec![Span::from("")]))
                    }
                })
                .collect();

            List::new(items).block(self.block.unwrap())
        };

        let mut diff_mut = self.diff.borrow_mut();
        diff_mut.height = area.height as usize;
        StatefulWidget::render(list, area, buf, &mut diff_mut.list);
    }
}
