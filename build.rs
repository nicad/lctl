use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let man_src = manifest_dir.join("man/lctl.1");
    println!("cargo:rerun-if-changed={}", man_src.display());

    // `cargo install` builds release by default; skip debug/test builds.
    if env::var("PROFILE").ok().as_deref() != Some("release") {
        return;
    }

    if !man_src.is_file() {
        println!("cargo:warning=man page missing at {}; skipping install", man_src.display());
        return;
    }

    let Some(man_dir) = install_man_dir() else {
        return;
    };

    if let Err(e) = fs::create_dir_all(&man_dir) {
        println!("cargo:warning=could not create {}: {e}", man_dir.display());
        return;
    }

    let man_dst = man_dir.join("lctl.1");
    if let Err(e) = fs::copy(&man_src, &man_dst) {
        println!("cargo:warning=could not install man page to {}: {e}", man_dst.display());
    }
}

/// `$CARGO_INSTALL_ROOT` / `$CARGO_HOME` / `$HOME/.cargo` → `share/man/man1`
fn install_man_dir() -> Option<PathBuf> {
    let root = env::var_os("CARGO_INSTALL_ROOT")
        .map(PathBuf::from)
        .or_else(|| env::var_os("CARGO_HOME").map(PathBuf::from))
        .or_else(|| env::var_os("HOME").map(|h| Path::new(&h).join(".cargo")))?;
    Some(root.join("share/man/man1"))
}
