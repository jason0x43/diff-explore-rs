use std::sync::Mutex;
use std::time::SystemTime;

use once_cell::sync::OnceCell;

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

static MESSAGES: OnceCell<Mutex<Vec<Message>>> = OnceCell::new();

fn ensure_messages() -> &'static Mutex<Vec<Message>> {
    MESSAGES.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn console_log(message: &str) {
    ensure_messages().lock().unwrap().push(Message::new(message))
}

pub fn get_messages() -> Vec<Message> {
    ensure_messages().lock().unwrap().to_vec()
}
