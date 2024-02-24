use ratatui::widgets::{Block, Widget};

pub trait WidgetWithBlock<'a>: Widget {
    fn block(&mut self, block: Block<'a>);
}
