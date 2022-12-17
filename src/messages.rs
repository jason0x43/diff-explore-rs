use std::time::SystemTime;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Message {
    time: SystemTime,
    content: String,
}

impl Message {
    pub fn new(content: String) -> Message {
        Message {
            time: SystemTime::now(),
            content,
        }
    }

    pub fn content(&self) -> String {
        self.content.clone()
    }
}
