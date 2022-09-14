use std::fs;
use std::io::Read;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::str::{contains, ends_with, is_empty, starts_with};
use tempfile::{tempdir, NamedTempFile};

fn cmd() -> Command {
    Command::cargo_bin("dbz").unwrap()
}

const DBZ_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");

#[test]
fn write_json_to_path() {
    // create a directory whose contents will be cleaned up at the end of the test
    let output_dir = tempdir().unwrap();
    let output_path = format!("{}/a.json", output_dir.path().to_string_lossy());
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbp-1.dbz"),
            "--output",
            &output_path,
            "--json",
        ])
        .assert()
        .success()
        .stdout(is_empty());
    let contents = fs::read_to_string(output_path).unwrap();
    assert!(contents.contains(","));
    assert!(contents.contains("["));
    assert!(contents.contains("]"));
    assert!(contents.contains("{"));
    assert!(contents.contains("{"));
    assert!(contents.ends_with("\n"));
}

#[test]
fn write_to_stdout() {
    cmd()
        .args(&[&format!("{DBZ_PATH}/test_data.mbo.dbz"), "--csv"])
        .assert()
        .success()
        .stdout(contains("channel_id"));
}

#[test]
fn write_to_nonexistent_path() {
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbo.dbz"),
            "--output",
            "./a/b/c/d/e",
            "-C", // CSV
        ])
        .assert()
        .failure()
        .stderr(contains("Unable to open output file './a/b/c/d/e'"));
}

#[test]
fn read_from_nonexistent_path() {
    let input_file = NamedTempFile::new().unwrap();
    let input_path = input_file.path().to_owned();
    // delete input_file and ensure it was cleaned up
    input_file.close().unwrap();
    let output_file = NamedTempFile::new().unwrap();
    cmd()
        .args(&[
            &input_path.to_string_lossy(),
            "--output",
            &output_file.path().to_string_lossy(),
        ])
        .assert()
        .failure()
        .stderr(contains("Error opening dbz file"));
}

#[test]
fn write_csv() {
    // create a directory whose contents will be cleaned up at the end of the test
    let output_dir = tempdir().unwrap();
    let output_path = format!("{}/a.json", output_dir.path().to_string_lossy());
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbp-1.dbz"),
            "--output",
            &output_path,
            "--csv",
        ])
        .assert()
        .success()
        .stdout(is_empty());
    let contents = fs::read_to_string(output_path).unwrap();
    assert!(contents.contains(","));
    assert!(!contents.contains("["));
    assert!(!contents.contains("]"));
    assert!(!contents.contains("{"));
    assert!(!contents.contains("{"));
    assert!(contents.ends_with("\n"));
}

#[test]
fn encoding_overrides_extension() {
    // create a directory whose contents will be cleaned up at the end of the test
    let output_dir = tempdir().unwrap();
    // output file extension is csv, but the encoding argument is json
    let output_path = format!("{}/a.csv", output_dir.path().to_string_lossy());
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbp-1.dbz"),
            "--output",
            &output_path,
            "-J", // JSON
        ])
        .assert()
        .success()
        .stdout(is_empty());
    let contents = fs::read_to_string(output_path).unwrap();
    assert!(contents.contains(","));
    assert!(contents.contains("["));
    assert!(contents.contains("]"));
    assert!(contents.contains("{"));
    assert!(contents.contains("{"));
    assert!(contents.ends_with("\n"));
}

#[test]
fn bad_infer() {
    let output_dir = tempdir().unwrap();
    let output_path = format!("{}/a.yaml", output_dir.path().to_string_lossy());
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbo.dbz"),
            "--output",
            &output_path,
        ])
        .assert()
        .failure()
        .stderr(contains(
            "Unable to infer output encoding from output file with extension 'yaml'",
        ));
}

#[test]
fn no_extension_infer() {
    let output_dir = tempdir().unwrap();
    let output_path = format!("{}/a", output_dir.path().to_string_lossy());
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbo.dbz"),
            "--output",
            &output_path,
        ])
        .assert()
        .failure()
        .stderr(contains(
            "Unable to infer output encoding from output file without an extension",
        ));
}

#[test]
fn overwrite_fails() {
    let output_file = NamedTempFile::new().unwrap();

    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbo.dbz"),
            "--output",
            &output_file.path().to_string_lossy(),
            "--csv",
        ])
        .assert()
        .failure()
        .stderr(contains("Output file exists"));
}

#[test]
fn force_overwrite() {
    let output_file = NamedTempFile::new().unwrap();
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbo.dbz"),
            "--output",
            &output_file.path().to_string_lossy(),
            "-C", // CSV
            "--force",
        ])
        .assert()
        .success()
        .stdout(is_empty());
    let mut contents = String::new();
    output_file.as_file().read_to_string(&mut contents).unwrap();
}

#[test]
fn cant_specify_json_and_csv() {
    cmd()
        .args(&[&format!("{DBZ_PATH}/test_data.mbo.dbz"), "--json", "--csv"])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
}

#[test]
fn metadata() {
    cmd()
        .args(&[&format!("{DBZ_PATH}/test_data.ohlcv-1m.dbz"), "-J", "-m"])
        .assert()
        .success()
        .stdout(starts_with("{"))
        .stdout(ends_with("}\n"))
        .stderr(is_empty());
}

#[test]
fn no_csv_metadata() {
    cmd()
        .args(&[&format!("{DBZ_PATH}/test_data.ohlcv-1m.dbz"), "--csv", "-m"])
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("unsupported"));
}

#[test]
fn pretty_print_data() {
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbo.dbz"),
            "--json",
            "--pretty-json",
        ])
        .assert()
        .success()
        .stdout(contains("    "))
        .stdout(contains(",\n"))
        .stderr(is_empty());
}

#[test]
fn pretty_print_data_metadata() {
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbo.dbz"),
            "-J",
            "--metadata",
            "-p",
        ])
        .assert()
        .stderr(is_empty());
}

#[test]
fn help() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("USAGE:"));
}

#[test]
fn version() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}
