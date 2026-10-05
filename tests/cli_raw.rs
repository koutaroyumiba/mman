use std::{fs, path::Path, process::Command};

use tempfile::tempdir;

fn mman_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mman"));
    command.env_remove("MMANPATH");
    command
}

fn write_page(root: &Path, topic: &str, source: &[u8]) {
    let path = root.join(format!("{topic}.md"));

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("page directory should be created");
    }

    fs::write(path, source).expect("page should be written");
}

#[test]
fn raw_writes_original_markdown_bytes() {
    let root = tempdir().expect("temporary root should be created");
    let source = b"# Ownership\r\n\r\nBorrowing notes.\r\n";
    write_page(root.path(), "concepts/ownership", source);

    let output = mman_command()
        .arg("-M")
        .arg(root.path())
        .args(["--raw", "concepts/ownership"])
        .output()
        .expect("mman should run");

    assert!(output.status.success());
    assert_eq!(output.stdout, source);
    assert!(output.stderr.is_empty());
}

#[test]
fn redirected_plain_topic_writes_original_markdown_bytes() {
    let root = tempdir().expect("temporary root should be created");
    let source = b"# Git\n\nWorking tree notes.\n";
    write_page(root.path(), "git", source);

    let output = mman_command()
        .arg("-M")
        .arg(root.path())
        .arg("git")
        .output()
        .expect("mman should run");

    assert!(output.status.success());
    assert_eq!(output.stdout, source);
    assert!(output.stderr.is_empty());
}

#[test]
fn redirected_plain_topic_reports_missing_topic() {
    let root = tempdir().expect("temporary root should be created");

    let output = mman_command()
        .arg("-M")
        .arg(root.path())
        .arg("missing")
        .output()
        .expect("mman should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("no manual page found for missing"));
}

#[test]
fn noninteractive_invocation_without_topic_reports_usage_error() {
    let output = mman_command().output().expect("mman should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("topic is required in a non-interactive session"));
    assert!(!stderr.contains("no documentation paths configured"));
}

#[test]
fn raw_reports_missing_topic_on_stderr() {
    let root = tempdir().expect("temporary root should be created");

    let output = mman_command()
        .arg("-M")
        .arg(root.path())
        .args(["--raw", "missing"])
        .output()
        .expect("mman should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("no manual page found for missing"));
    assert!(!stderr.contains("Did you mean?"));
}

#[test]
fn missing_topic_reports_similar_topic_suggestions() {
    let root = tempdir().expect("temporary root should be created");
    write_page(root.path(), "concepts/ownership", b"# Ownership\n");

    let output = mman_command()
        .arg("-M")
        .arg(root.path())
        .args(["--raw", "owership"])
        .output()
        .expect("mman should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("no manual page found for owership"));
    assert!(stderr.contains("Did you mean?"));
    assert!(stderr.contains("  concepts/ownership"));
}
