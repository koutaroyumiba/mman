use std::{fs, process::Command};

use tempfile::tempdir;

fn mman_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mman"));
    command.env_remove("MMANPATH");
    command
}

#[test]
fn duplicate_selection_requires_an_interactive_terminal() {
    let root = tempdir().expect("root should be created");
    fs::write(root.path().join("ownership.md"), "# Ownership\n").expect("page should be written");

    for flag in ["-s", "--select"] {
        let output = mman_command()
            .arg("-M")
            .arg(root.path())
            .args([flag, "ownership"])
            .output()
            .expect("mman should run");

        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .expect("stderr should be UTF-8")
                .contains("source selection requires an interactive terminal")
        );
    }
}

#[test]
fn duplicate_selection_requires_a_topic() {
    let output = mman_command()
        .arg("--select")
        .output()
        .expect("mman should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .expect("stderr should be UTF-8")
            .contains("required arguments")
    );
}
