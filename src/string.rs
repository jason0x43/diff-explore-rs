pub trait Ellipses {
    fn ellipses(&self, max: usize) -> String;
}

impl Ellipses for String {
    fn ellipses(&self, max: usize) -> String {
        if self.len() <= max {
            self.clone()
        } else {
            format!("{}...", &self[..max - 3])
        }
    }
}
