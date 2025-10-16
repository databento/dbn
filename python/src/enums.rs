#[cfg(test)]
mod tests {
    use pyo3::{ffi::c_str, prelude::*, types::PyDict, Python};
    use rstest::*;

    use crate::tests::python;

    #[rstest]
    #[case("Compression")]
    #[case("Encoding")]
    #[case("Schema")]
    #[case("SType")]
    fn test_enum_name_coercion(_python: (), #[case] enum_name: &str) {
        Python::attach(|py| {
            let globals = PyDict::new(py);
            globals.set_item("enum_name", enum_name).unwrap();
            Python::run(
                py,
                c_str!(
                    r#"import _lib as db

enum_type = getattr(db, enum_name)
for variant in enum_type.variants():
    assert variant == enum_type(variant.name)
    assert variant == enum_type(variant.name.replace('_', '-'))
    assert variant == enum_type(variant.name.lower())
    assert variant == enum_type(variant.name.upper())
    try:
        enum_type("bar")     # sanity check
        assert False, "did not raise an exception"
    except db.DBNError:
        pass"#
                ),
                Some(&globals),
                None,
            )
            .unwrap();
        });
    }

    #[rstest]
    fn test_compression_none_coercible(_python: ()) {
        Python::attach(|py| {
            py.run(
                c_str!(
                    r#"import _lib as db

assert db.Compression(None) == db.Compression.NONE
                "#
                ),
                // Create an empty `globals` dict to keep tests hermetic
                Some(&PyDict::new(py)),
                None,
            )
            .unwrap()
        });
    }

    #[rstest]
    #[case("Encoding")]
    #[case("Schema")]
    #[case("SType")]
    fn test_enum_none_not_coercible(_python: (), #[case] enum_name: &str) {
        Python::attach(|py| {
            let globals = PyDict::new(py);
            globals.set_item("enum_name", enum_name).unwrap();
            Python::run(
                py,
                c_str!(
                    r#"import _lib as db

enum_type = getattr(db, enum_name)
try:
    enum_type(None)
    assert False, "did not raise an exception"
except db.DBNError:
    pass"#
                ),
                Some(&globals),
                None,
            )
            .unwrap();
        });
    }
}
