use std::time::SystemTime;
use std::{cmp::min, sync::Mutex};

use once_cell::sync::Lazy;
use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Paragraph, Widget},
};

use crate::widget::WidgetWithBlock;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Message {
    time: SystemTime,
    content: String,
}

impl Message {
    pub fn new(content: &str) -> Message {
        Message {
            time: SystemTime::now(),
            content: String::from(content),
        }
    }

    pub fn content(&self) -> String {
        self.content.clone()
    }
}

pub struct Console {
    offset: usize,
    height: usize,
}

impl Console {
    pub fn new() -> Console {
        Console {
            offset: 0,
            height: 0,
        }
    }

    pub fn scroll_up(&mut self) {
        let delta = min(1, self.offset);
        self.offset -= delta;
    }

    pub fn scroll_down(&mut self) {
        let count = get_message_count();
        if count - self.offset > self.height {
            let limit = count - self.offset - self.height;
            let delta = min(limit, 1);
            self.offset += delta;
        }
    }
}

pub struct ConsoleView<'a> {
    console: &'a mut Console,
    block: Option<Block<'a>>,
}

impl<'a> ConsoleView<'a> {
    pub fn new(console: &'a mut Console) -> ConsoleView<'a> {
        ConsoleView {
            console,
            block: None,
        }
    }
}

impl<'a> WidgetWithBlock<'a> for ConsoleView<'a> {
    fn block(&mut self, block: Block<'a>) {
        self.block = Some(block);
    }
}

impl<'a> Widget for ConsoleView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let messages = get_messages();
        self.console.height = area.height as usize;

        let start = self.console.offset;
        let end = if start + self.console.height < messages.len() {
            start + self.console.height
        } else {
            messages.len()
        };
        let lines: Vec<String> =
            messages[start..end].iter().map(|a| a.content()).collect();

        let mut console = Paragraph::new(lines.join("\n"));

        if let Some(b) = self.block {
            console = console.block(b);
        }

        Widget::render(console, area, buf);
    }
}

static MESSAGES: Lazy<Mutex<Vec<Message>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub fn console_log(message: &str) {
    MESSAGES.lock().unwrap().push(Message::new(message))
}

fn get_messages() -> Vec<Message> {
    MESSAGES.lock().unwrap().to_vec()
}

fn get_message_count() -> usize {
    MESSAGES.lock().unwrap().len()
}

#[macro_export]
macro_rules! console {
    ($($t:tt)*) => {{
        console::console_log(&format!($($t)*));
    }};
}
