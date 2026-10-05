#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerAction {
    MoveDown,
    MoveUp,
    Input(char),
    Backspace,
    Select,
    Quit,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PickerState {
    pub filter: String,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub scroll: usize,
    pub selected_topic: Option<String>,
    pub should_quit: bool,
}

impl PickerState {
    pub fn new(topics: &[String]) -> Self {
        let mut state = Self::default();
        state.refresh(topics);
        state
    }

    pub fn apply(&mut self, action: PickerAction, topics: &[String], viewport_height: usize) {
        match action {
            PickerAction::MoveDown => {
                self.selected = self
                    .selected
                    .saturating_add(1)
                    .min(self.filtered.len().saturating_sub(1));
            }
            PickerAction::MoveUp => {
                self.selected = self.selected.saturating_sub(1);
            }
            PickerAction::Input(character) => {
                self.filter.push(character);
                self.refresh(topics);
            }
            PickerAction::Backspace => {
                self.filter.pop();
                self.refresh(topics);
            }
            PickerAction::Select => {
                self.selected_topic = self
                    .filtered
                    .get(self.selected)
                    .and_then(|index| topics.get(*index))
                    .cloned();
                self.should_quit = self.selected_topic.is_some();
            }
            PickerAction::Quit => self.should_quit = true,
        }

        self.keep_selection_visible(viewport_height);
    }

    pub fn selected_topic<'a>(&self, topics: &'a [String]) -> Option<&'a str> {
        self.filtered
            .get(self.selected)
            .and_then(|index| topics.get(*index))
            .map(String::as_str)
    }

    fn refresh(&mut self, topics: &[String]) {
        let filter = self.filter.to_lowercase();
        self.filtered = topics
            .iter()
            .enumerate()
            .filter_map(|(index, topic)| topic.to_lowercase().contains(&filter).then_some(index))
            .collect();
        self.selected = 0;
        self.scroll = 0;
    }

    fn keep_selection_visible(&mut self, viewport_height: usize) {
        if self.filtered.is_empty() {
            self.selected = 0;
            self.scroll = 0;
            return;
        }

        self.selected = self.selected.min(self.filtered.len() - 1);
        let height = viewport_height.max(1);
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll.saturating_add(height) {
            self.scroll = self.selected + 1 - height;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PickerAction, PickerState};

    fn topics() -> Vec<String> {
        vec![
            String::from("algorithms/binary-search"),
            String::from("commands/cargo-test"),
            String::from("concepts/ownership"),
        ]
    }

    #[test]
    fn starts_with_all_topics_in_source_order() {
        let state = PickerState::new(&topics());

        assert_eq!(state.filtered, vec![0, 1, 2]);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn filters_case_insensitively_and_clears_with_backspace() {
        let topics = topics();
        let mut state = PickerState::new(&topics);

        for character in "CARGO".chars() {
            state.apply(PickerAction::Input(character), &topics, 5);
        }
        assert_eq!(state.filtered, vec![1]);
        assert_eq!(state.selected_topic(&topics), Some("commands/cargo-test"));

        for _ in 0..5 {
            state.apply(PickerAction::Backspace, &topics, 5);
        }
        assert_eq!(state.filtered, vec![0, 1, 2]);
    }

    #[test]
    fn movement_clamps_and_scrolls_to_keep_selection_visible() {
        let topics = topics();
        let mut state = PickerState::new(&topics);

        for _ in 0..10 {
            state.apply(PickerAction::MoveDown, &topics, 2);
        }
        assert_eq!(state.selected, 2);
        assert_eq!(state.scroll, 1);

        for _ in 0..10 {
            state.apply(PickerAction::MoveUp, &topics, 2);
        }
        assert_eq!(state.selected, 0);
        assert_eq!(state.scroll, 0);
    }

    #[test]
    fn selecting_returns_the_visible_topic() {
        let topics = topics();
        let mut state = PickerState::new(&topics);
        state.apply(PickerAction::MoveDown, &topics, 5);
        state.apply(PickerAction::Select, &topics, 5);

        assert_eq!(state.selected_topic.as_deref(), Some("commands/cargo-test"));
        assert!(state.should_quit);
    }

    #[test]
    fn empty_results_cannot_be_selected() {
        let topics = topics();
        let mut state = PickerState::new(&topics);
        state.apply(PickerAction::Input('z'), &topics, 5);
        state.apply(PickerAction::Select, &topics, 5);

        assert!(state.selected_topic.is_none());
        assert!(!state.should_quit);
    }
}
