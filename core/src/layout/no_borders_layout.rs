use wtftw::layout::Layout;
use super::with_borders_layout::WithBordersLayout;

pub struct NoBordersLayout;

impl NoBordersLayout {
    pub fn new(layout: Box<dyn Layout>) -> Box<dyn Layout> {
        WithBordersLayout::new(0, layout)
    }
}
