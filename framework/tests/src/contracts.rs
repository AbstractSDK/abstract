use cw_orch::prelude::*;

use cosmwasm_std::Empty;
use cw_orch::interface;

#[interface(
    abstract_etf::msg::InstantiateMsg,
    abstract_etf::msg::ExecuteMsg,
    abstract_etf::msg::QueryMsg,
    Empty
)]
pub struct AbstractETF;

impl<Chain: CwEnv> Uploadable for AbstractETF<Chain> {
    fn wasm(&self) -> WasmPath {
        WasmPath::new("artifacts/abstract_etf.wasm").unwrap()
    }
}

#[interface(
    cw20_base::msg::InstantiateMsg,
    cw20_base::msg::ExecuteMsg,
    cw20_base::msg::QueryMsg,
    Empty
)]
pub struct Cw20Base;

impl<Chain: CwEnv> Uploadable for Cw20Base<Chain> {
    fn wasm(&self) -> WasmPath {
        WasmPath::new("artifacts/cw20_base.wasm").unwrap()
    }
}
