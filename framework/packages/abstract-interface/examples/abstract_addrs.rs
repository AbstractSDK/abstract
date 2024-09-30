use abstract_std::{
    native_addrs::TEST_ABSTRACT_CREATOR, ANS_HOST, IBC_CLIENT, IBC_HOST, MODULE_FACTORY, REGISTRY,
};
use cosmwasm_std::{instantiate2_address, CanonicalAddr};
use cw_blob::interface::CwBlob;
use cw_orch::prelude::*;

/// Print abstract addresses
fn main() -> Result<(), CwOrchError> {
    let creator = CanonicalAddr::from(TEST_ABSTRACT_CREATOR);
    let blob_checksum = CwBlob::<Daemon>::checksum();

    let ans_addr = instantiate2_address(blob_checksum.as_slice(), &creator, ANS_HOST.as_bytes())?;
    let registry_addr =
        instantiate2_address(blob_checksum.as_slice(), &creator, REGISTRY.as_bytes())?;
    let module_factory_addr = instantiate2_address(
        blob_checksum.as_slice(),
        &creator,
        MODULE_FACTORY.as_bytes(),
    )?;
    let ibc_client_addr =
        instantiate2_address(blob_checksum.as_slice(), &creator, IBC_CLIENT.as_bytes())?;
    let ibc_host_addr =
        instantiate2_address(blob_checksum.as_slice(), &creator, IBC_HOST.as_bytes())?;
    // Put output in `abstract-std/src/native_addrs.rs`
    // If someone wants to over-engineer it you can replace it with actual rust tokens with quote!()
    println!("Abstract Addresses:");
    println!(
        "pub const ANS_ADDR: [u8;{}] = {:?};",
        ans_addr.len(),
        ans_addr.as_slice()
    );
    println!(
        "pub const REGISTRY_ADDR: [u8;{}] = {:?};",
        registry_addr.len(),
        registry_addr.as_slice()
    );
    println!(
        "pub const MODULE_FACTORY_ADDR: [u8;{}] = {:?};",
        module_factory_addr.len(),
        module_factory_addr.as_slice()
    );
    println!(
        "pub const IBC_CLIENT_ADDR: [u8;{}] = {:?};",
        ibc_client_addr.len(),
        ibc_client_addr.as_slice()
    );
    println!(
        "pub const IBC_HOST_ADDR: [u8;{}] = {:?};",
        ibc_host_addr.len(),
        ibc_host_addr.as_slice()
    );
    Ok(())
}
