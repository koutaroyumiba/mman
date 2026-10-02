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
fn where_prints_selected_page_path() {
    let root = tempdir().expect("temporary root should be created");
    write_page(root.path(), "concepts/ownership");

    let output = mman_command()
        .arg("-M")
        .arg(root.path())
        .args(["--where", "concepts/ownership"])
        .output()
        .expect("mman should run");

    let expected_path = root
        .path()
        .canonicalize()
        .expect("root should canonicalize")
        .join("concepts/ownership.md");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        format!("{}\n", expected_path.display())
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn where_reports_missing_topic_on_stderr() {
    let root = tempdir().expect("temporary root should be created");

    let output = mman_command()
        .arg("-M")
        .arg(root.path())
        .args(["--where", "missing"])
        .output()
        .expect("mman should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("no manual page found for missing"));
}

#[test]
fn where_warns_about_invalid_root_and_uses_valid_root() {
    let directory = tempdir().expect("temporary directory should be created");
    let valid_root = directory.path().join("valid");
    let missing_root = directory.path().join("missing");

    fs::create_dir(&valid_root).expect("valid root should be created");
    write_page(&valid_root, "git");

    let paths = env::join_paths([missing_root.as_path(), valid_root.as_path()])
        .expect("test paths should form a path list");
    let output = mman_command()
        .arg("-M")
        .arg(paths)
        .args(["--where", "git"])
        .output()
        .expect("mman should run");

    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .expect("stderr should be UTF-8")
            .contains(&missing_root.display().to_string())
    );
}

#[test]
fn command_line_paths_override_mmanpath() {
    let command_line_root = tempdir().expect("command-line root should be created");
    let environment_root = tempdir().expect("environment root should be created");
    write_page(command_line_root.path(), "git");
    write_page(environment_root.path(), "git");

    let output = mman_command()
        .env("MMANPATH", environment_root.path())
        .arg("-M")
        .arg(command_line_root.path())
        .args(["--where", "git"])
        .output()
        .expect("mman should run");

    let expected_path = command_line_root
        .path()
        .canonicalize()
        .expect("root should canonicalize")
        .join("git.md");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        format!("{}\n", expected_path.display())
    );
}
