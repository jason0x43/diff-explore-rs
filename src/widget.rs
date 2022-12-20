use tui::{
    layout::Rect,
    widgets::{Block, Borders, Widget},
    Frame, backend::Backend,
};

pub trait WidgetWithBlock<'a>: Widget {
    fn block(&mut self, block: Block<'a>);
}

pub trait RenderBorderedWidget {
    fn draw_widget<'a, W>(&mut self, w: W, t: &'a str, r: Rect)
    where
        W: WidgetWithBlock<'a>;
}

impl<B> RenderBorderedWidget for Frame<'_, B> where B: Backend {
    fn draw_widget<'a, W>(&mut self, mut w: W, t: &'a str, r: Rect)
    where
        W: WidgetWithBlock<'a>,
    {
        w.block(Block::default().borders(Borders::ALL).title(t));
        Frame::render_widget(self, w, r);
    }
}
