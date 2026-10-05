use std::{env, fs, path::Path, process::Command};

use tempfile::tempdir;

fn mman_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mman"));
    command.env_remove("MMANPATH");
    command
}

fn write_page(root: &Path, topic: &str) {
    let path = root.join(format!("{topic}.md"));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("page directory should be created");
    }
    fs::write(path, "# Test page\n").expect("page should be written");
}

#[test]
fn list_prints_unique_topics_in_alphabetical_order() {
    let first_root = tempdir().expect("first root should be created");
    let second_root = tempdir().expect("second root should be created");
    write_page(first_root.path(), "zsh");
    write_page(first_root.path(), "concepts/ownership");
    write_page(first_root.path(), "cargo");
    write_page(second_root.path(), "cargo");
    write_page(second_root.path(), "algorithms/binary-search");

    let paths = env::join_paths([first_root.path(), second_root.path()])
        .expect("test roots should form a path list");
    let output = mman_command()
        .arg("-M")
        .arg(paths)
        .arg("-l")
        .output()
        .expect("mman should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        "algorithms/binary-search\ncargo\nconcepts/ownership\nzsh\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn long_list_flag_supports_an_empty_collection() {
    let root = tempdir().expect("root should be created");
    let output = mman_command()
        .arg("-M")
        .arg(root.path())
        .arg("--list")
        .output()
        .expect("mman should run");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn list_rejects_a_topic() {
    let root = tempdir().expect("root should be created");
    let output = mman_command()
        .arg("-M")
        .arg(root.path())
        .args(["-l", "ownership"])
        .output()
        .expect("mman should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .expect("stderr should be UTF-8")
            .contains("cannot be used with")
    );
}
