use std::{
    fs,
    io::{Read, Write},
    process,
};

use assert_cmd::{cargo::cargo_bin_cmd, Command};
use dbn::Schema;
use predicates::{
    boolean::PredicateBooleanExt,
    ord::eq,
    str::{contains, ends_with, is_empty, is_match, starts_with},
};
use rstest::*;
use tempfile::{tempdir, NamedTempFile, TempDir};

fn cmd() -> Command {
    cargo_bin_cmd!("dbn")
}

const TEST_DATA_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");

#[fixture]
fn output_dir() -> TempDir {
    tempdir().unwrap()
}

#[rstest]
fn write_json_to_path(output_dir: TempDir) {
    let output_path = format!("{}/a.json", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbp-1.v3.dbn.zst"),
            "--output",
            &output_path,
            "--json",
        ])
        .assert()
        .success()
        .stdout(is_empty());
    let contents = fs::read_to_string(output_path).unwrap();
    assert!(contents.contains(','));
    assert!(contents.contains('['));
    assert!(contents.contains(']'));
    assert!(contents.contains('{'));
    assert!(contents.contains('{'));
    assert!(contents.ends_with('\n'));
    // no map symbols
    assert!(!contents.contains("\"symbol\":"));
}

#[test]
fn write_to_stdout() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--csv",
        ])
        .assert()
        .success()
        .stdout(contains("channel_id"));
}

#[test]
fn write_to_nonexistent_path() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn"),
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
        .args([
            (input_path.to_str().unwrap()),
            "--output",
            (output_file.path().to_str().unwrap()),
        ])
        .assert()
        .failure()
        .stderr(contains("opening file to decode"));
}

#[rstest]
fn write_csv(output_dir: TempDir) {
    let output_path = format!("{}/a.csv", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbp-1.v3.dbn.zst"),
            "--output",
            &output_path,
            "--csv",
        ])
        .assert()
        .success()
        .stdout(is_empty());
    let contents = fs::read_to_string(output_path).unwrap();
    assert!(contents.contains(','));
    assert!(!contents.contains('['));
    assert!(!contents.contains(']'));
    assert!(!contents.contains('{'));
    assert!(!contents.contains('{'));
    assert!(contents.ends_with('\n'));
}

#[rstest]
fn encoding_overrides_extension(output_dir: TempDir) {
    // output file extension is csv, but the encoding argument is json
    let output_path = format!("{}/a.csv", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbp-10.v3.dbn.zst"),
            "--output",
            &output_path,
            "-J", // JSON
        ])
        .assert()
        .success()
        .stdout(is_empty());
    let contents = fs::read_to_string(output_path).unwrap();
    assert!(contents.contains(','));
    assert!(contents.contains('['));
    assert!(contents.contains(']'));
    assert!(contents.contains('{'));
    assert!(contents.contains('{'));
    assert!(contents.ends_with('\n'));
}

#[rstest]
fn bad_infer(output_dir: TempDir) {
    let output_path = format!("{}/a.yaml", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.trades.dbz"),
            "--output",
            &output_path,
        ])
        .assert()
        .failure()
        .stderr(contains("Unable to infer output encoding from output path"));
}

#[rstest]
fn no_extension_infer(output_dir: TempDir) {
    let output_path = format!("{}/a", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--output",
            &output_path,
        ])
        .assert()
        .failure()
        .stderr(contains("Unable to infer output encoding from output path"));
}

#[test]
fn overwrite_fails() {
    let output_file = NamedTempFile::new().unwrap();

    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v2.dbn.zst"),
            "--output",
            (output_file.path().to_str().unwrap()),
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
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--output",
            (output_file.path().to_str().unwrap()),
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
fn force_truncates_file() {
    let mut output_file = NamedTempFile::new().unwrap();
    // Fill file with 10KB of bytes
    for _ in 0..(10 * 1024) {
        output_file.write_all(b"\0").unwrap();
    }
    let before_size = output_file.path().metadata().unwrap().len();
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.ohlcv-1d.v1.dbn.zst"),
            "--output",
            (output_file.path().to_str().unwrap()),
            "-C", // CSV
            "--force",
        ])
        .assert()
        .success()
        .stdout(is_empty());
    let after_size = output_file.path().metadata().unwrap().len();
    assert!(after_size < before_size);
}

#[test]
fn cant_specify_json_and_csv() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--json",
            "--csv",
        ])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
}

#[test]
fn metadata() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.ohlcv-1m.v3.dbn.zst"),
            "-J",
            "-m",
        ])
        .assert()
        .success()
        .stdout(starts_with("{").and(ends_with("}\n")))
        .stderr(is_empty());
}

#[test]
fn no_csv_metadata() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.ohlcv-1m.v3.dbn.zst"),
            "--csv",
            "-m",
        ])
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("cannot be used with"));
}

#[test]
fn pretty_print_json_data() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--json",
            "--pretty",
        ])
        .assert()
        .success()
        .stdout(
            contains("    ")
                .and(contains(",\n"))
                .and(contains(": "))
                .and(is_match(format!(".*\\s\"{PRETTY_TS_REGEX}\".*")).unwrap())
                // prices should also be quoted
                .and(is_match(format!(".*\\s\"{PRETTY_PX_REGEX}\".*")).unwrap()),
        )
        .stderr(is_empty());
}

#[test]
fn pretty_print_csv_data() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--csv",
            "--pretty",
        ])
        .assert()
        .success()
        .stdout(
            is_match(format!(".*{PRETTY_TS_REGEX},.*"))
                .unwrap()
                .and(is_match(format!(".*,{PRETTY_PX_REGEX},.*")).unwrap()),
        )
        .stderr(is_empty());
}

const PRETTY_TS_REGEX: &str = r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{9}Z";
const PRETTY_PX_REGEX: &str = r"\d+\.\d{9}";

#[test]
fn pretty_print_data_metadata() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "-J",
            "--metadata",
            "-p",
        ])
        .assert()
        .stderr(is_empty());
}

#[rstest]
fn read_from_stdin(#[values("csv", "json")] output_enc: &str) {
    let path = format!("{TEST_DATA_PATH}/test_data.mbp-10.v3.dbn.zst");
    let read_from_stdin_output = cmd()
        .args([
            "-", // STDIN
            &format!("--{output_enc}"),
        ])
        // Pipe input from file
        .pipe_stdin(&path)
        .unwrap()
        .ok()
        .unwrap();
    let read_from_file_output = cmd()
        .args([&path, &format!("--{output_enc}")])
        .ok()
        .unwrap();
    assert_eq!(read_from_stdin_output.stdout, read_from_file_output.stdout);
    assert!(read_from_stdin_output.stderr.is_empty());
    assert!(read_from_file_output.stderr.is_empty());
}

#[rstest]
fn convert_dbz_to_dbn(output_dir: TempDir) {
    let output_path = format!("{}/a.dbn", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.definition.dbz"),
            "--dbn",
            "-o",
            &output_path,
        ])
        .assert()
        .success()
        .stderr(is_empty());
}

#[test]
fn limit_and_schema_filter_update_metadata() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.ohlcv-1m.v3.dbn.zst"),
            "--json",
            "--metadata",
            "--limit",
            "1",
            "--schema",
            "ohlcv-1d",
        ])
        .assert()
        .success()
        .stdout(contains(r#""limit":"1""#).and(contains(r#""schema":"ohlcv-1d""#)));
}

#[rstest]
#[case::uncompressed("--input-fragment", "dbn.frag")]
#[case::zstd("--input-zstd-fragment", "dbn.frag.zst")]
fn fragment_conflicts_with_metadata(#[case] flag: &str, #[case] extension: &str) {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.definition.{extension}"),
            flag,
            "--json",
            "--metadata",
        ])
        .assert()
        .failure()
        .stderr(contains(format!(
            "'{flag}' cannot be used with '--metadata'"
        )));
}

#[rstest]
#[case::uncompressed("--input-fragment", "dbn.frag")]
#[case::zstd("--input-zstd-fragment", "dbn.frag.zst")]
fn fragment_conflicts_with_dbn_output(#[case] flag: &str, #[case] extension: &str) {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.definition.{extension}"),
            flag,
            "--dbn",
        ])
        .assert()
        .failure()
        .stderr(contains(format!("'{flag}' cannot be used with '--dbn'")));
}

#[rstest]
#[case::uncompressed_to_csv("csv", "--input-fragment", "dbn.frag", 3)]
#[case::uncompressed_to_json("json", "--input-fragment", "dbn.frag", 2)]
#[case::zstd_to_csv("csv", "--input-zstd-fragment", "dbn.frag.zst", 3)]
#[case::zstd_to_json("json", "--input-zstd-fragment", "dbn.frag.zst", 2)]
fn input_fragment(
    #[case] output_enc: &str,
    #[case] flag: &str,
    #[case] extension: &str,
    #[case] exp_line_count: usize,
) {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.definition.v3.{extension}"),
            flag,
            &format!("--{output_enc}"),
        ])
        .assert()
        .success()
        .stdout(contains('\n').count(exp_line_count))
        .stderr(is_empty());
}

#[rstest]
fn upgraded_definitions_data_is_same_but_larger(output_dir: TempDir) {
    let orig_csv = format!("{}/a.csv", output_dir.path().to_str().unwrap());
    let new_csv = format!("{}/b.csv", output_dir.path().to_str().unwrap());
    let upgraded_dbn_output = format!("{}/a.dbn.zst", output_dir.path().to_str().unwrap());
    let input_path = format!("{TEST_DATA_PATH}/test_data.definition.v1.dbn.zst");
    cmd()
        .args([&input_path, "--csv", "--output", &orig_csv])
        .assert()
        .success()
        .stderr(is_empty())
        .stdout(is_empty());
    let orig_csv_contents = std::fs::read_to_string(orig_csv).unwrap();
    let mut upgrade_cmd = cmd();
    upgrade_cmd.args([&input_path, "--upgrade", "--output", &upgraded_dbn_output]);
    upgrade_cmd
        .assert()
        .success()
        .stderr(is_empty())
        .stdout(is_empty());
    cmd()
        .args([&upgraded_dbn_output, "--csv", "--output", &new_csv])
        .assert()
        .success()
        .stderr(is_empty())
        .stdout(is_empty());
    let new_csv_contents = std::fs::read_to_string(new_csv).unwrap();
    let orig_lines: Vec<_> = orig_csv_contents.lines().collect();
    let new_lines: Vec<_> = new_csv_contents.lines().collect();
    assert_eq!(orig_lines.len(), new_lines.len());
    for (orig_line, new_line) in new_lines.into_iter().zip(orig_lines.into_iter()) {
        // V3 adds new fields only at the end
        new_line.starts_with(orig_line);
    }
    assert!(
        std::fs::metadata(input_path).unwrap().len()
            < std::fs::metadata(upgraded_dbn_output).unwrap().len()
    );
}

#[rstest]
#[case::uncompressed_trades("--input-fragment", Schema::Trades, "dbn.frag", "")]
#[case::zstd_trades("--input-zstd-fragment", Schema::Trades, "dbn.frag.zst", "--zstd")]
#[case::uncompressed_mbo("--input-fragment", Schema::Mbo, "dbn.frag", "")]
#[case::zstd_mbo("--input-zstd-fragment", Schema::Mbo, "dbn.frag.zst", "--zstd")]
#[case::uncompressed_definition("--input-fragment", Schema::Definition, "dbn.frag", "")]
#[case::zstd_definition("--input-zstd-fragment", Schema::Definition, "dbn.frag.zst", "--zstd")]
fn write_fragment(
    output_dir: TempDir,
    #[case] input_flag: &str,
    #[case] schema: Schema,
    #[case] extension: &str,
    #[case] zstd_flag: &str,
) {
    let orig_csv = format!("{}/a.csv", output_dir.path().to_str().unwrap());
    let frag_output = format!("{}/a.{extension}", output_dir.path().to_str().unwrap());
    let input_path = format!("{TEST_DATA_PATH}/test_data.{schema}.v3.dbn.zst");
    cmd()
        .args([&input_path, "--csv", "--output", &orig_csv])
        .assert()
        .success()
        .stderr(is_empty())
        .stdout(is_empty());
    let orig_csv_contents = std::fs::read_to_string(orig_csv).unwrap();
    let mut write_frag_cmd = cmd();
    write_frag_cmd.args([&input_path, "--fragment", "--output", &frag_output]);
    if !zstd_flag.is_empty() {
        write_frag_cmd.arg(zstd_flag);
    }
    write_frag_cmd
        .assert()
        .success()
        .stderr(is_empty())
        .stdout(is_empty());
    cmd()
        .args([&frag_output, input_flag, "--csv"])
        .assert()
        .success()
        .stderr(is_empty())
        .stdout(eq(orig_csv_contents));
}

#[rstest]
fn test_limit_updates_metadata(output_dir: TempDir) {
    // Check metadata shows limit = 2
    let input = format!("{TEST_DATA_PATH}/test_data.definition.v3.dbn.zst");
    cmd()
        .args([&input, "--json", "--metadata"])
        .assert()
        .success()
        .stdout(contains("\"limit\":\"2\","));
    // Check contains 2 records
    cmd()
        .args([&input, "--json"])
        .assert()
        .success()
        .stdout(contains('\n').count(2));
    let output_path = format!("{}/limited.dbn", output_dir.path().to_str().unwrap());
    cmd()
        .args([&input, "--output", &output_path, "--limit", "1"])
        .assert()
        .success()
        .stderr(is_empty());
    // Check metadata shows limit = 1
    cmd()
        .args([&output_path, "--json", "--metadata"])
        .assert()
        .success()
        .stdout(contains("\"limit\":\"1\","));
    // Check contains 1 record
    cmd()
        .args([&output_path, "--json"])
        .assert()
        .success()
        .stdout(contains('\n').count(1));
}

#[cfg(not(target_os = "windows"))]
#[rstest]
#[case::uncompressed_to_csv("test_data.mbo.v3.dbn", "--csv", "")]
#[case::zstd_to_csv("test_data.mbo.v3.dbn.zst", "--csv", "")]
#[case::uncompressed_fragment_to_csv(
    "test_data.definition.v3.dbn.frag",
    "--csv",
    "--input-fragment"
)]
#[case::zstd_fragment_to_csv(
    "test_data.definition.v3.dbn.frag.zst",
    "--csv",
    "--input-zstd-fragment"
)]
#[case::uncompressed_to_json("test_data.mbo.v3.dbn", "--json", "")]
#[case::zstd_to_json("test_data.mbo.v3.dbn.zst", "--json", "")]
#[case::uncompressed_fragment_to_json(
    "test_data.definition.v3.dbn.frag",
    "--json",
    "--input-fragment"
)]
#[case::zstd_fragment_to_json(
    "test_data.definition.v3.dbn.frag.zst",
    "--json",
    "--input-zstd-fragment"
)]
fn broken_pipe_is_silent(
    #[case] file_name: &str,
    #[case] output_flag: &str,
    #[case] fragment_flag: &str,
) {
    let mut dbn_cmd = process::Command::new(assert_cmd::cargo::cargo_bin!("dbn"));
    dbn_cmd.args([&format!("{TEST_DATA_PATH}/{file_name}"), output_flag]);
    if !fragment_flag.is_empty() {
        dbn_cmd.arg(fragment_flag);
    }
    let mut dbn_res = dbn_cmd
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .unwrap();
    dbn_res.wait().unwrap();
    let mut false_cmd = process::Command::new("false");
    false_cmd.stdin(dbn_res.stdout.unwrap());
    Command::from_std(false_cmd)
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(is_empty());
    let mut stderr = String::new();
    dbn_res.stderr.unwrap().read_to_string(&mut stderr).unwrap();
    assert!(stderr.is_empty(), "Stderr: {stderr}");
}

#[test]
fn writes_csv_header_for_0_records() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.ohlcv-1d.v3.dbn.zst"),
            "--csv",
        ])
        .assert()
        .success()
        .stdout(starts_with("ts_event,").and(contains('\n').count(1)))
        .stderr(is_empty());
}

#[rstest]
#[case::csv("--csv")]
#[case::json("--json")]
fn map_symbols(#[case] output_flag: &str) {
    let cmd = cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            output_flag,
            "--map-symbols",
        ])
        .assert()
        .success()
        .stderr(is_empty());
    if output_flag == "--csv" {
        cmd.stdout(contains(",symbol\n").count(1));
    } else {
        cmd.stdout(contains("\"symbol\":\"").count(2));
    }
}

#[rstest]
#[case::dbn("--dbn")]
#[case::fragment("--fragment")]
fn map_symbols_fails_for_invalid_output(#[case] output_flag: &str) {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            output_flag,
            "--map-symbols",
        ])
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains(format!("'{output_flag}'")).and(contains("'--map-symbols'")));
}

#[test]
fn passing_current_dbn_version_is_accepted() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.definition.v3.dbn.frag"),
            "--input-fragment",
            "--input-dbn-version",
            &dbn::DBN_VERSION.to_string(),
            "--json",
        ])
        .assert()
        .success()
        .stderr(is_empty());
}

#[test]
fn passing_next_dbn_version_is_rejected() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.definition.v1.dbn.frag"),
            "--input-fragment",
            "--input-dbn-version",
            &(dbn::DBN_VERSION + 1).to_string(),
            "--json",
        ])
        .assert()
        .failure()
        .stderr(contains("invalid value").and(contains("--input-dbn-version")));
}

#[rstest]
#[case::csv("--csv")]
#[case::dbn("--dbn")]
#[case::fragment("--fragment")]
#[case::json("--json")]
fn passing_dbn_version_conflicts_with_non_fragment_input(#[case] output_flag: &str) {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            output_flag,
            "--input-dbn-version",
            "1",
        ])
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains(
            "the following required arguments were not provided:",
        ));
}

#[rstest]
#[case::csv("csv", ',')]
#[case::csv("tsv", '\t')]
fn omit_csv_header(#[case] output_enc: &str, #[case] sep: char) {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            &format!("--{output_enc}"),
            "--omit-header",
        ])
        .assert()
        .success()
        .stdout(
            contains('\n')
                .count(2)
                .and(contains(format!("ts_event{sep}")).not()),
        )
        .stderr(is_empty());
}

#[rstest]
#[case::csv("csv", ',')]
#[case::csv("tsv", '\t')]
fn omit_csv_header_fragment(#[case] output_enc: &str, #[case] sep: char) {
    cmd()
        .args([
            "--input-zstd-fragment",
            &format!("{TEST_DATA_PATH}/test_data.definition.v3.dbn.frag.zst"),
            &format!("--{output_enc}"),
            "--omit-header",
        ])
        .assert()
        .success()
        .stdout(
            contains('\n')
                .count(2)
                .and(contains(format!("ts_event{sep}")).not()),
        )
        .stderr(is_empty());
}

#[rstest]
#[case::dbn("dbn")]
#[case::json("json")]
#[case::fragment("fragment")]
fn omit_header_conflicts(#[case] output_enc: &str) {
    cmd()
        .args([
            "--input-zstd-fragment",
            &format!("{TEST_DATA_PATH}/test_data.definition.v3.dbn.frag.zst"),
            &format!("--{output_enc}"),
            "--omit-header",
        ])
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("--omit-header"));
}

#[test]
fn test_merge_mbo_and_mbp10_diff_compression() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbp-10.v3.dbn.zst"),
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn"),
            "--json",
        ])
        .assert()
        .success()
        .stdout(
            contains("\"rtype\":160,")
                .count(2)
                .and(contains("\"rtype\":10,").count(2)),
        )
        .stderr(is_empty());
}

#[test]
fn test_merge_mbo_and_mbp10_metadata() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbp-10.v3.dbn.zst"),
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn"),
            "--json",
            "--metadata",
        ])
        .assert()
        .success()
        .stdout(contains("\"schema\":null,"))
        .stderr(is_empty());
}

#[test]
fn help() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("Usage:"))
        .stderr(is_empty());
}

#[test]
fn empty() {
    cmd().assert().failure().stdout(is_empty()).stderr(
        contains("the following required arguments were not provided:\n  <FILE...>")
            .and(contains("Usage:")),
    );
}

#[test]
fn version() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains(env!("CARGO_PKG_VERSION")))
        .stderr(is_empty());
}
