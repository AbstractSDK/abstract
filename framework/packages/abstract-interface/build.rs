use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use base64::prelude::*;

const DEFAULT_ABSTRACT_CREATOR: [u8; 33] = [
    2, 146, 187, 207, 156, 96, 230, 188, 163, 167, 152, 64, 234, 101, 130, 38, 50, 89, 139, 233,
    56, 192, 110, 242, 251, 222, 103, 198, 68, 80, 201, 159, 3,
];

fn main() {
    // We don't need build script for wasm
    if env::var("TARGET").unwrap().contains("wasm") {
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
        fs::metadata(state_path.clone()).is_ok(),
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
    let read_dir = fs::read_dir(artifacts_path).expect("You should create an artifacts dir in your crate to export the wasm files along with the cw-orch library");

    // Path to OUT_DIR
    let out_dir = env::var("OUT_DIR").unwrap();

    let creator = if let Ok(creator) = env::var("ABSTRACT_CREATOR") {
        let public_key = bip32::Mnemonic::new(&creator, Default::default())
            .map(|phrase| {
                let seed = phrase.to_seed("");
                let derive_path: bip32::DerivationPath = "m/44'/118'/0'/0/0".parse().unwrap();
                let xprv = bip32::XPrv::derive_from_path(seed, &derive_path).unwrap();
                xprv.public_key().to_bytes().to_vec()
            })
            .ok()
            .or(BASE64_STANDARD.decode(&creator).ok())
            .expect("ABSTRACT_CREATOR public key supposed to be encoded as base64 or seed phrase");
        // We can't edit len of the creator blob, *unless we somehow find all references to it
        assert!(
            public_key.len() == DEFAULT_ABSTRACT_CREATOR.len(),
            "Pubkey length for abstract creator should be {}",
            DEFAULT_ABSTRACT_CREATOR.len()
        );
        Some(public_key)
    } else {
        None
    };

    for entry in read_dir {
        let entry = entry.unwrap();
        let mut file_content = fs::read(entry.path()).unwrap();
        if entry.path().extension().and_then(OsStr::to_str) == Some("wasm") {
            // Edit wasms if we have custom abstract creator
            if let Some(creator) = creator.as_deref() {
                if let Some(position) = file_content
                    .windows(DEFAULT_ABSTRACT_CREATOR.len())
                    .position(|window| window == DEFAULT_ABSTRACT_CREATOR)
                {
                    file_content[position..position + DEFAULT_ABSTRACT_CREATOR.len()]
                        .copy_from_slice(&creator);
                }
            }
            // write content
            fs::write(Path::new(&out_dir).join(entry.file_name()), file_content).unwrap();
        }
    }

    println!("cargo::rerun-if-changed=build.rs");
    // TODO: rerun if changed any of the wasm files in artifacts/, is it wildcard-able?
    println!("cargo::rerun-if-env-changed=ABSTRACT_CREATOR")
    // println!("cargo:rerun-if-changed={}", absolute_state_path.display());
}
