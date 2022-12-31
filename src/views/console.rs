use std::sync::Mutex;
use std::time::SystemTime;

use once_cell::sync::Lazy;
use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Paragraph, Widget},
};

use crate::list::{ListInfo, ListScroll};
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
}

impl ListInfo for Console {
    fn list_count(&self) -> usize {
        get_num_lines()
    }

    fn list_pos(&self) -> usize {
        self.offset
    }

    fn set_list_pos(&mut self, pos: usize) {
        self.offset = pos;
    }
}

impl ListScroll for Console {
    fn height(&self) -> usize {
        self.height
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

        let mut console = Paragraph::new(
            messages
                .iter()
                .map(|m| m.content.clone())
                .collect::<Vec<String>>()
                .join("\n"),
        )
        .scroll((self.console.offset as u16, 0));

        if let Some(b) = self.block {
            console = console.block(b);
            self.console.height -= 2;
        }

        Widget::render(console, area, buf);
    }
}

static MESSAGES: Lazy<Mutex<Vec<Message>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
static NUM_MESSAGE_LINES: Lazy<Mutex<usize>> =
    Lazy::new(|| Mutex::new(0));

pub fn console_log(message: &str) {
    MESSAGES.lock().unwrap().push(Message::new(message));
    let mut count = NUM_MESSAGE_LINES.lock().unwrap();
    *count += message.lines().count();
}

fn get_messages() -> Vec<Message> {
    MESSAGES.lock().unwrap().to_vec()
}

fn get_num_lines() -> usize {
    NUM_MESSAGE_LINES.lock().unwrap().clone()
}

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {{
        $crate::views::console::console_log(&format!($($t)*));
    }};
}
