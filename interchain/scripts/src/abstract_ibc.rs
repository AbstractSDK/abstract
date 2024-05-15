use abstract_interface::{Abstract, AccountFactoryExecFns};
use abstract_std::ibc_client::{ExecuteMsgFns as _, QueryMsgFns as _};
use abstract_std::ibc_host::{ExecuteMsgFns, QueryMsgFns};
use abstract_std::objects::chain_name::ChainName;
use anyhow::anyhow;
use cw_orch::interchain::InterchainError;
use cw_orch::prelude::*;
use cw_orch_polytone::Polytone;
use polytone_note::msg::QueryMsgFns as _;
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
        chain2_name.clone(),
        dest.ibc.host.address()?.to_string(),
        polytone_src.note.address()?.to_string(),
    )?;
    // We make sure the IBC execution is done so that the proxy address is saved inside the Abstract contract
    interchain.wait_ibc(&chain1_id, proxy_tx_result).unwrap();

    // Finally, we get the proxy address and register the proxy with the ibc host for the dest chain
    let proxy_address = abstr.ibc.client.host(chain2_name)?;

    dest.ibc
        .host
        .register_chain_proxy(chain1_name, proxy_address.remote_polytone_proxy.unwrap())?;

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
pub fn verify_polytone_connection(
    src_chain: ChainInfo,
    dst_chain: ChainInfo,
    rt: &Handle,
) -> anyhow::Result<()> {
    // We just need to verify if the polytone deployment crate has the contracts in it
    let deployment_id = get_polytone_deployment_id(&src_chain, &dst_chain);
    let src_daemon = Daemon::builder()
        .handle(rt)
        .chain(src_chain)
        .deployment_id(deployment_id.clone())
        .build()?;

    let src_polytone = Polytone::load_from(src_daemon)?;

    src_polytone.note.active_channel()?.ok_or(anyhow!(
        "No channel found on polytone source between the chains",
    ))?;

    Ok(())
}

pub fn verify_abstract_ibc(
    src_chain: ChainInfo,
    dst_chain: ChainInfo,
    rt: &Handle,
) -> anyhow::Result<()> {
    let src_daemon = Daemon::builder()
        .handle(rt)
        .chain(src_chain.clone())
        .build()?;
    let dst_daemon = Daemon::builder()
        .handle(rt)
        .chain(dst_chain.clone())
        .build()?;

    let src_abstract = Abstract::load_from(src_daemon.clone())?;
    let dst_abstract = Abstract::load_from(dst_daemon.clone())?;

    // We make sure the client has a registered host
    let host = src_abstract
        .ibc
        .client
        .host(ChainName::from_chain_id(dst_chain.chain_id))?;

    // We verify the host matches dst chain host for this original chain
    if host.remote_polytone_proxy.is_none() {
        anyhow::bail!(
            "Remote connection still not established. Connection creation has still not returned"
        )
    }
    if host.remote_host != dst_abstract.ibc.host.address()? {
        anyhow::bail!("Wrong host address on the src chain")
    }

    let proxy = dst_abstract
        .ibc
        .host
        .client_proxy(ChainName::from_chain_id(src_chain.chain_id).into_string())?;

    if host.remote_polytone_proxy.unwrap() != proxy.proxy {
        anyhow::bail!("Wrong proxy address registered on the dst chain")
    }

    Ok(())
}

pub fn has_abstract_ibc(src_chain: ChainInfo, dst_chain: ChainInfo, rt: &Handle) -> bool {
    verify_abstract_ibc(src_chain, dst_chain, rt).is_ok()
}

pub fn has_polytone_connection(src_chain: ChainInfo, dst_chain: ChainInfo, rt: &Handle) -> bool {
    verify_polytone_connection(src_chain, dst_chain, rt).is_ok()
}
