//! Writes generated C header for DBN functions and symbols to
//! ${target_directory}/include/dbn/dbn.h

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

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_dir = find_target_dir();
    let include_dir = target_dir.join("include").join("dbn");
    fs::create_dir_all(&include_dir).unwrap();
    let out_path = include_dir.join("dbn.h");

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(out_path);
}
