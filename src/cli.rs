use std::ffi::OsString;

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

    /// Topic to open.
    #[arg(value_name = "TOPIC")]
    pub topic: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;

    #[test]
    fn parses_plain_topic() {
        let cli =
            Cli::try_parse_from(["mman", "concepts/ownership"]).expect("arguments should parse");

        assert_eq!(cli.topic.as_deref(), Some("concepts/ownership"));
        assert!(!cli.raw);
        assert!(!cli.where_path);
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
}
