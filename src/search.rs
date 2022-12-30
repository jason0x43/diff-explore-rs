use crate::list::{ListInfo, ListScroll};

pub trait Search: ListInfo + ListScroll {
    fn set_search(&mut self, search: Option<String>);
    fn get_search(&self) -> Option<String>;
    fn is_match(&self, index: usize) -> bool;

    fn search_next(&mut self) {
        if self.get_search().is_some() {
            for i in self.list_pos() + 1..self.list_count() {
                if self.is_match(i) {
                    self.scroll_to(i);
                    break;
                }
            }
        }
    }

    fn search_prev(&mut self) {
        if self.get_search().is_some() && self.list_pos() > 0 {
            for i in (0..self.list_pos()).rev() {
                if self.is_match(i) {
                    self.scroll_to(i);
                    break;
                }
            }
        }
    }
}
