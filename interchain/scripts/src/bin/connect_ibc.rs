use abstract_interface::Abstract;
use abstract_scripts::abstract_ibc::abstract_ibc_connection_with;
use cw_orch::daemon::networks::neutron::NEUTRON_NETWORK;
use cw_orch::daemon::networks::{ARCHWAY_1, JUNO_1, OSMOSIS_1, PHOENIX_1};
use cw_orch::environment::ChainKind;
use cw_orch::prelude::*;
use cw_orch::tokio::runtime::Handle;
use cw_orch_polytone::Polytone;
use tokio::runtime::Runtime;

/// <https://github.com/cosmos/chain-registry/blob/master/neutron/chain.json>
pub const NEUTRON_1: ChainInfo = ChainInfo {
    kind: ChainKind::Mainnet,
    chain_id: "neutron-1",
    gas_denom: "untrn",
    gas_price: 0.075,
    grpc_urls: &["http://grpc-kralum.neutron-1.neutron.org:80"],
    network_info: NEUTRON_NETWORK,
    lcd_url: Some("https://rest-kralum.neutron-1.neutron.org"),
    fcd_url: None,
};

fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let chains = vec![
        (JUNO_1, None),
        (OSMOSIS_1, Some(std::env::var("OSMOSIS_MNEMONIC")?)),
    ];
    let runtime = Runtime::new()?;

    let src_chain = &chains[1];
    let dst_chain = &chains[0];

    connect(src_chain.clone(), dst_chain.clone(), runtime.handle())?;

    Ok(())
}

fn get_daemon(
    chain: ChainInfo,
    handle: &Handle,
    mnemonic: Option<String>,
    deployment_id: Option<String>,
) -> cw_orch::anyhow::Result<Daemon> {
    let mut builder = DaemonBuilder::default();
    builder.chain(chain).handle(handle);
    if let Some(mnemonic) = mnemonic {
        builder.mnemonic(mnemonic);
    }
    if let Some(deployment_id) = deployment_id {
        builder.deployment_id(deployment_id);
    }
    Ok(builder.build()?)
}

pub fn get_deployment_id(src_chain: &ChainInfo, dst_chain: &ChainInfo) -> String {
    format!("{}-->{}", src_chain.chain_id, dst_chain.chain_id)
}

fn connect(
    (src_chain, src_mnemonic): (ChainInfo, Option<String>),
    (dst_chain, dst_mnemonic): (ChainInfo, Option<String>),
    handle: &Handle,
) -> cw_orch::anyhow::Result<()> {
    let src_daemon = get_daemon(src_chain.clone(), handle, src_mnemonic.clone(), None)?;
    let dst_daemon = get_daemon(dst_chain.clone(), handle, dst_mnemonic, None)?;

    let src_abstract = Abstract::load_from(src_daemon.clone())?;
    let dst_abstract = Abstract::load_from(dst_daemon.clone())?;

    let src_polytone_daemon = get_daemon(
        src_chain.clone(),
        handle,
        src_mnemonic,
        Some(get_deployment_id(&src_chain, &dst_chain)),
    )?;

    let src_polytone = Polytone::load_from(src_polytone_daemon)?;

    let interchain = DaemonInterchainEnv::from_daemons(
        handle,
        vec![src_daemon, dst_daemon],
        &ChannelCreationValidator,
    );

    abstract_ibc_connection_with(&src_abstract, &interchain, &dst_abstract, &src_polytone)?;

    Ok(())
}
