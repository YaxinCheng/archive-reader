use std::path::PathBuf;

fn main() {
    let libarchive = pkg_config::Config::new()
        .atleast_version("3.2.0")
        .probe("libarchive")
        .expect("Unable to find libarchive");
    let bindings = bindgen::Builder::default()
        .clang_args(
            libarchive
                .include_paths
                .into_iter()
                .map(|path| format!("-I{}", path.display())),
        )
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Failed to generate binding");
    let mut output_path = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
    output_path.push("bindings.rs");
    bindings
        .write_to_file(output_path)
        .expect("Failed to write");
}
