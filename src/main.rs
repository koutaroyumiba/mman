use std::io;
use std::io::IsTerminal;
use std::io::Write;
use std::process::ExitCode;
use std::{env, error::Error, path::PathBuf};

use clap::Parser;
use mman::{
    cli::{Cli, ExecutionMode, TerminalState, select_mode},
    pages::PageIndex,
    paths::{search_paths, validate_roots},
    tui::{run_picker, run_viewer},
};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("mman: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let mode = select_mode(
        &cli,
        TerminalState {
            stdin: io::stdin().is_terminal(),
            stdout: io::stdout().is_terminal(),
        },
    )?;

    let env_path = env::var_os("MMANPATH");
    let home = env::var_os("HOME").map(PathBuf::from);

    let path_bufs = search_paths(cli.paths.as_deref(), env_path.as_deref(), home.as_deref());
    let validated_paths = validate_roots(&path_bufs)?;

    for warning in &validated_paths.warnings {
        eprintln!("mman: [WARN] {warning}");
    }

    let page_index = PageIndex::discover(&validated_paths.roots)?;

    match mode {
        ExecutionMode::Where { topic } => {
            let page = page_index.lookup(&topic)?;
            println!("{}", page.path.display());
        }
        ExecutionMode::Raw { topic } => {
            let page = page_index.lookup(&topic)?;
            let raw_content = page.read_raw()?;
            let mut stdout = io::stdout().lock();
            stdout.write_all(&raw_content)?;
        }
        ExecutionMode::Viewer { topic } => {
            let page = page_index.lookup(&topic)?;
            let source = String::from_utf8(page.read_raw()?)?;
            run_viewer(&topic, &source)?;
        }
        ExecutionMode::Picker => {
            let topics: Vec<String> = page_index.topics().map(str::to_owned).collect();
            if let Some(topic) = run_picker(&topics, &validated_paths.roots)? {
                let page = page_index.lookup(&topic)?;
                let source = String::from_utf8(page.read_raw()?)?;
                run_viewer(&topic, &source)?;
            }
        }
    }

    Ok(())
}
