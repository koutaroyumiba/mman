use std::{env, fs, path::Path, process::Command};

use tempfile::tempdir;

fn mman_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mman"));
    command.env_remove("MMANPATH");
    command
}

fn write_page(root: &Path, topic: &str, source: &str) {
    let path = root.join(format!("{topic}.md"));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("page directory should be created");
    }
    fs::write(path, source).expect("page should be written");
}

#[test]
fn search_matches_topics_and_content_with_stable_limited_previews() {
    let first_root = tempdir().expect("first root should be created");
    let second_root = tempdir().expect("second root should be created");
    write_page(
        first_root.path(),
        "beta",
        "zero\nNeedle one\nskip\nneedle two\nNEEDLE three\nneedle four\n",
    );
    write_page(
        first_root.path(),
        "nested/needle-topic",
        "content without the term\n",
    );
    write_page(
        second_root.path(),
        "beta",
        "needle from lower-precedence page\n",
    );
    write_page(second_root.path(), "unrelated", "nothing here\n");

    let paths = env::join_paths([first_root.path(), second_root.path()])
        .expect("test roots should form a path list");
    let output = mman_command()
        .arg("-M")
        .arg(paths)
        .args(["-k", "nEeDlE"])
        .output()
        .expect("mman should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        concat!(
            "beta\n",
            "  2: Needle one\n",
            "  4: needle two\n",
            "  5: NEEDLE three\n",
            "nested/needle-topic\n",
        )
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn long_search_flag_supports_multi_word_terms() {
    let root = tempdir().expect("root should be created");
    write_page(root.path(), "git", "The working tree contains changes.\n");

    let output = mman_command()
        .arg("-M")
        .arg(root.path())
        .args(["--search", "WORKING TREE"])
        .output()
        .expect("mman should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        "git\n  1: The working tree contains changes.\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn search_with_no_matches_has_empty_successful_output() {
    let root = tempdir().expect("root should be created");
    write_page(root.path(), "git", "working tree\n");

    let output = mman_command()
        .arg("-M")
        .arg(root.path())
        .args(["-k", "missing"])
        .output()
        .expect("mman should run");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn search_replaces_control_characters_in_previews() {
    let root = tempdir().expect("root should be created");
    write_page(root.path(), "controls", "needle\u{1b}[31m\n");

    let output = mman_command()
        .arg("-M")
        .arg(root.path())
        .args(["-k", "needle"])
        .output()
        .expect("mman should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        "controls\n  1: needle�[31m\n"
    );
}
