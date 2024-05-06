use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = cc::Build::new();

    println!("cargo:rerun-if-env-changed=TARGET");
    println!("cargo:rerun-if-env-changed=LFS_DIR");
    println!("cargo:rerun-if-env-changed=LFS_CONFIG");

    let target = env::var("TARGET")?;
    let lfs_dir: PathBuf = env::var("LFS_DIR").unwrap_or("littlefs".into()).into();
    let lfs_config = env::var("LFS_CONFIG").ok();

    let builder = builder.flag("-std=c11");

    let builder = if let Some(lfs_config) = lfs_config {
        builder.flag(format!("-DLFS_CONFIG={lfs_config}").as_str())
    } else {
        builder
            .flag("-DLFS_NO_DEBUG")
            .flag("-DLFS_NO_WARN")
            .flag("-DLFS_NO_ERROR")
    };

    let builder = builder
        .file(lfs_dir.join("lfs.c"))
        .file(lfs_dir.join("lfs_util.c"))
        .file("string.c");

    #[cfg(feature = "software-intrinsics")]
    let builder = builder.flag("-DLFS_NO_INTRINSICS");

    #[cfg(not(feature = "assertions"))]
    let builder = builder.flag("-DLFS_NO_ASSERT");

    #[cfg(feature = "trace")]
    let builder = builder.flag("-DLFS_YES_TRACE");

    #[cfg(not(feature = "malloc"))]
    builder.flag("-DLFS_NO_MALLOC");

    builder.compile("lfs-sys");

    let bindings = bindgen::Builder::default()
        .header("littlefs/lfs.h")
        .clang_arg(format!("--target={}", target))
        .use_core()
        .ctypes_prefix("cty")
        .allowlist_item("lfs_.*")
        .allowlist_item("LFS_.*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}
