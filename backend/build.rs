use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=web.config.json");
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("target"));
    let out_dir = target_dir.join(&profile);
    let src = manifest_dir.join("web.config.json");
    if !src.exists() {
        return;
    }
    if let Err(e) = fs::create_dir_all(&out_dir) {
        println!(
            "cargo:warning=failed to create output dir {}: {}",
            out_dir.display(),
            e
        );
        return;
    }
    let dest = out_dir.join("web.config.json");
    if let Err(e) = fs::copy(&src, &dest) {
        println!(
            "cargo:warning=failed to copy {} to {}: {}",
            src.display(),
            dest.display(),
            e
        );
    } else {
        println!(
            "cargo:warning=copied {} -> {}",
            src.display(),
            dest.display()
        );
    }
}
