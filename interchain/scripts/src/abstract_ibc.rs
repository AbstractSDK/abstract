use abstract_core::ibc_client::{ExecuteMsgFns as _, QueryMsgFns};
use abstract_core::ibc_host::ExecuteMsgFns;
use abstract_core::objects::chain_name::ChainName;
use abstract_interface::{Abstract, AccountFactoryExecFns};
use cw_orch::daemon::{ChainInfo, ChainRegistryData};
use cw_orch::interchain::InterchainError;
use cw_orch::prelude::*;
use cw_orch_polytone::Polytone;
use tokio::runtime::Handle;

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

pub fn get_polytone_deployment_id(src_chain: &ChainInfo, dst_chain: &ChainInfo) -> String {
    format!("{}-->{}", src_chain.chain_id, dst_chain.chain_id)
}
pub fn has_polytone_connection(
    src_chain: ChainInfo,
    dst_chain: ChainInfo,
    rt: &Handle,
) -> anyhow::Result<bool> {
    // We just need to verify if the polytone deployment crate has the contracts in it
    let deployment_id = get_polytone_deployment_id(&src_chain, &dst_chain);
    let src_daemon = Daemon::builder()
        .handle(rt)
        .chain(src_chain)
        .deployment_id(deployment_id.clone())
        .build()?;
    let dst_daemon = Daemon::builder()
        .handle(rt)
        .chain(dst_chain)
        .deployment_id(deployment_id)
        .build()?;

    let src_polytone = Polytone::load_from(src_daemon)?;
    let dst_polytone = Polytone::load_from(dst_daemon)?;

    Ok(src_polytone.note.address().is_ok() && dst_polytone.voice.address().is_ok())
}
