use crate::config::GeneralConfig;
use crate::core::stack::Stack;
use crate::layout::{mirror_rect, Layout, LayoutMessage};
use crate::window_system::{Rectangle, Window, WindowSystem};

/// A simple layout container that just
/// rotates the layout of its contained layout
/// by 90° clockwise
pub struct MirrorLayout {
    pub layout: Box<dyn Layout>,
}

impl MirrorLayout {
    /// Create a new MirrorLayout containing the given layout
    pub fn new(layout: Box<dyn Layout>) -> Box<dyn Layout> {
        Box::new(MirrorLayout { layout: layout })
    }
}

impl Layout for MirrorLayout {
    fn apply_layout(
        &mut self,
        w: &dyn WindowSystem,
        screen: Rectangle,
        config: &GeneralConfig,
        stack: &Option<Stack<Window>>,
    ) -> Vec<(Window, Rectangle)> {
        // Rotate the screen, apply the layout, ...
        self.layout
            .apply_layout(w, mirror_rect(&screen), config, stack)
            .iter()
            // and then rotate all resulting windows by 90° clockwise
            .map(|&(w, r)| (w, mirror_rect(&r)))
            .collect()
    }

    fn apply_message(
        &mut self,
        message: LayoutMessage,
        window_system: &dyn WindowSystem,
        stack: &Option<Stack<Window>>,
        config: &GeneralConfig,
    ) -> bool {
        self.layout
            .apply_message(message, window_system, stack, config)
    }

    fn description(&self) -> String {
        self.layout.description()
    }

    fn copy(&self) -> Box<dyn Layout> {
        Box::new(MirrorLayout {
            layout: self.layout.copy(),
        })
    }
}
