use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopicError {
    Empty,
    Absolute,
    EmptyComponent,
    CurrentDirectory,
    ParentDirectory,
    HiddenComponent,
    ControlCharacter,
}

impl fmt::Display for TopicError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "topic cannot be empty",
            Self::Absolute => "topic cannot be an absolute path",
            Self::EmptyComponent => "topic cannot contain empty path components",
            Self::CurrentDirectory => "topic cannot contain a current-directory component",
            Self::ParentDirectory => "topic cannot contain a parent-directory component",
            Self::HiddenComponent => "topic cannot contain hidden path components",
            Self::ControlCharacter => "topic cannot contain control characters",
        };

        formatter.write_str(message)
    }
}

impl Error for TopicError {}

pub fn normalize_topic(topic: &str) -> Result<String, TopicError> {
    if topic.is_empty() {
        return Err(TopicError::Empty);
    }

    if topic.starts_with('/') {
        return Err(TopicError::Absolute);
    }

    let topic = topic.strip_suffix(".md").unwrap_or(topic);

    if topic.is_empty() {
        return Err(TopicError::Empty);
    }

    for component in topic.split('/') {
        if component.is_empty() {
            return Err(TopicError::EmptyComponent);
        }

        if component == "." {
            return Err(TopicError::CurrentDirectory);
        }

        if component == ".." {
            return Err(TopicError::ParentDirectory);
        }

        if component.starts_with('.') {
            return Err(TopicError::HiddenComponent);
        }

        if component.chars().any(char::is_control) {
            return Err(TopicError::ControlCharacter);
        }
    }

    Ok(topic.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{TopicError, normalize_topic};

    #[test]
    fn accepts_plain_topic() {
        assert_eq!(normalize_topic("ownership"), Ok(String::from("ownership")));
    }

    #[test]
    fn accepts_nested_topic() {
        assert_eq!(
            normalize_topic("concepts/ownership"),
            Ok(String::from("concepts/ownership"))
        );
    }

    #[test]
    fn removes_markdown_extension() {
        assert_eq!(
            normalize_topic("concepts/ownership.md"),
            Ok(String::from("concepts/ownership"))
        );
    }

    #[test]
    fn preserves_topic_case() {
        assert_eq!(
            normalize_topic("Concepts/Ownership"),
            Ok(String::from("Concepts/Ownership"))
        );
    }

    #[test]
    fn rejects_absolute_topic() {
        assert_eq!(
            normalize_topic("/concepts/ownership"),
            Err(TopicError::Absolute)
        );
    }

    #[test]
    fn rejects_current_directory_component() {
        assert_eq!(
            normalize_topic("./ownership"),
            Err(TopicError::CurrentDirectory)
        );
    }

    #[test]
    fn rejects_parent_directory_component() {
        assert_eq!(
            normalize_topic("concepts/../ownership"),
            Err(TopicError::ParentDirectory)
        );
    }

    #[test]
    fn rejects_hidden_component() {
        assert_eq!(
            normalize_topic("concepts/.private/ownership"),
            Err(TopicError::HiddenComponent)
        );
    }

    #[test]
    fn rejects_repeated_separator() {
        assert_eq!(
            normalize_topic("concepts//ownership"),
            Err(TopicError::EmptyComponent)
        );
    }

    #[test]
    fn rejects_control_characters() {
        assert_eq!(
            normalize_topic("concepts/\u{1b}ownership"),
            Err(TopicError::ControlCharacter)
        );
    }
}
