use log::{info, warn};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process;

fn run_git(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<(), String> {
    let stdout = process::Stdio::inherit();

    let exit_status = process::Command::new("git")
        .args(args)
        .stdout(stdout)
        .status()
        .expect("git exit status missing");

    if !exit_status.success() {
        return Err(format!(
            "git exited with error code: {:?}",
            exit_status.code()
        ));
    }

    Ok(())
}

pub fn update_submodule(dir: &str, rev: &str) {
    let full_path = |p: &str| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(p);

    info!("Updating {} submodule...", dir);

    // switch to the given revision
    run_git(["-C", full_path(dir).to_str().unwrap(), "checkout", rev]).expect("failed to checkout");

    // pull the latest changes
    match run_git(["-C", full_path(dir).to_str().unwrap(), "pull"]) {
        Ok(_) => info!("Updated {} submodule to revision {}", dir, rev),
        Err(_) => warn!(
            "Failed to update {} with revision {}. This might be caused by revision is a tag",
            dir, rev
        ),
    };

    // run_git(["submodule", "update", "--init"]).expect("failed to update submodules");
}
