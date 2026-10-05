#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewBounds {
    pub line_count: usize,
    pub viewport_height: usize,
    pub widest_line: usize,
    pub viewport_width: usize,
}

impl ViewBounds {
    pub fn maximum_vertical_scroll(self) -> usize {
        self.line_count.saturating_sub(self.viewport_height)
    }

    pub fn maximum_horizontal_scroll(self) -> usize {
        self.widest_line.saturating_sub(self.viewport_width)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ScrollDown,
    ScrollUp,
    PageDown,
    PageUp,
    ScrollRight,
    ScrollLeft,
    GoToTop,
    GoToBottom,
    ToggleHelp,
    Quit,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ViewerState {
    pub vertical_scroll: usize,
    pub horizontal_scroll: usize,
    pub show_help: bool,
    pub should_quit: bool,
}

impl ViewerState {
    pub fn apply(&mut self, action: Action, bounds: ViewBounds) {
        match action {
            Action::ScrollDown => self.vertical_scroll = self.vertical_scroll.saturating_add(1),
            Action::ScrollUp => self.vertical_scroll = self.vertical_scroll.saturating_sub(1),
            Action::PageDown => {
                self.vertical_scroll = self
                    .vertical_scroll
                    .saturating_add(bounds.viewport_height.max(1));
            }
            Action::PageUp => {
                self.vertical_scroll = self
                    .vertical_scroll
                    .saturating_sub(bounds.viewport_height.max(1));
            }
            Action::ScrollRight => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_add(2);
            }
            Action::ScrollLeft => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_sub(2);
            }
            Action::GoToTop => self.vertical_scroll = 0,
            Action::GoToBottom => self.vertical_scroll = bounds.maximum_vertical_scroll(),
            Action::ToggleHelp => self.show_help = !self.show_help,
            Action::Quit => self.should_quit = true,
        }

        self.clamp(bounds);
    }

    pub fn clamp(&mut self, bounds: ViewBounds) {
        self.vertical_scroll = self.vertical_scroll.min(bounds.maximum_vertical_scroll());
        self.horizontal_scroll = self
            .horizontal_scroll
            .min(bounds.maximum_horizontal_scroll());
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, ViewBounds, ViewerState};

    fn bounds() -> ViewBounds {
        ViewBounds {
            line_count: 20,
            viewport_height: 5,
            widest_line: 30,
            viewport_width: 10,
        }
    }

    #[test]
    fn scrolling_clamps_to_document_boundaries() {
        let mut state = ViewerState::default();

        state.apply(Action::ScrollUp, bounds());
        state.apply(Action::ScrollLeft, bounds());
        assert_eq!(state.vertical_scroll, 0);
        assert_eq!(state.horizontal_scroll, 0);

        for _ in 0..100 {
            state.apply(Action::ScrollDown, bounds());
            state.apply(Action::ScrollRight, bounds());
        }

        assert_eq!(state.vertical_scroll, 15);
        assert_eq!(state.horizontal_scroll, 20);
    }

    #[test]
    fn page_and_document_actions_use_viewport_bounds() {
        let mut state = ViewerState::default();

        state.apply(Action::PageDown, bounds());
        assert_eq!(state.vertical_scroll, 5);

        state.apply(Action::GoToBottom, bounds());
        assert_eq!(state.vertical_scroll, 15);

        state.apply(Action::PageUp, bounds());
        assert_eq!(state.vertical_scroll, 10);

        state.apply(Action::GoToTop, bounds());
        assert_eq!(state.vertical_scroll, 0);
    }

    #[test]
    fn empty_and_tiny_views_do_not_underflow() {
        let tiny = ViewBounds {
            line_count: 0,
            viewport_height: 0,
            widest_line: 0,
            viewport_width: 0,
        };
        let mut state = ViewerState::default();

        state.apply(Action::GoToBottom, tiny);
        state.apply(Action::PageDown, tiny);
        state.apply(Action::ScrollRight, tiny);

        assert_eq!(state.vertical_scroll, 0);
        assert_eq!(state.horizontal_scroll, 0);
    }

    #[test]
    fn help_and_quit_are_explicit_state_transitions() {
        let mut state = ViewerState::default();

        state.apply(Action::ToggleHelp, bounds());
        assert!(state.show_help);
        state.apply(Action::ToggleHelp, bounds());
        assert!(!state.show_help);

        state.apply(Action::Quit, bounds());
        assert!(state.should_quit);
    }
}
