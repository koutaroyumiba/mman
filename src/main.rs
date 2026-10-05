use std::io;
use std::io::Write;
use std::process::ExitCode;
use std::{env, error::Error, path::PathBuf};

use clap::Parser;
use mman::{
    cli::Cli,
    pages::PageIndex,
    paths::{search_paths, validate_roots},
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
    let env_path = env::var_os("MMANPATH");
    let home = env::var_os("HOME").map(PathBuf::from);

    let path_bufs = search_paths(cli.paths.as_deref(), env_path.as_deref(), home.as_deref());
    let validated_paths = validate_roots(&path_bufs)?;

    for warning in &validated_paths.warnings {
        eprintln!("mman: [WARN] {warning}");
    }

    let page_index = PageIndex::discover(&validated_paths.roots)?;
    if cli.where_path || cli.raw {
        let Some(topic) = cli.topic.as_deref() else {
            if cli.where_path {
                return Err("topic required for --where".into());
            }
            return Err("topic required for --raw".into());
        };
        let page = page_index.lookup(topic)?;

        if cli.where_path {
            println!("{}", page.path.display());
        } else {
            let raw_content = page.read_raw()?;
            let mut stdout = io::stdout().lock();
            stdout.write_all(&raw_content)?;
        }
    }

    Ok(())
}
