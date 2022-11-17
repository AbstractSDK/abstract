//! # Ibc Client
//! The IbcClient object provides helper function for ibc-related queries or actions.
//!

use abstract_os::{
    ibc_client::{CallbackInfo, ExecuteMsg as IbcClientMsg},
    ibc_host::HostAction,
    proxy::ExecuteMsg,
};
use cosmwasm_std::{to_binary, Coin, CosmosMsg, Deps, StdError, WasmMsg};

use super::Identification;

/// Interact with other chains over IBC.
pub trait IbcInterface: Identification {
    fn ibc_client<'a>(&'a self, deps: Deps<'a>) -> IbcClient<Self> {
        IbcClient { base: self, deps }
    }
}

impl<T> IbcInterface for T where T: Identification {}

#[derive(Clone)]
pub struct IbcClient<'a, T: IbcInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: IbcInterface> IbcClient<'a, T> {
    /// Call a [`HostAction`] on the host of the provided `host_chain`.
    pub fn host_action(
        &self,
        host_chain: String,
        action: HostAction,
        callback: Option<CallbackInfo>,
        retries: u8,
    ) -> Result<CosmosMsg, StdError> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.base.proxy_address(self.deps)?.to_string(),
            msg: to_binary(&ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::SendPacket {
                    host_chain,
                    action,
                    callback_info: callback,
                    retries,
                }],
            })?,
            funds: vec![],
        }))
    }
    /// IbcClient the provided coins from the OS to its proxy on the `receiving_chain`.
    pub fn ics20_transfer(
        &self,
        receiving_chain: String,
        funds: Vec<Coin>,
    ) -> Result<CosmosMsg, StdError> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.base.proxy_address(self.deps)?.to_string(),
            msg: to_binary(&ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::SendFunds {
                    host_chain: receiving_chain,
                    funds,
                }],
            })?,
            funds: vec![],
        }))
    }
}
