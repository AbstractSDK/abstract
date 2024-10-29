use std::{env, fs, path::Path};

use base64::prelude::*;

const DEFAULT_ABSTRACT_CREATOR: [u8; 33] = [
    2, 146, 187, 207, 156, 96, 230, 188, 163, 167, 152, 64, 234, 101, 130, 38, 50, 89, 139, 233,
    56, 192, 110, 242, 251, 222, 103, 198, 68, 80, 201, 159, 3,
];

fn main() {
    let creator = if let Ok(creator) = env::var("ABSTRACT_CREATOR") {
        BASE64_STANDARD
            .decode(creator)
            .expect("ABSTRACT_CREATOR public key supposed to be encoded as base64")
    } else {
        DEFAULT_ABSTRACT_CREATOR.to_vec()
    };

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("creator");
    fs::write(dest_path, creator).unwrap();

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=ABSTRACT_CREATOR")
}
