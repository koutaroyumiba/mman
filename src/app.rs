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
    EnterSearch,
    SearchCharacter(char),
    SearchBackspace,
    SubmitSearch,
    CancelSearch,
    ClearSearch,
    NextMatch,
    PreviousMatch,
    ToggleHelp,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewerMode {
    #[default]
    Reading,
    SearchInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SearchState {
    pub query: String,
    pub input: String,
    pub matches: Vec<SearchMatch>,
    pub current: Option<usize>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ViewerState {
    pub vertical_scroll: usize,
    pub horizontal_scroll: usize,
    pub show_help: bool,
    pub should_quit: bool,
    pub mode: ViewerMode,
    pub search: SearchState,
}

impl ViewerState {
    pub fn apply(&mut self, action: Action, bounds: ViewBounds, lines: &[String]) {
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
            Action::EnterSearch => {
                self.mode = ViewerMode::SearchInput;
                self.search.input.clear();
            }
            Action::SearchCharacter(character) => self.search.input.push(character),
            Action::SearchBackspace => {
                self.search.input.pop();
            }
            Action::SubmitSearch => {
                self.mode = ViewerMode::Reading;
                self.search.query = std::mem::take(&mut self.search.input);
                self.refresh_search(lines);
                self.reveal_current_match(bounds);
            }
            Action::CancelSearch => {
                self.mode = ViewerMode::Reading;
                self.search.input.clear();
            }
            Action::ClearSearch => {
                self.search = SearchState::default();
            }
            Action::NextMatch => self.move_match(1, bounds),
            Action::PreviousMatch => self.move_match(-1, bounds),
            Action::ToggleHelp => self.show_help = !self.show_help,
            Action::Quit => self.should_quit = true,
        }

        self.clamp(bounds);
    }

    pub fn refresh_search(&mut self, lines: &[String]) {
        let previous = self
            .search
            .current
            .and_then(|index| self.search.matches.get(index))
            .cloned();
        self.search.matches = find_matches(lines, &self.search.query);
        self.search.current = previous
            .and_then(|item| {
                self.search
                    .matches
                    .iter()
                    .position(|candidate| *candidate == item)
            })
            .or((!self.search.matches.is_empty()).then_some(0));
    }

    pub fn clamp(&mut self, bounds: ViewBounds) {
        self.vertical_scroll = self.vertical_scroll.min(bounds.maximum_vertical_scroll());
        self.horizontal_scroll = self
            .horizontal_scroll
            .min(bounds.maximum_horizontal_scroll());
    }

    fn move_match(&mut self, direction: isize, bounds: ViewBounds) {
        if self.search.matches.is_empty() {
            self.search.current = None;
            return;
        }

        let count = self.search.matches.len();
        let current = self.search.current.unwrap_or(0);
        self.search.current = Some(if direction > 0 {
            (current + 1) % count
        } else {
            (current + count - 1) % count
        });
        self.reveal_current_match(bounds);
    }

    fn reveal_current_match(&mut self, bounds: ViewBounds) {
        let Some(line) = self
            .search
            .current
            .and_then(|index| self.search.matches.get(index))
            .map(|item| item.line)
        else {
            return;
        };

        if line < self.vertical_scroll {
            self.vertical_scroll = line;
        } else if line
            >= self
                .vertical_scroll
                .saturating_add(bounds.viewport_height.max(1))
        {
            self.vertical_scroll = line.saturating_sub(bounds.viewport_height.saturating_sub(1));
        }
    }
}

fn find_matches(lines: &[String], query: &str) -> Vec<SearchMatch> {
    if query.is_empty() {
        return Vec::new();
    }

    let folded_query = query.to_lowercase();
    let mut matches = Vec::new();

    for (line, text) in lines.iter().enumerate() {
        let (folded_text, byte_map) = fold_with_byte_map(text);
        for (start, matched) in folded_text.match_indices(&folded_query) {
            let end = start + matched.len();
            let Some(original_start) = byte_map.get(start).map(|range| range.0) else {
                continue;
            };
            let Some(original_end) = byte_map.get(end.saturating_sub(1)).map(|range| range.1)
            else {
                continue;
            };
            matches.push(SearchMatch {
                line,
                start: original_start,
                end: original_end,
            });
        }
    }

    matches
}

fn fold_with_byte_map(text: &str) -> (String, Vec<(usize, usize)>) {
    let mut folded = String::new();
    let mut byte_map = Vec::new();

    for (start, character) in text.char_indices() {
        let end = start + character.len_utf8();
        for folded_character in character.to_lowercase() {
            folded.push(folded_character);
            byte_map.extend(std::iter::repeat_n(
                (start, end),
                folded_character.len_utf8(),
            ));
        }
    }

    (folded, byte_map)
}

#[cfg(test)]
mod tests {
    use super::{Action, ViewBounds, ViewerMode, ViewerState};

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

        state.apply(Action::ScrollUp, bounds(), &[]);
        state.apply(Action::ScrollLeft, bounds(), &[]);
        assert_eq!(state.vertical_scroll, 0);
        assert_eq!(state.horizontal_scroll, 0);

        for _ in 0..100 {
            state.apply(Action::ScrollDown, bounds(), &[]);
            state.apply(Action::ScrollRight, bounds(), &[]);
        }

        assert_eq!(state.vertical_scroll, 15);
        assert_eq!(state.horizontal_scroll, 20);
    }

    #[test]
    fn page_and_document_actions_use_viewport_bounds() {
        let mut state = ViewerState::default();

        state.apply(Action::PageDown, bounds(), &[]);
        assert_eq!(state.vertical_scroll, 5);

        state.apply(Action::GoToBottom, bounds(), &[]);
        assert_eq!(state.vertical_scroll, 15);

        state.apply(Action::PageUp, bounds(), &[]);
        assert_eq!(state.vertical_scroll, 10);

        state.apply(Action::GoToTop, bounds(), &[]);
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

        state.apply(Action::GoToBottom, tiny, &[]);
        state.apply(Action::PageDown, tiny, &[]);
        state.apply(Action::ScrollRight, tiny, &[]);

        assert_eq!(state.vertical_scroll, 0);
        assert_eq!(state.horizontal_scroll, 0);
    }

    #[test]
    fn help_and_quit_are_explicit_state_transitions() {
        let mut state = ViewerState::default();

        state.apply(Action::ToggleHelp, bounds(), &[]);
        assert!(state.show_help);
        state.apply(Action::ToggleHelp, bounds(), &[]);
        assert!(!state.show_help);

        state.apply(Action::Quit, bounds(), &[]);
        assert!(state.should_quit);
    }

    #[test]
    fn search_is_case_insensitive_and_navigates_with_wraparound() {
        let lines = vec![
            String::from("  Alpha alpha"),
            String::from("other"),
            String::from("ALPHA last"),
        ];
        let mut state = ViewerState::default();

        state.apply(Action::EnterSearch, bounds(), &lines);
        for character in "alpha".chars() {
            state.apply(Action::SearchCharacter(character), bounds(), &lines);
        }
        state.apply(Action::SubmitSearch, bounds(), &lines);

        assert_eq!(state.mode, ViewerMode::Reading);
        assert_eq!(state.search.matches.len(), 3);
        assert_eq!(state.search.current, Some(0));

        state.apply(Action::PreviousMatch, bounds(), &lines);
        assert_eq!(state.search.current, Some(2));
        assert_eq!(state.vertical_scroll, 0);
        state.apply(Action::NextMatch, bounds(), &lines);
        assert_eq!(state.search.current, Some(0));
    }

    #[test]
    fn search_handles_unicode_case_folding_and_character_boundaries() {
        let lines = vec![String::from("Élan İST")];
        let mut state = ViewerState::default();

        state.apply(Action::EnterSearch, bounds(), &lines);
        for character in "élan".chars() {
            state.apply(Action::SearchCharacter(character), bounds(), &lines);
        }
        state.apply(Action::SubmitSearch, bounds(), &lines);

        assert_eq!(state.search.matches.len(), 1);
        assert_eq!(state.search.matches[0].start, 0);
        assert_eq!(state.search.matches[0].end, "Élan".len());
    }

    #[test]
    fn empty_search_clears_matches_and_escape_keeps_previous_query() {
        let lines = vec![String::from("needle")];
        let mut state = ViewerState::default();

        state.apply(Action::EnterSearch, bounds(), &lines);
        for character in "needle".chars() {
            state.apply(Action::SearchCharacter(character), bounds(), &lines);
        }
        state.apply(Action::SubmitSearch, bounds(), &lines);
        assert_eq!(state.search.matches.len(), 1);

        state.apply(Action::EnterSearch, bounds(), &lines);
        state.apply(Action::SearchCharacter('x'), bounds(), &lines);
        state.apply(Action::CancelSearch, bounds(), &lines);
        assert_eq!(state.search.query, "needle");

        state.apply(Action::ClearSearch, bounds(), &lines);
        assert!(state.search.query.is_empty());
        assert!(state.search.matches.is_empty());

        state.apply(Action::EnterSearch, bounds(), &lines);
        state.apply(Action::SubmitSearch, bounds(), &lines);
        assert!(state.search.query.is_empty());
        assert!(state.search.matches.is_empty());
    }
}
