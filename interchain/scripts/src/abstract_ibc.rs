use abstract_core::ibc_client::{ExecuteMsgFns as _, QueryMsgFns};
use abstract_core::ibc_host::ExecuteMsgFns;
use abstract_core::objects::chain_name::ChainName;
use abstract_interface::{Abstract, AccountFactoryExecFns};
use cw_orch::interchain::InterchainError;
use cw_orch::prelude::*;
use cw_orch_polytone::Polytone;

/// This is only used for testing and shouldn't be used in production
pub fn abstract_ibc_connection_with<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
    abstr: &Abstract<Chain>,
    interchain: &IBC,
    dest: &Abstract<Chain>,
    polytone_src: &Polytone<Chain>,
) -> Result<(), InterchainError> {
    // First we register client and host respectively
    let chain1_id = abstr.ibc.client.get_chain().chain_id();
    let chain1_name = ChainName::from_chain_id(&chain1_id);

    let chain2_id = dest.ibc.client.get_chain().chain_id();
    let chain2_name = ChainName::from_chain_id(&chain2_id);

    // First, we register the host with the client.
    // We register the polytone note with it because they are linked
    // This triggers an IBC message that is used to get back the proxy address
    let proxy_tx_result = abstr.ibc.client.register_infrastructure(
        chain2_name.to_string(),
        dest.ibc.host.address()?.to_string(),
        polytone_src.note.address()?.to_string(),
    )?;
    // We make sure the IBC execution is done so that the proxy address is saved inside the Abstract contract
    interchain.wait_ibc(&chain1_id, proxy_tx_result).unwrap();

    // Finally, we get the proxy address and register the proxy with the ibc host for the dest chain
    let proxy_address = abstr.ibc.client.host(chain2_name.to_string())?;

    dest.ibc.host.register_chain_proxy(
        chain1_name.to_string(),
        proxy_address.remote_polytone_proxy.unwrap(),
    )?;

    dest.account_factory.update_config(
        None,
        Some(dest.ibc.host.address()?.to_string()),
        None,
        None,
    )?;

    Ok(())
}
