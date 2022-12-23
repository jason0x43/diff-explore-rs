use tui::layout::Rect;

pub trait Truncatable {
    fn ellipses(&self, width: usize) -> Self;
}

impl Truncatable for String {
    fn ellipses(&self, width: usize) -> Self {
        if self.len() > width {
            format!("{}...", &self[..width - 3])
        } else {
            String::from(self)
        }
    }
}

pub struct Dimensions {
    pub width: u16,
    pub height: u16,
}

impl Dimensions {
    fn new(width: u16, height: u16) -> Dimensions {
        Dimensions { width, height }
    }
}

pub trait HasDimensions {
    fn dimensions(&self) -> Dimensions;
    fn resized_dimensions(&self, x_delta: i16, y_delta: i16) -> Dimensions;
}

impl HasDimensions for Rect {
    fn dimensions(&self) -> Dimensions {
        Dimensions::new(self.width, self.height)
    }

    fn resized_dimensions(&self, x_delta: i16, y_delta: i16) -> Dimensions {
        let new_width = if x_delta < 0 {
            self.width - x_delta.abs() as u16
        } else {
            self.width + x_delta.abs() as u16
        };
        let new_height = if y_delta < 0 {
            self.height - y_delta.abs() as u16
        } else {
            self.height + y_delta.abs() as u16
        };
        Dimensions::new(new_width, new_height)
    }
}
