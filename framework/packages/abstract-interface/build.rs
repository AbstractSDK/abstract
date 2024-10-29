use std::{
    ffi::OsStr,
    fs::{Metadata, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use base64::prelude::*;

const DEFAULT_ABSTRACT_CREATOR: [u8; 33] = [
    2, 146, 187, 207, 156, 96, 230, 188, 163, 167, 152, 64, 234, 101, 130, 38, 50, 89, 139, 233,
    56, 192, 110, 242, 251, 222, 103, 198, 68, 80, 201, 159, 3,
];

fn main() {
    // We don't need build script for wasm
    if std::env::var("TARGET").unwrap().contains("wasm") {
        return;
    }

    // This is where the custom state comes from, not possible to change that for now
    let state_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("state.json")
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

    // We also verify that the local artifacts dir exists
    assert!(std::fs::metadata(artifacts_path.clone()).is_ok(), "You should create an artifacts dir in your crate to export the wasm files along with the cw-orch library");

    if let Ok(creator) = std::env::var("ABSTRACT_CREATOR") {
        let creator = BASE64_STANDARD
            .decode(creator)
            .expect("ABSTRACT_CREATOR public key supposed to be encoded as base64");
        assert!(
            creator.len() == DEFAULT_ABSTRACT_CREATOR.len(),
            "Pubkey for abstract creator should be 33"
        );

        for entry in std::fs::read_dir(artifacts_path).unwrap() {
            let entry = entry.unwrap();
            if entry.path().extension().and_then(OsStr::to_str) == Some("wasm") {
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(false)
                    .open(entry.path())
                    .unwrap();

                let mut buf = Vec::new();
                file.read_to_end(&mut buf).unwrap();
                if let Some(position) = buf
                    .windows(DEFAULT_ABSTRACT_CREATOR.len())
                    .position(|window| window == DEFAULT_ABSTRACT_CREATOR)
                {
                    file.seek(SeekFrom::Start(position as u64)).unwrap();
                    file.write_all(&buf).unwrap();
                }
            }
        }
    };

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=ABSTRACT_CREATOR")
    // println!("cargo:rerun-if-changed={}", absolute_state_path.display());
}
