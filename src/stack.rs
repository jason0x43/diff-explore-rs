use std::collections::LinkedList;

pub trait Stack<T> {
    fn push(&mut self, value: T);
    fn pop(&mut self) -> Option<T>;
    fn top(&self) -> Option<&T>;
}

impl<T> Stack<T> for LinkedList<T> {
    fn push(&mut self, value: T) {
        self.push_back(value);
    }

    fn pop(&mut self) -> Option<T> {
        self.pop_back()
    }

    fn top(&self) -> Option<&T> {
        self.back()
    }
}
