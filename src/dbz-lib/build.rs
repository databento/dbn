fn main() {
    // Sets the correct linker arguments when building with `cargo`
    #[cfg(feature = "python")]
    pyo3_build_config::add_extension_module_link_args();
}
