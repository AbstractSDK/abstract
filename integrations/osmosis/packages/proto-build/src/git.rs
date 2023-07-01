use log::info;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process;

fn run_git(args: impl IntoIterator<Item = impl AsRef<OsStr>>) {
    let stdout = process::Stdio::inherit();

    let exit_status = process::Command::new("git")
        .args(args)
        .stdout(stdout)
        .status()
        .expect("git exit status missing");

    if !exit_status.success() {
        panic!("git exited with error code: {:?}", exit_status.code());
    }
}

pub fn update_submodule(dir: &str, rev: &str) {
    let full_path = |p: &str| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(p);

    info!("Updating {} submodule...", dir);
    run_git(["submodule", "update", "--init"]);
    run_git(["-C", full_path(dir).to_str().unwrap(), "fetch"]);
    run_git([
        "-C",
        full_path(dir).to_str().unwrap(),
        "reset",
        "--hard",
        rev,
    ]);
}
