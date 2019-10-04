use crate::config::GeneralConfig;
use crate::core::stack::Stack;
use crate::layout::{Layout, LayoutMessage};
use crate::window_system::{Rectangle, Window, WindowSystem};


pub struct GapLayout {
    gap: u32,
    layout: Box<dyn Layout>,
}

impl GapLayout {
    pub fn new(gap: u32, layout: Box<dyn Layout>) -> Box<dyn Layout> {
        Box::new(GapLayout {
            gap: gap,
            layout: layout.copy(),
        })
    }
}

impl Layout for GapLayout {
    fn apply_layout(
        &mut self,
        window_system: &dyn WindowSystem,
        screen: Rectangle,
        config: &GeneralConfig,
        stack: &Option<Stack<Window>>,
    ) -> Vec<(Window, Rectangle)> {
        let layout = self
            .layout
            .apply_layout(window_system, screen, config, stack);

        let g = self.gap;
        layout
            .iter()
            .map(|&(win, Rectangle(x, y, w, h))| {
                (
                    win,
                    Rectangle(x + g as i32, y + g as i32, w - 2 * g, h - 2 * g),
                )
            })
            .collect()
    }

    fn apply_message(
        &mut self,
        message: LayoutMessage,
        window_system: &dyn WindowSystem,
        stack: &Option<Stack<Window>>,
        config: &GeneralConfig,
    ) -> bool {
        match message {
            LayoutMessage::IncreaseGap => {
                self.gap += 1;
                true
            }
            LayoutMessage::DecreaseGap => {
                if self.gap > 0 {
                    self.gap -= 1;
                }
                true
            }
            _ => self
                .layout
                .apply_message(message, window_system, stack, config),
        }
    }

    fn description(&self) -> String {
        self.layout.description()
    }

    fn copy(&self) -> Box<dyn Layout> {
        Box::new(GapLayout {
            gap: self.gap,
            layout: self.layout.copy(),
        })
    }
}
