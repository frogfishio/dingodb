use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));
    let build_path = manifest_dir.join("BUILD");

    println!("cargo:rerun-if-changed={}", build_path.display());

    let version = env::var("CARGO_PKG_VERSION").expect("missing CARGO_PKG_VERSION");
    let build = fs::read_to_string(&build_path)
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|_| "0".to_string());

    println!("cargo:rustc-env=SDA_VERSION={version}");
    println!("cargo:rustc-env=SDA_BUILD={build}");
}