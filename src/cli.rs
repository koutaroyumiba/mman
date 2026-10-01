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

    /// Print the selected page's sourth path
    #[arg(long = "where", requires = "topic", conflicts_with = "raw")]
    pub where_path: bool,

    /// Topic to open.
    #[arg(value_name = "TOPIC")]
    pub topic: Option<String>,
}
