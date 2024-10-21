use abstract_interface::Abstract;
use abstract_std::ibc_client::QueryMsgFns as _;
use abstract_std::ibc_host::QueryMsgFns;
use abstract_std::objects::TruncatedChainId;
use anyhow::anyhow;
use cw_orch::prelude::*;
use cw_orch_polytone::Polytone;
use polytone_note::msg::QueryMsgFns as _;
use tokio::runtime::Handle;

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
    let src_daemon = Daemon::builder(src_chain)
        .handle(rt)
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
    let src_daemon = Daemon::builder(src_chain.clone()).handle(rt).build()?;
    let dst_daemon = Daemon::builder(dst_chain.clone())
        .state(src_daemon.state())
        .handle(rt)
        .build()?;

    let src_abstract = Abstract::load_from(src_daemon.clone())?;
    let dst_abstract = Abstract::load_from(dst_daemon.clone())?;

    // We make sure the client has a registered host
    let host = src_abstract
        .ibc
        .client
        .host(TruncatedChainId::from_chain_id(dst_chain.chain_id))?;

    // We verify the host matches dst chain host for this original chain
    if host.remote_polytone_proxy.is_none() {
        anyhow::bail!(
            "Remote connection still not established. Connection creation has still not returned"
        )
    }
    if host.remote_host != dst_abstract.ibc.host.address()?.as_str() {
        anyhow::bail!("Wrong host address on the src chain")
    }

    let proxy = dst_abstract
        .ibc
        .host
        .client_proxy(TruncatedChainId::from_chain_id(src_chain.chain_id).into_string())?;

    if host.remote_polytone_proxy.unwrap() != proxy.proxy.as_str() {
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
