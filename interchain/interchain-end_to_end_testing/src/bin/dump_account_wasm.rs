use abstract_interface::AccountI;
use cosmwasm_std::Binary;
use cw_orch::daemon::networks::XION_TESTNET_1;
use cw_orch::prelude::*;
use flate2::{write, Compression};
use std::io::Write;

pub fn main() -> cw_orch::anyhow::Result<()> {
    let wasm_path = AccountI::<Daemon>::wasm(&XION_TESTNET_1.into());

    let file_contents = std::fs::read(wasm_path.path())?;
    let mut e = write::GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(&file_contents)?;
    let wasm_byte_code = e.finish()?;
    let binary = Binary::from(wasm_byte_code);
    println!("{}", binary.to_base64());

    Ok(())
}
