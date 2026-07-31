use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");
    let version_path = root.join("VERSION");
    let build_path = root.join("BUILD");

    println!("cargo:rerun-if-changed={}", version_path.display());
    println!("cargo:rerun-if-changed={}", build_path.display());

    let version = fs::read_to_string(&version_path)
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|_| env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".into()));
    let build = fs::read_to_string(&build_path)
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|_| "0".into());

    println!("cargo:rustc-env=RESIDIUUM_VERSION={version}");
    println!("cargo:rustc-env=RESIDIUUM_BUILD={build}");
}
