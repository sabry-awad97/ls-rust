use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

const PROG_NAME: &'static str = "lsr";

#[test]
fn test_ls() {
    let mut cmd = Command::cargo_bin(PROG_NAME).unwrap();

    // Set the command-line arguments and options
    cmd.arg(".");

    // Run the command and check the output
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Cargo.toml"));
}
#[test]

fn test_all() {
    let mut cmd = Command::cargo_bin(PROG_NAME).unwrap();

    // Set the command-line arguments and options
    cmd.arg("-a").arg(".");

    // Run the command and check the output
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(".git"));
}
#[test]

fn test_almost_all() {
    let mut cmd = Command::cargo_bin(PROG_NAME).unwrap();

    // Set the command-line arguments and options
    cmd.arg("-A").arg(".");

    // Run the command and check the output
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(".gitignore"));
}
