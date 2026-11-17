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
    upgrade_cmd.args([
        &input_path,
        "--upgrade",
        "3",
        "--output",
        &upgraded_dbn_output,
    ]);
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
fn map_symbols_parquet(output_dir: TempDir) {
    use arrow::array::{Array, StringArray};
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

    let output_path = format!("{}/out.parquet", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--output",
            &output_path,
            "--parquet",
            "--map-symbols",
        ])
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());

    let file = std::fs::File::open(&output_path).unwrap();
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .unwrap()
        .build()
        .unwrap();
    let batches: Vec<_> = reader.into_iter().map(|b| b.unwrap()).collect();
    let batch = arrow::compute::concat_batches(&batches[0].schema(), &batches).unwrap();
    let symbols = batch
        .column_by_name("symbol")
        .expect("symbol column should be present in parquet output")
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("symbol column should be Utf8");
    assert!(batch.num_rows() > 0);
    let any_populated = (0..batch.num_rows()).any(|i| !symbols.is_null(i));
    assert!(any_populated, "expected at least one non-null symbol");
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

#[rstest]
fn split_by_day(output_dir: TempDir) {
    let output_pattern = format!("{}/{{date}}.json", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--split-by",
            "day",
            "--output-pattern",
            &output_pattern,
            "--json",
        ])
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let output_path = format!("{}/2020-12-28.json", output_dir.path().to_str().unwrap());
    let contents = fs::read_to_string(&output_path).unwrap();
    assert!(contents.contains("\"rtype\":160"));
    assert_eq!(contents.lines().count(), 2);
}

#[rstest]
fn split_by_week(output_dir: TempDir) {
    let output_pattern = format!("{}/{{date}}.json", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--split-by",
            "week",
            "--output-pattern",
            &output_pattern,
            "--json",
        ])
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    // 2020-12-28 is a Monday, week starts on Sunday 2020-12-27
    let output_path = format!("{}/2020-12-27.json", output_dir.path().to_str().unwrap());
    let contents = fs::read_to_string(&output_path).unwrap();
    assert!(contents.contains("\"rtype\":160"));
}

#[rstest]
fn split_by_month(output_dir: TempDir) {
    let output_pattern = format!("{}/{{date}}.json", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--split-by",
            "month",
            "--output-pattern",
            &output_pattern,
            "--json",
        ])
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let output_path = format!("{}/2020-12-01.json", output_dir.path().to_str().unwrap());
    let contents = fs::read_to_string(&output_path).unwrap();
    assert!(contents.contains("\"rtype\":160"));
}

#[rstest]
fn split_by_year(output_dir: TempDir) {
    let output_pattern = format!("{}/{{date}}.json", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--split-by",
            "year",
            "--output-pattern",
            &output_pattern,
            "--json",
        ])
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let output_path = format!("{}/2020-01-01.json", output_dir.path().to_str().unwrap());
    let contents = fs::read_to_string(&output_path).unwrap();
    assert!(contents.contains("\"rtype\":160"));
}

#[rstest]
fn split_by_symbol(output_dir: TempDir) {
    let output_pattern = format!("{}/{{symbol}}.json", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--split-by",
            "symbol",
            "--output-pattern",
            &output_pattern,
            "--json",
        ])
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let output_path = format!("{}/ESH1.json", output_dir.path().to_str().unwrap());
    let contents = fs::read_to_string(&output_path).unwrap();
    assert!(contents.contains("\"rtype\":160"));
}

#[test]
fn output_pattern_requires_split_by() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--output-pattern",
            "{date}.json",
            "--json",
        ])
        .assert()
        .failure()
        .stderr(contains("split-by"));
}

#[test]
fn split_by_requires_output_pattern() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--split-by",
            "day",
            "--json",
        ])
        .assert()
        .failure()
        .stderr(contains("output-pattern"));
}

#[test]
fn output_pattern_conflicts_with_output() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--split-by",
            "day",
            "--output",
            "out.json",
            "--output-pattern",
            "{date}.json",
            "--json",
        ])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
}

#[rstest]
fn split_by_day_fragment(output_dir: TempDir) {
    let output_pattern = format!("{}/{{date}}.dbn", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--split-by",
            "day",
            "--output-pattern",
            &output_pattern,
            "--fragment",
        ])
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    // Verify the fragment file exists and can be decoded
    let output_path = format!("{}/2020-12-28.dbn", output_dir.path().to_str().unwrap());
    let contents = std::fs::read(&output_path).unwrap();
    assert_ne!(
        &contents[..3],
        b"DBN",
        "Fragment should not have metadata header"
    );
    assert!(!contents.is_empty());
}

#[rstest]
fn split_input_fragment_by_day_to_json(output_dir: TempDir) {
    let output_pattern = format!("{}/{{date}}.json", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.definition.v3.dbn.frag"),
            "--input-fragment",
            "--split-by",
            "day",
            "--output-pattern",
            &output_pattern,
        ])
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let output_path = format!("{}/2020-12-27.json", output_dir.path().to_str().unwrap());
    let contents = std::fs::read_to_string(&output_path).unwrap();
    assert!(contents.contains("\"rtype\""), "Output should be JSON");
    assert!(!contents.is_empty());
}

#[rstest]
fn split_input_fragment_by_day_to_fragment(output_dir: TempDir) {
    let output_pattern = format!("{}/{{date}}.dbn.frag", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.definition.v3.dbn.frag"),
            "--input-fragment",
            "--split-by",
            "day",
            "--output-pattern",
            &output_pattern,
        ])
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let output_path = format!(
        "{}/2020-12-27.dbn.frag",
        output_dir.path().to_str().unwrap()
    );
    // Verify file exists and is a fragment: no DBN header
    let contents = std::fs::read(&output_path).unwrap();
    assert_ne!(
        &contents[..3],
        b"DBN",
        "Fragment should not have metadata header"
    );
    assert!(!contents.is_empty());
}

#[test]
fn split_input_fragment_by_symbol_fails() {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.definition.v3.dbn.frag"),
            "--input-fragment",
            "--split-by",
            "symbol",
            "--output-pattern",
            "{symbol}.dbn.frag",
        ])
        .assert()
        .failure()
        .stderr(contains("Cannot split by symbol when input is a fragment"));
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

// Parquet magic bytes: "PAR1"
const PARQUET_MAGIC: &[u8; 4] = b"PAR1";

#[rstest]
fn write_parquet_with_flag(output_dir: TempDir) {
    let output_path = format!("{}/out.parquet", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--output",
            &output_path,
            "--parquet",
        ])
        .assert()
        .success()
        .stdout(is_empty());
    let contents = fs::read(&output_path).unwrap();
    assert!(contents.len() > 8, "Parquet file too small");
    assert_eq!(
        &contents[..4],
        PARQUET_MAGIC,
        "missing Parquet header magic"
    );
    assert_eq!(
        &contents[contents.len() - 4..],
        PARQUET_MAGIC,
        "missing Parquet footer magic"
    );
}

#[rstest]
fn write_parquet_infer_from_extension(output_dir: TempDir) {
    let output_path = format!("{}/out.parquet", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbp-1.v3.dbn.zst"),
            "--output",
            &output_path,
        ])
        .assert()
        .success()
        .stdout(is_empty());
    let contents = fs::read(&output_path).unwrap();
    assert_eq!(&contents[..4], PARQUET_MAGIC);
}

#[test]
fn write_parquet_to_stdout() {
    let output = cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--parquet",
        ])
        .assert()
        .success();
    let stdout = output.get_output().stdout.clone();
    assert!(stdout.len() > 8, "stdout should contain Parquet output");
    assert_eq!(
        &stdout[..4],
        PARQUET_MAGIC,
        "stdout should begin with Parquet magic"
    );
}

// Arrow IPC file magic: "ARROW1\0\0"
const ARROW_IPC_MAGIC: &[u8; 6] = b"ARROW1";

#[rstest]
fn write_arrow_with_flag(output_dir: TempDir) {
    let output_path = format!("{}/out.arrow", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--output",
            &output_path,
            "--arrow",
        ])
        .assert()
        .success()
        .stdout(is_empty());
    let contents = fs::read(&output_path).unwrap();
    assert!(contents.len() > 16, "Arrow IPC file too small");
    assert_eq!(
        &contents[..6],
        ARROW_IPC_MAGIC,
        "missing Arrow IPC header magic"
    );
    assert_eq!(
        &contents[contents.len() - 6..],
        ARROW_IPC_MAGIC,
        "missing Arrow IPC footer magic"
    );
}

#[rstest]
fn write_arrow_infer_from_extension(output_dir: TempDir) {
    let output_path = format!("{}/out.feather", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbp-1.v3.dbn.zst"),
            "--output",
            &output_path,
        ])
        .assert()
        .success()
        .stdout(is_empty());
    let contents = fs::read(&output_path).unwrap();
    assert_eq!(&contents[..6], ARROW_IPC_MAGIC);
}

#[rstest]
#[case::parquet("parquet")]
#[case::arrow("arrow")]
fn omit_header_conflicts_with_columnar(#[case] output_enc: &str) {
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            &format!("--{output_enc}"),
            "--omit-header",
        ])
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("--omit-header"));
}

#[rstest]
#[case::parquet("parquet")]
#[case::arrow("arrow")]
fn input_fragment_conflicts_with_columnar(#[case] output_enc: &str) {
    cmd()
        .args([
            "--input-zstd-fragment",
            &format!("{TEST_DATA_PATH}/test_data.definition.v3.dbn.frag.zst"),
            &format!("--{output_enc}"),
        ])
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("cannot be used with"));
}

#[rstest]
fn write_arrow_zstd_reads_back(output_dir: TempDir) {
    use arrow::ipc::reader::FileReader;

    let output_path = format!("{}/out.arrow", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
            "--output",
            &output_path,
            "--arrow",
            "--zstd",
        ])
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let reader = FileReader::try_new(fs::File::open(&output_path).unwrap(), None).unwrap();
    assert!(reader.schema().field_with_name("price").is_ok());
    let num_rows: usize = reader.map(|batch| batch.unwrap().num_rows()).sum();
    assert_eq!(num_rows, 2);
}

#[rstest]
fn write_parquet_zero_rows_with_schema_filter(output_dir: TempDir) {
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

    let output_path = format!("{}/out.parquet", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn"),
            "--output",
            &output_path,
            "--parquet",
            "--schema",
            "trades",
        ])
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let builder =
        ParquetRecordBatchReaderBuilder::try_new(fs::File::open(&output_path).unwrap()).unwrap();
    assert_eq!(builder.metadata().file_metadata().num_rows(), 0);
    assert!(builder.schema().field_with_name("price").is_ok());
    assert!(builder.schema().field_with_name("order_id").is_err());
    assert!(builder.schema().metadata()["databento:metadata"].contains("\"schema\":\"trades\""));
    let num_rows: usize = builder
        .build()
        .unwrap()
        .map(|batch| batch.unwrap().num_rows())
        .sum();
    assert_eq!(num_rows, 0);
}

fn stdout_of<'a>(args: impl IntoIterator<Item = &'a str>) -> String {
    let output = cmd().args(args).assert().success().get_output().clone();
    String::from_utf8(output.stdout).unwrap()
}

fn upgrade_to_v4(input: &str, output_dir: &TempDir) -> String {
    let file_name = input.rsplit('/').next().unwrap();
    let output_path = format!("{}/{file_name}.v4.dbn", output_dir.path().to_str().unwrap());
    cmd()
        .args([input, "--upgrade", "4", "--dbn", "--output", &output_path])
        .assert()
        .success()
        .stderr(is_empty());
    output_path
}

#[rstest]
fn v4_text_matches_v3(
    output_dir: TempDir,
    #[values("--csv", "--json")] output_flag: &str,
    #[values(None, Some("--map-symbols"), Some("--pretty"))] extra_flag: Option<&str>,
) {
    let input = format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst");
    let v4_input = upgrade_to_v4(&input, &output_dir);
    let flags = [Some(output_flag), extra_flag].into_iter().flatten();
    let v3_output = stdout_of([input.as_str()].into_iter().chain(flags.clone()));
    assert_eq!(
        stdout_of([v4_input.as_str()].into_iter().chain(flags.clone())),
        v3_output
    );
    assert_eq!(
        stdout_of([input.as_str(), "--upgrade", "4"].into_iter().chain(flags)),
        v3_output
    );
}

#[rstest]
fn v4_metadata(output_dir: TempDir) {
    let v4_input = upgrade_to_v4(
        &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
        &output_dir,
    );
    cmd()
        .args([&v4_input, "--json", "--metadata"])
        .assert()
        .success()
        .stdout(starts_with("{\"version\":4,"))
        .stderr(is_empty());
}

#[rstest]
fn merge_v4_matches_v3(output_dir: TempDir) {
    let mbo = format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst");
    let ohlcv = format!("{TEST_DATA_PATH}/test_data.ohlcv-1s.v3.dbn.zst");
    let mbo_v4 = upgrade_to_v4(&mbo, &output_dir);
    let ohlcv_v4 = upgrade_to_v4(&ohlcv, &output_dir);
    let v3_output = stdout_of([mbo.as_str(), &ohlcv, "--json"]);
    assert_eq!(v3_output.lines().count(), 4);
    assert_eq!(stdout_of([mbo_v4.as_str(), &ohlcv_v4, "--json"]), v3_output);
    assert_eq!(
        stdout_of([mbo_v4.as_str(), &ohlcv, "--json", "--upgrade", "4"]),
        v3_output
    );
}

#[rstest]
fn split_v4_by_schema_matches_v3(output_dir: TempDir) {
    let dir = output_dir.path().to_str().unwrap();
    let mbo = format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst");
    let ohlcv = format!("{TEST_DATA_PATH}/test_data.ohlcv-1s.v3.dbn.zst");
    for version in ["3", "4"] {
        let merged = format!("{dir}/merged.v{version}.dbn");
        cmd()
            .args([
                &mbo,
                &ohlcv,
                "--upgrade",
                version,
                "--dbn",
                "--output",
                &merged,
            ])
            .assert()
            .success();
        cmd()
            .args([
                &merged,
                "--split-by",
                "schema",
                "--output-pattern",
                &format!("{dir}/{{schema}}.v{version}.csv"),
            ])
            .assert()
            .success()
            .stderr(is_empty());
    }
    for schema in ["mbo", "ohlcv-1s"] {
        let v4_output = fs::read_to_string(format!("{dir}/{schema}.v4.csv")).unwrap();
        assert_eq!(v4_output.lines().count(), 3);
        assert_eq!(
            v4_output,
            fs::read_to_string(format!("{dir}/{schema}.v3.csv")).unwrap()
        );
    }
}

#[rstest]
fn read_v4_zstd_from_file_and_stdin(output_dir: TempDir) {
    let input = format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst");
    let v4_path = format!("{}/mbo.v4.dbn.zst", output_dir.path().to_str().unwrap());
    cmd()
        .args([
            &input,
            "--upgrade",
            "4",
            "--dbn",
            "--zstd",
            "--output",
            &v4_path,
        ])
        .assert()
        .success();
    let v3_output = stdout_of([input.as_str(), "--json"]);
    assert_eq!(stdout_of([v4_path.as_str(), "--json"]), v3_output);
    let stdin_output = cmd()
        .args(["-", "--json"])
        .pipe_stdin(&v4_path)
        .unwrap()
        .ok()
        .unwrap();
    assert_eq!(String::from_utf8(stdin_output.stdout).unwrap(), v3_output);
}

#[rstest]
fn split_v4_by_day_matches_input(output_dir: TempDir) {
    let dir = output_dir.path().to_str().unwrap();
    let v4_input = upgrade_to_v4(
        &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
        &output_dir,
    );
    cmd()
        .args([
            &v4_input,
            "--split-by",
            "day",
            "--output-pattern",
            &format!("{dir}/{{date}}.dbn"),
        ])
        .assert()
        .success()
        .stderr(is_empty());
    assert_eq!(
        stdout_of([format!("{dir}/2020-12-28.dbn").as_str(), "--json"]),
        stdout_of([v4_input.as_str(), "--json"])
    );
}

#[rstest]
fn merge_fragments_upgraded_to_v4() {
    let input = format!("{TEST_DATA_PATH}/test_data.definition.v3.dbn.frag");
    let single = stdout_of([
        input.as_str(),
        "--input-fragment",
        "--json",
        "--upgrade",
        "4",
    ]);
    let merged = stdout_of([
        input.as_str(),
        &input,
        "--input-fragment",
        "--json",
        "--upgrade",
        "4",
    ]);
    assert_eq!(merged.lines().count(), 2 * single.lines().count());
}

#[rstest]
fn upgrade_fragment_to_v4() {
    let input = format!("{TEST_DATA_PATH}/test_data.definition.v3.dbn.frag");
    let v3_output = stdout_of([input.as_str(), "--input-fragment", "--json"]);
    let v4_output = stdout_of([
        input.as_str(),
        "--input-fragment",
        "--json",
        "--upgrade",
        "4",
    ]);
    assert_eq!(v4_output.lines().count(), v3_output.lines().count());
    assert!(v4_output.contains("\"raw_symbol\":\"GEM3 P9812\""));
}

#[rstest]
#[case::bare_upgrade(&["--json", "--upgrade"], "a value is required")]
#[case::downgrade(&["--json", "--upgrade", "3"], "Can't downgrade DBN version 4")]
#[case::fragment_output(&["--fragment"], "fragment output isn't supported")]
#[case::parquet(&["--parquet", "--output", "out.parquet"], "Parquet and Arrow output don't support")]
#[case::split_by_symbol(
    &["--csv", "--split-by", "symbol", "--output-pattern", "{symbol}.csv"],
    "by symbol isn't supported"
)]
fn v4_unsupported_args(output_dir: TempDir, #[case] args: &[&str], #[case] expected_err: &str) {
    let v4_input = upgrade_to_v4(
        &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
        &output_dir,
    );
    cmd()
        .current_dir(output_dir.path())
        .arg(&v4_input)
        .args(args)
        .assert()
        .failure()
        .stderr(contains(expected_err));
}

#[rstest]
fn merge_v4_with_earlier_version_requires_upgrade(output_dir: TempDir) {
    let mbo_v4 = upgrade_to_v4(
        &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
        &output_dir,
    );
    cmd()
        .args([
            &mbo_v4,
            &format!("{TEST_DATA_PATH}/test_data.ohlcv-1s.v3.dbn.zst"),
            "--json",
        ])
        .assert()
        .failure()
        .stderr(contains("requires `--upgrade 4`"));
}

const REGROUPED_ID: u64 = 9999;
const REGROUPED_SYMBOL: &str = "NEWSYM";

fn write_v4_with_record_groups(output_dir: &TempDir) -> String {
    use dbn::{
        decode::{dbn::fsm::DbnFsm, DbnMetadata, DecodeRecord},
        encode::dbn::v4::Encoder,
        v4, SymbolInterval,
    };

    let upgraded = upgrade_to_v4(
        &format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst"),
        output_dir,
    );
    let fsm = DbnFsm::builder()
        .header::<v4::RecordHeader>()
        .build()
        .unwrap();
    let decoder =
        dbn::decode::dbn::v4::Decoder::with_fsm(fs::File::open(upgraded).unwrap(), fsm).unwrap();
    let metadata = decoder.metadata().clone();
    let mut records = decoder.decode_records::<v4::MboMsg>().unwrap();
    assert_eq!(records.len(), 2);
    records[1].hd.instrument_id = REGROUPED_ID;

    let output_path = format!("{}/groups.v4.dbn", output_dir.path().to_str().unwrap());
    let mut encoder = Encoder::new(fs::File::create(&output_path).unwrap(), &metadata).unwrap();
    encoder
        .encode_record_group_header(records[0].ts_recv, &[])
        .unwrap();
    encoder.encode(&records[0]).unwrap();
    encoder
        .encode_record_group_header(
            records[1].ts_recv,
            &[SymbolInterval {
                start_ts: records[1].ts_recv,
                end_ts: metadata.end.unwrap().get(),
                instrument_id: REGROUPED_ID,
                stype_in_symbol: REGROUPED_SYMBOL.to_owned(),
                stype_out_symbol: REGROUPED_ID.to_string(),
            }],
        )
        .unwrap();
    encoder.encode(&records[1]).unwrap();
    output_path
}

#[rstest]
fn v4_dbn_output_passes_through_record_groups(
    output_dir: TempDir,
    #[values(&[], &["--schema", "mbo", "--zstd"], &["--limit", "1"])] extra_args: &[&str],
) {
    use dbn::{
        decode::{dbn::fsm::DbnFsm, DynReader},
        v4,
    };

    let input = write_v4_with_record_groups(&output_dir);
    let output_path = format!("{}/out.v4.dbn", output_dir.path().to_str().unwrap());
    cmd()
        .args([&input, "--dbn", "--output", &output_path])
        .args(extra_args)
        .assert()
        .success()
        .stderr(is_empty());

    let fsm = DbnFsm::builder()
        .header::<v4::RecordHeader>()
        .build()
        .unwrap();
    let mut decoder =
        dbn::decode::dbn::v4::Decoder::with_fsm(DynReader::from_file(&output_path).unwrap(), fsm)
            .unwrap()
            .with_symbol_map()
            .unwrap();
    let mut symbols = Vec::new();
    while let Some((_, symbol)) = decoder.decode_ref_with_symbol().unwrap() {
        symbols.push(symbol.map(str::to_owned));
    }
    let expected = [Some("ESH1".to_owned()), Some(REGROUPED_SYMBOL.to_owned())];
    let expected_len = if extra_args.contains(&"--limit") {
        1
    } else {
        2
    };
    assert_eq!(symbols, expected[..expected_len]);
    if extra_args.is_empty() {
        assert_eq!(fs::read(&output_path).unwrap(), fs::read(&input).unwrap());
    }
}

#[rstest]
#[case::map_symbols(&["--csv", "--map-symbols"], false)]
#[case::merged_dbn(&["--dbn"], true)]
#[case::split_dbn(&["--split-by", "day", "--output-pattern", "{date}.dbn"], false)]
fn v4_record_groups_unsupported(
    output_dir: TempDir,
    #[case] args: &[&str],
    #[case] is_merged: bool,
) {
    let input = write_v4_with_record_groups(&output_dir);
    let mut cmd = cmd();
    cmd.current_dir(output_dir.path()).arg(&input);
    if is_merged {
        cmd.arg(&input);
    }
    cmd.args(args)
        .assert()
        .failure()
        .stderr(contains("Record groups aren't supported yet"));
}

#[rstest]
fn v4_text_output_skips_record_groups(output_dir: TempDir) {
    let input = write_v4_with_record_groups(&output_dir);
    cmd()
        .args([&input, &input, "--csv"])
        .assert()
        .success()
        .stdout(contains('\n').count(5))
        .stderr(is_empty());
}
