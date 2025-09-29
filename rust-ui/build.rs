use std::{env, fs, io, path::{Path, PathBuf}};

fn copy_if_exists(src: &Path, dst_dir: &Path) -> io::Result<()> {
    if src.exists() {
        let dst = dst_dir.join(src.file_name().unwrap());
        fs::copy(src, &dst)?;
        println!("cargo:rerun-if-changed={}", src.display());
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.ancestors().nth(3).unwrap().to_path_buf();

    let local_dlls_dir = PathBuf::from("./x86_64");
    if local_dlls_dir.exists() {
        for entry in fs::read_dir(local_dlls_dir)? {
            let path = entry?.path();
            if path
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("dll"))
                .unwrap_or(false)
            {
                copy_if_exists(&path, &target_dir)?;
            }
        }
    }
    let build_dir = out_dir.ancestors().nth(2).unwrap(); // target/{debug,release}/build
    for entry in fs::read_dir(build_dir)? {
        let path = entry?.path();
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("glfw-sys-"))
            .unwrap_or(false)
        {
            for sub in walkdir::WalkDir::new(&path) {
                let sub = sub.unwrap().into_path();
                if sub
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.eq_ignore_ascii_case("glfw3.dll"))
                    .unwrap_or(false)
                    && sub.to_string_lossy().contains("lib-vc2022")
                {
                    copy_if_exists(&sub, &target_dir)?;
                }
            }
        }
    }

    Ok(())
}
