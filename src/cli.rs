use std::{error::Error, ffi::OsString, fmt};

use clap::Parser;

/// Browse custom Markdown man pages
#[derive(Debug, Parser)]
#[command(name = "mman", version, about)]
pub struct Cli {
    /// Override MMANPATH for this invocation
    #[arg(short = 'M', value_name = "PATHS")]
    pub paths: Option<OsString>,

    /// Print the original Markdown
    #[arg(long, requires = "topic", conflicts_with = "where_path")]
    pub raw: bool,

    /// Print the selected page's source path
    #[arg(long = "where", requires = "topic", conflicts_with = "raw")]
    pub where_path: bool,

    /// Select among duplicate page sources
    #[arg(
        short = 's',
        long = "select",
        requires = "topic",
        conflicts_with_all = ["raw", "where_path", "list", "search"]
    )]
    pub select: bool,

    /// List available topics
    #[arg(
        short = 'l',
        long = "list",
        conflicts_with_all = ["raw", "where_path", "select", "search", "topic"]
    )]
    pub list: bool,

    /// Search topic names and page content
    #[arg(
        short = 'k',
        long = "search",
        value_name = "TERM",
        conflicts_with_all = ["raw", "where_path", "select", "list", "topic"],
        value_parser = clap::builder::NonEmptyStringValueParser::new()
    )]
    pub search: Option<String>,

    /// Topic to open.
    #[arg(value_name = "TOPIC")]
    pub topic: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalState {
    pub stdin: bool,
    pub stdout: bool,
}

impl TerminalState {
    fn is_interactive(self) -> bool {
        self.stdin && self.stdout
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionMode {
    Picker,
    Viewer { topic: String },
    Raw { topic: String },
    Where { topic: String },
    List,
    Search { term: String },
    Select { topic: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeSelectionError {
    TopicRequiredNonInteractive,
    SelectionRequiresInteractive,
}

impl fmt::Display for ModeSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TopicRequiredNonInteractive => {
                formatter.write_str("topic is required in a non-interactive session")
            }
            Self::SelectionRequiresInteractive => {
                formatter.write_str("source selection requires an interactive terminal")
            }
        }
    }
}

impl Error for ModeSelectionError {}

pub fn select_mode(
    cli: &Cli,
    terminals: TerminalState,
) -> Result<ExecutionMode, ModeSelectionError> {
    if cli.list {
        return Ok(ExecutionMode::List);
    }

    if let Some(term) = cli.search.as_ref() {
        return Ok(ExecutionMode::Search { term: term.clone() });
    }

    let Some(topic) = cli.topic.as_ref() else {
        return if terminals.is_interactive() && !cli.raw && !cli.where_path {
            Ok(ExecutionMode::Picker)
        } else {
            Err(ModeSelectionError::TopicRequiredNonInteractive)
        };
    };

    if cli.select {
        return if terminals.is_interactive() {
            Ok(ExecutionMode::Select {
                topic: topic.clone(),
            })
        } else {
            Err(ModeSelectionError::SelectionRequiresInteractive)
        };
    }

    if cli.where_path {
        return Ok(ExecutionMode::Where {
            topic: topic.clone(),
        });
    }

    if cli.raw || !terminals.is_interactive() {
        return Ok(ExecutionMode::Raw {
            topic: topic.clone(),
        });
    }

    Ok(ExecutionMode::Viewer {
        topic: topic.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::{Cli, ExecutionMode, ModeSelectionError, TerminalState, select_mode};
    use clap::Parser;

    #[test]
    fn parses_plain_topic() {
        let cli =
            Cli::try_parse_from(["mman", "concepts/ownership"]).expect("arguments should parse");

        assert_eq!(cli.topic.as_deref(), Some("concepts/ownership"));
        assert!(!cli.raw);
        assert!(!cli.where_path);
        assert!(!cli.list);
        assert!(!cli.select);
        assert!(cli.search.is_none());
    }

    #[test]
    fn parses_raw_topic() {
        let cli = Cli::try_parse_from(["mman", "--raw", "concepts/ownership"])
            .expect("arguments should parse");

        assert!(cli.raw);
        assert_eq!(cli.topic.as_deref(), Some("concepts/ownership"));
    }

    #[test]
    fn rejects_raw_without_topic() {
        let result = Cli::try_parse_from(["mman", "--raw"]);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_conflicting_output_modes() {
        let result = Cli::try_parse_from(["mman", "--raw", "--where", "concepts/ownership"]);

        assert!(result.is_err());
    }

    #[test]
    fn parses_list_without_a_topic() {
        let cli = Cli::try_parse_from(["mman", "-l"]).expect("arguments should parse");

        assert!(cli.list);
        assert!(cli.topic.is_none());
        assert_eq!(
            select_mode(
                &cli,
                TerminalState {
                    stdin: false,
                    stdout: false,
                }
            ),
            Ok(ExecutionMode::List)
        );
    }

    #[test]
    fn rejects_list_with_topic_or_another_output_mode() {
        for arguments in [
            vec!["mman", "-l", "ownership"],
            vec!["mman", "-l", "--raw"],
            vec!["mman", "-l", "--where"],
        ] {
            assert!(Cli::try_parse_from(arguments).is_err());
        }
    }

    #[test]
    fn parses_collection_search_without_a_topic() {
        let cli =
            Cli::try_parse_from(["mman", "-k", "working tree"]).expect("arguments should parse");

        assert_eq!(cli.search.as_deref(), Some("working tree"));
        assert_eq!(
            select_mode(
                &cli,
                TerminalState {
                    stdin: false,
                    stdout: false,
                }
            ),
            Ok(ExecutionMode::Search {
                term: String::from("working tree")
            })
        );
    }

    #[test]
    fn rejects_empty_or_conflicting_collection_search() {
        for arguments in [
            vec!["mman", "-k", ""],
            vec!["mman", "-k", "term", "ownership"],
            vec!["mman", "-k", "term", "--raw"],
            vec!["mman", "-k", "term", "--where"],
            vec!["mman", "-k", "term", "-l"],
        ] {
            assert!(Cli::try_parse_from(arguments).is_err());
        }
    }

    #[test]
    fn source_selection_requires_a_topic_and_interactive_terminal() {
        assert!(Cli::try_parse_from(["mman", "--select"]).is_err());

        let cli = Cli::try_parse_from(["mman", "-s", "ownership"]).expect("arguments should parse");
        assert_eq!(
            select_mode(
                &cli,
                TerminalState {
                    stdin: true,
                    stdout: true,
                }
            ),
            Ok(ExecutionMode::Select {
                topic: String::from("ownership")
            })
        );
        assert_eq!(
            select_mode(
                &cli,
                TerminalState {
                    stdin: false,
                    stdout: false,
                }
            ),
            Err(ModeSelectionError::SelectionRequiresInteractive)
        );
    }

    #[test]
    fn source_selection_rejects_other_modes() {
        for arguments in [
            vec!["mman", "-s", "ownership", "--raw"],
            vec!["mman", "-s", "ownership", "--where"],
            vec!["mman", "-s", "ownership", "-l"],
            vec!["mman", "-s", "ownership", "-k", "term"],
        ] {
            assert!(Cli::try_parse_from(arguments).is_err());
        }
    }

    #[test]
    fn interactive_no_topic_selects_picker() {
        let cli = Cli::try_parse_from(["mman"]).expect("arguments should parse");
        let terminals = TerminalState {
            stdin: true,
            stdout: true,
        };

        assert_eq!(select_mode(&cli, terminals), Ok(ExecutionMode::Picker));
    }

    #[test]
    fn noninteractive_no_topic_is_a_usage_error() {
        let cli = Cli::try_parse_from(["mman"]).expect("arguments should parse");

        for terminals in [
            TerminalState {
                stdin: false,
                stdout: true,
            },
            TerminalState {
                stdin: true,
                stdout: false,
            },
        ] {
            assert_eq!(
                select_mode(&cli, terminals),
                Err(ModeSelectionError::TopicRequiredNonInteractive)
            );
        }
    }

    #[test]
    fn plain_topic_selects_viewer_only_when_fully_interactive() {
        let cli = Cli::try_parse_from(["mman", "ownership"]).expect("arguments should parse");

        assert_eq!(
            select_mode(
                &cli,
                TerminalState {
                    stdin: true,
                    stdout: true,
                }
            ),
            Ok(ExecutionMode::Viewer {
                topic: String::from("ownership"),
            })
        );

        for terminals in [
            TerminalState {
                stdin: false,
                stdout: true,
            },
            TerminalState {
                stdin: true,
                stdout: false,
            },
        ] {
            assert_eq!(
                select_mode(&cli, terminals),
                Ok(ExecutionMode::Raw {
                    topic: String::from("ownership"),
                })
            );
        }
    }

    #[test]
    fn explicit_raw_overrides_interactive_terminal_state() {
        let cli =
            Cli::try_parse_from(["mman", "--raw", "ownership"]).expect("arguments should parse");

        assert_eq!(
            select_mode(
                &cli,
                TerminalState {
                    stdin: true,
                    stdout: true,
                }
            ),
            Ok(ExecutionMode::Raw {
                topic: String::from("ownership"),
            })
        );
    }

    #[test]
    fn where_mode_does_not_depend_on_terminal_state() {
        let cli =
            Cli::try_parse_from(["mman", "--where", "ownership"]).expect("arguments should parse");

        assert_eq!(
            select_mode(
                &cli,
                TerminalState {
                    stdin: false,
                    stdout: false,
                }
            ),
            Ok(ExecutionMode::Where {
                topic: String::from("ownership"),
            })
        );
    }
}
