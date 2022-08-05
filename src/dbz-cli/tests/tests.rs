use std::fs;
use std::io::Read;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::str::{contains, is_empty};
use tempfile::{tempdir, NamedTempFile};

fn cmd() -> Command {
    Command::cargo_bin("dbz").unwrap()
}

const DBZ_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../public/databento-python/tests/data"
);

#[test]
#[ignore = "Need to update DBZ file"]
fn write_json_to_path() {
    // create a directory whose contents will be cleaned up at the end of the test
    let output_dir = tempdir().unwrap();
    let output_path = format!("{}/a.json", output_dir.path().to_string_lossy());
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbp-1.dbz"),
            "--output",
            &output_path,
            "--encoding",
            "json",
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
#[ignore = "Need to update DBZ file"]
fn write_to_stdout() {
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbo.dbz"),
            "--stdout",
            "--encoding",
            "csv",
        ])
        .assert()
        .success()
        .stdout(contains("chan_id"));
}

#[test]
#[ignore = "Need to update DBZ file"]
fn write_to_nonexistent_path() {
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbo.dbz"),
            "--output",
            "./a/b/c/d/e",
            "--encoding",
            "csv",
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
#[ignore = "Need to update DBZ file"]
fn write_csv() {
    // create a directory whose contents will be cleaned up at the end of the test
    let output_dir = tempdir().unwrap();
    let output_path = format!("{}/a.json", output_dir.path().to_string_lossy());
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbp-1.dbz"),
            "--output",
            &output_path,
            "--encoding",
            "csv",
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
#[ignore = "Need to update DBZ file"]
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
            "--encoding",
            "json",
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
#[ignore = "Need to update DBZ file"]
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
#[ignore = "Need to update DBZ file"]
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
#[ignore = "Need to update DBZ file"]
fn overwrite_fails() {
    let output_file = NamedTempFile::new().unwrap();

    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbo.dbz"),
            "--output",
            &output_file.path().to_string_lossy(),
            "--encoding",
            "csv",
        ])
        .assert()
        .failure()
        .stderr(contains("Output file exists"));
}

#[test]
#[ignore = "Need to update DBZ file"]
fn force_overwrite() {
    let output_file = NamedTempFile::new().unwrap();
    cmd()
        .args(&[
            &format!("{DBZ_PATH}/test_data.mbo.dbz"),
            "--output",
            &output_file.path().to_string_lossy(),
            "--encoding",
            "csv",
            "--force",
        ])
        .assert()
        .success()
        .stdout(is_empty());
    let mut contents = String::new();
    output_file.as_file().read_to_string(&mut contents).unwrap();
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
