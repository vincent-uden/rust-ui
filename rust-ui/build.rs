use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let dll_src = PathBuf::from("./x86_64/freetype.dll");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // OUT_DIR is something like target/debug/build/<pkg>/out
    // The final binary is one level up in target/debug or target/release
    let target_dir = out_dir
        .ancestors()
        .nth(3) // walk up to target/{debug,release}
        .unwrap()
        .to_path_buf();

    let dll_dst = target_dir.join("freetype.dll");

    // Copy DLL if it exists
    if let Err(e) = fs::copy(&dll_src, &dll_dst) {
        panic!(
            "Failed to copy {} to {}: {}",
            dll_src.display(),
            dll_dst.display(),
            e
        );
    }

    // Re-run build.rs if the DLL changes
    println!("cargo:rerun-if-changed={}", dll_src.display());
}
