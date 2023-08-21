use cw_orch::{interchain::interchain_channel::InterchainChannel, interface, prelude::*};
// This file contains all interfaces to the polytone contracts

#[interface(
    polytone_note::msg::InstantiateMsg,
    polytone_note::msg::ExecuteMsg,
    polytone_note::msg::QueryMsg,
    polytone_note::msg::MigrateMsg
)]
pub struct Note;

impl<Chain: CwEnv> Uploadable for Note<Chain> {
    // No direct integration because this is an IBC protocol (so not compatible with cw-multi-test)
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("polytone_note")
            .unwrap()
    }
}

#[interface(
    polytone_proxy::msg::InstantiateMsg,
    polytone_proxy::msg::ExecuteMsg,
    polytone_proxy::msg::QueryMsg,
    Empty
)]
pub struct Proxy;

impl<Chain: CwEnv> Uploadable for Proxy<Chain> {
    // No direct integration because this is an IBC protocol (so not compatible with cw-multi-test)
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("polytone_proxy")
            .unwrap()
    }
}

#[interface(
    polytone_voice::msg::InstantiateMsg,
    polytone_voice::msg::ExecuteMsg,
    polytone_voice::msg::QueryMsg,
    polytone_voice::msg::MigrateMsg
)]
pub struct Voice;

impl<Chain: CwEnv> Uploadable for Voice<Chain> {
    // No direct integration because this is an IBC protocol (so not compatible with cw-multi-test)
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("polytone_voice")
            .unwrap()
    }
}

pub struct Polytone<Chain: CwEnv> {
    pub note: Note<Chain>,
    pub voice: Voice<Chain>,
    pub channel: InterchainChannel,
}
