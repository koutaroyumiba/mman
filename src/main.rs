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
    if cli.where_path {
        let Some(topic) = cli.topic else {
            return Err("topic required for --where".into());
        };
        let page = page_index.lookup(topic.as_str())?;
        println!("{}", page.path.display());
    }

    Ok(())
}
