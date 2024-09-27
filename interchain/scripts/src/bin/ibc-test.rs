use abstract_client::AbstractClient;

use cosmwasm_std::CosmosMsg;
use cw_orch::daemon::networks::{PION_1, XION_TESTNET_1};
use cw_orch::daemon::DaemonState;
use cw_orch::environment::ChainState;
use cw_orch::prelude::*;
use cw_orch::tokio::runtime::Handle;

use cw_orch_interchain::prelude::*;
use tokio::runtime::Runtime;

/// Connect IBC between two chains.
/// @TODO update this to take in the networks as arguments.
fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let chains = vec![
        (PION_1, None),
        (XION_TESTNET_1, None),
        // (OSMOSIS_1, Some(std::env::var("OSMOSIS_MNEMONIC")?)),
    ];
    let runtime = Runtime::new()?;

    let src_chain = &chains[0];
    let dst_chain = &chains[1];

    test_ibc(src_chain.clone(), dst_chain.clone(), runtime.handle())?;

    Ok(())
}

fn get_daemon(
    chain: ChainInfo,
    handle: &Handle,
    mnemonic: Option<String>,
    state: Option<DaemonState>,
    deployment_id: Option<String>,
) -> cw_orch::anyhow::Result<Daemon> {
    let mut builder = DaemonBuilder::new(chain);
    builder.handle(handle);
    if let Some(mnemonic) = mnemonic {
        builder.mnemonic(mnemonic);
    }
    if let Some(deployment_id) = deployment_id {
        builder.deployment_id(deployment_id);
    }
    if let Some(state) = state {
        builder.state(state);
    }
    Ok(builder.build()?)
}

pub fn get_deployment_id(src_chain: &ChainInfo, dst_chain: &ChainInfo) -> String {
    format!("{}-->{}", src_chain.chain_id, dst_chain.chain_id)
}

fn test_ibc(
    (src_chain, src_mnemonic): (ChainInfo, Option<String>),
    (dst_chain, dst_mnemonic): (ChainInfo, Option<String>),
    handle: &Handle,
) -> cw_orch::anyhow::Result<()> {
    let src_daemon = get_daemon(src_chain.clone(), handle, src_mnemonic.clone(), None, None)?;
    let dst_daemon = get_daemon(
        dst_chain.clone(),
        handle,
        dst_mnemonic,
        Some(src_daemon.state()),
        None,
    )?;

    let interchain = DaemonInterchain::from_daemons(
        vec![src_daemon.clone(), dst_daemon.clone()],
        &ChannelCreationValidator,
    );

    let src_abstract = AbstractClient::new(src_daemon)?;
    let dst_abstract = AbstractClient::new(dst_daemon)?;

    let account = src_abstract.account_builder().build()?;

    account.set_ibc_status(true)?;

    let remote_account = account
        .remote_account_builder(interchain, &dst_abstract)
        .build()?;

    let msgs: Vec<CosmosMsg> = vec![];

    remote_account.execute(msgs)?;

    Ok(())
}
