#[cfg(test)]
mod tests {
    use pyo3::{prelude::*, types::PyDict, Python};
    use rstest::*;

    use crate::tests::{python, TEST_DATA_PATH};

    const N_THREADS: usize = 8;

    fn run(py: Python<'_>, globals: &Bound<'_, PyDict>, code: &str) {
        Python::run(
            py,
            &std::ffi::CString::new(code).unwrap(),
            Some(globals),
            None,
        )
        .unwrap();
    }

    fn globals<'py>(py: Python<'py>, file: &str) -> Bound<'py, PyDict> {
        let globals = PyDict::new(py);
        globals
            .set_item("path", format!("{TEST_DATA_PATH}/{file}"))
            .unwrap();
        globals.set_item("N_THREADS", N_THREADS).unwrap();
        globals
    }

    /// Python helper that runs `work(i)` on `N_THREADS` threads released together by a
    /// barrier, collecting results and any exception.
    const THREADED: &str = r#"
import threading

def run_threaded(work):
    barrier = threading.Barrier(N_THREADS)
    results = [None] * N_THREADS
    errors = []

    def target(i):
        try:
            barrier.wait()
            results[i] = work(i)
        except BaseException as exc:
            errors.append(exc)

    threads = [threading.Thread(target=target, args=(i,)) for i in range(N_THREADS)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    assert not errors, errors
    return results
"#;

    #[rstest]
    fn test_import_keeps_gil_disabled(_python: ()) {
        Python::attach(|py| {
            run(
                py,
                &PyDict::new(py),
                r#"
import sys
import sysconfig

import _lib

if sysconfig.get_config_var("Py_GIL_DISABLED"):
    assert not sys._is_gil_enabled(), "importing _lib re-enabled the GIL"
"#,
            );
        });
    }

    #[rstest]
    #[case("test_data.mbo.v3.dbn", "NONE")]
    #[case("test_data.mbo.v3.dbn.zst", "ZSTD")]
    fn test_concurrent_decoders(_python: (), #[case] file: &str, #[case] compression: &str) {
        Python::attach(|py| {
            let globals = globals(py, file);
            globals.set_item("compression", compression).unwrap();
            run(py, &globals, THREADED);
            run(
                py,
                &globals,
                r#"
from _lib import Compression, DBNDecoder

with open(path, "rb") as f:
    data = f.read()

def decode(_):
    decoder = DBNDecoder(compression=Compression(compression))
    decoder.write(data)
    records = decoder.decode()
    metadata = records[0]
    return (
        metadata.dataset,
        metadata.schema,
        [bytes(r) for r in records[1:]],
        [(r.rtype, r.publisher_id, r.instrument_id, r.ts_event) for r in records[1:]],
    )

expected = decode(0)
assert len(expected[2]) == 2
for result in run_threaded(decode):
    assert result == expected
"#,
            );
        });
    }

    #[rstest]
    fn test_concurrent_transcoders(_python: ()) {
        Python::attach(|py| {
            let globals = globals(py, "test_data.mbo.v3.dbn");
            run(py, &globals, THREADED);
            // `Transcoder.write()` detaches from the interpreter while transcoding and
            // `PyFileLike` re-attaches to write to the `BytesIO`
            run(
                py,
                &globals,
                r#"
import io

from _lib import Compression, Encoding, Transcoder

with open(path, "rb") as f:
    data = f.read()

def transcode(_):
    out = io.BytesIO()
    transcoder = Transcoder(out, Encoding.JSON, Compression.NONE)
    transcoder.write(data)
    transcoder.flush()
    transcoder.finish()
    return out.getvalue()

expected = transcode(0)
assert expected.count(b"\n") == 2
for result in run_threaded(transcode):
    assert result == expected
"#,
            );
        });
    }

    #[rstest]
    fn test_shared_decoder_raises_or_succeeds(_python: ()) {
        Python::attach(|py| {
            let globals = globals(py, "test_data.mbo.v3.dbn");
            run(py, &globals, THREADED);
            // A decoder shared between threads is not supported, but PyO3's runtime borrow
            // check must turn concurrent access into `RuntimeError`, never a crash or a
            // corrupted stream
            run(
                py,
                &globals,
                r#"
import threading

from _lib import DBNDecoder

ITERATIONS = 50

with open(path, "rb") as f:
    data = f.read()
decoder = DBNDecoder()
decoder.write(data)
records = decoder.decode()
record_bytes = b"".join(bytes(r) for r in records[1:])
records_per_write = len(records) - 1

counts = {"written": 0, "decoded": 0}
counts_lock = threading.Lock()

def hammer(_):
    for _ in range(ITERATIONS):
        try:
            decoder.write(record_bytes)
            with counts_lock:
                counts["written"] += 1
        except RuntimeError:
            pass
        try:
            decoded = len(decoder.decode())
            with counts_lock:
                counts["decoded"] += decoded
        except RuntimeError:
            pass

run_threaded(hammer)
counts["decoded"] += len(decoder.decode())
assert counts["written"] > 0
assert counts["decoded"] == counts["written"] * records_per_write, counts
"#,
            );
        });
    }

    #[rstest]
    fn test_concurrent_enum_and_record_construction(_python: ()) {
        Python::attach(|py| {
            let globals = globals(py, "test_data.mbo.v3.dbn");
            run(py, &globals, THREADED);
            run(
                py,
                &globals,
                r#"
from _lib import Action, MBOMsg, SType, Schema, Side

def make():
    return MBOMsg(
        publisher_id=1,
        instrument_id=2,
        ts_event=3,
        order_id=4,
        price=5,
        size=6,
        action=Action.ADD,
        side=Side.BID,
        ts_recv=7,
    )

expected = bytes(make())

def work(_):
    for _ in range(200):
        msg = make()
        assert msg.action == Action.ADD
        assert msg.side == Side.BID
        assert msg.price == 5
        msg.action = Action.CANCEL
        assert msg.action == Action.CANCEL
        assert Action("T") == Action.TRADE
        assert Schema("mbo") == Schema.MBO
        assert SType("raw_symbol") == SType.RAW_SYMBOL
        assert bytes(make()) == expected
        assert str(Schema.MBO) == "mbo"
        assert list(Side.variants())

run_threaded(work)
"#,
            );
        });
    }

    /// Attaches to the interpreter from Rust-created threads, each decoding independently.
    #[rstest]
    fn test_attach_from_rust_threads(_python: ()) {
        std::thread::scope(|scope| {
            for _ in 0..N_THREADS {
                scope.spawn(|| {
                    Python::attach(|py| {
                        let globals = globals(py, "test_data.mbo.v3.dbn");
                        run(
                            py,
                            &globals,
                            r#"
from _lib import DBNDecoder

decoder = DBNDecoder()
with open(path, "rb") as f:
    decoder.write(f.read())
records = decoder.decode()
assert len(records) == 3
"#,
                        );
                    });
                });
            }
        });
    }
}
