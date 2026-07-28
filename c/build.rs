//! Writes generated C header for DBN functions and symbols to
//! ${target_directory}/include/dbn/dbn.h, or to `$DBN_C_HEADER_DIR` when set

extern crate cbindgen;

use std::{env, ffi::OsStr, fs, path::PathBuf};

fn find_target_dir() -> PathBuf {
    if let Some(target_dir) = env::var_os("CARGO_TARGET_DIR") {
        return PathBuf::from(target_dir);
    }
    let mut dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    loop {
        if dir.file_name() == Some(OsStr::new("target"))
            // Want to find the top directory containing a CACHEDIR.TAG file
            || (dir.join("CACHEDIR.TAG").exists()
                && !dir
                    .parent().is_none_or(|p| p.join("CACHEDIR.TAG").exists()))
        {
            return dir;
        }
        assert!(dir.pop(), "Unable to determine target dir");
    }
}

fn header_dir() -> PathBuf {
    if let Some(header_dir) = env::var_os("DBN_C_HEADER_DIR") {
        return PathBuf::from(header_dir);
    }
    find_target_dir().join("include").join("dbn")
}

fn main() {
    // Emitting any rerun-if directive opts out of cargo's default of rerunning when any
    // file in the package changes, so every input has to be declared. cbindgen is
    // configured with `parse_deps`, which makes the `dbn` crate's sources an input too.
    println!("cargo:rerun-if-env-changed=DBN_C_HEADER_DIR");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=../rust/dbn/src");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let header_dir = header_dir();
    fs::create_dir_all(&header_dir).unwrap();
    let out_path = header_dir.join("dbn.h");

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(out_path);
}
