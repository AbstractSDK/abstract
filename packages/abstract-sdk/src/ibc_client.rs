use abstract_os::{
    ibc_client::{CallbackInfo, ExecuteMsg as IbcClientMsg},
    ibc_host::HostAction,
};
use cosmwasm_std::{Addr, Coin, StdError};

use crate::{proxy::os_ibc_action, OsAction};

/// Call a [`HostAction`] on the host of the provided `host_chain`.
pub fn host_ibc_action(
    proxy_address: &Addr,
    host_chain: String,
    action: HostAction,
    callback: Option<CallbackInfo>,
    retries: u8,
) -> Result<OsAction, StdError> {
    os_ibc_action(
        vec![IbcClientMsg::SendPacket {
            host_chain,
            action,
            callback_info: callback,
            retries,
        }],
        proxy_address,
    )
}
/// Transfer the provided coins from the OS to it's proxy on the `receiving_chain`.
pub fn ics20_transfer(
    proxy_address: &Addr,
    receiving_chain: String,
    funds: Vec<Coin>,
) -> Result<OsAction, StdError> {
    os_ibc_action(
        vec![IbcClientMsg::SendFunds {
            host_chain: receiving_chain,
            funds,
        }],
        proxy_address,
    )
}
