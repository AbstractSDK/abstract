use abstract_os::cw_staking::{ExecuteMsg, InstantiateMsg, QueryMsg, CW_STAKING};
use boot_core::prelude::boot_contract;
use boot_core::{BootEnvironment, Contract, IndexResponse, TxResponse};
use cosmwasm_std::{Addr, Empty};

/// Contract wrapper for interacting with BOOT
#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct CwStakingApi<Chain>;

/// implement chain-generic functions
impl<Chain: BootEnvironment> CwStakingApi<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(id: &str, chain: Chain) -> Self {
        Self(Contract::new(id, chain).with_wasm_path("cw_staking"))
    }

    pub fn load(chain: Chain, addr: &Addr) -> Self {
        Self(Contract::new(CW_STAKING, chain).with_address(Some(addr)))
    }
}
