use std::path::PathBuf;

fn main() {
    // This is where the custom state comes from, not possible to change that for now
    let state_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("daemon_state.json")
        .display()
        .to_string();
    // This is where the compiled wasm come from, not possible to change that for now
    let artifacts_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .display()
        .to_string();

    // First we load the daemon json file
    // We verify that the daemon_file is actually present where it should be located
    assert!(
        std::fs::metadata(state_path.clone()).is_ok(),
        "File should be present at {}",
        state_path
    );

    // Now, we output the json file so that it can be used in the daemon state. We want this load to be non-null when exporting the package

    // This is useless for now, should we include that automatically ?
    // // This will be loaded from scripts out of the manifest dir
    // let absolute_state_path = PathBuf::from(CRATE_PATH).join(state_path);
    // fs::write(
    //     dest_path,
    //     format!(
    //         "
    //     use cw_orch::prelude::CwEnv;
    //     pub fn custom_state<T: CwEnv>(chain: &mut T){{
    //         chain.custom_state_file(\"{}\".to_string())
    //     }}",
    //         absolute_state_path.display()
    //     ),
    // )
    // .unwrap();

    // We also verify that the local artifacts fir exists
    assert!(std::fs::metadata(artifacts_path).is_ok(), "You should create an artifacts dir in your crate to export the wasm files along with the cw-orch library");

    println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rerun-if-changed={}", absolute_state_path.display());
}
