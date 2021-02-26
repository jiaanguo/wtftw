use super::with_borders_layout::WithBordersLayout;
use crate::layout::Layout;

pub struct NoBordersLayout;

impl NoBordersLayout {
    pub fn new(layout: Box<dyn Layout>) -> Box<dyn Layout> {
        WithBordersLayout::new(0, layout)
    }
}
