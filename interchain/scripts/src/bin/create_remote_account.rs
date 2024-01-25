use abstract_client::AbstractClient;
use abstract_core::ibc_host::HostAction;
use abstract_core::objects::namespace::Namespace;
use abstract_interface::Abstract;
use abstract_scripts::abstract_ibc::abstract_ibc_connection_with;
use cw_orch::daemon::networks::neutron::NEUTRON_NETWORK;
use cw_orch::daemon::networks::{ARCHWAY_1, JUNO_1, OSMOSIS_1, PHOENIX_1};
use cw_orch::daemon::ChainKind;
use cw_orch::prelude::*;
use cw_orch::{
    daemon::{ChainInfo, Daemon},
    deploy::Deploy,
    tokio::runtime::Handle,
};
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
        (PHOENIX_1, None),
        (ARCHWAY_1, None),
        (NEUTRON_1, None),
        (OSMOSIS_1, Some(std::env::var("OSMOSIS_MNEMONIC")?)),
    ];
    let runtime = Runtime::new()?;

    // for src_chain in &chains {
    //     for dst_chain in &chains {
    //         if src_chain.0.chain_id != dst_chain.0.chain_id {
    //             connect(src_chain.clone(), dst_chain.clone(), runtime.handle())?;
    //         }
    //     }
    // }
    let chain0 = &chains[1];
    let chain1 = &chains[4];

    connect(chain0.clone(), chain1.clone(), runtime.handle())?;
    // connect(chain1.clone(), chain0.clone(), runtime.handle())?;

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

    let interchain = DaemonInterchainEnv::from_daemons(
        handle,
        vec![src_daemon.clone(), dst_daemon],
        &ChannelCreationValidator,
    );
    let client = AbstractClient::new(src_daemon)?;
    let account = client
        .account_builder()
        .namespace(Namespace::new("abstract")?)
        .fetch_if_namespace_claimed(true)
        .build()?;

    let tx_response = account.create_ibc_account(dst_chain.network_info.id, None, None, vec![])?;

    // We make sure the IBC execution is done so that the proxy address is saved inside the Abstract contract
    interchain
        .wait_ibc(&src_chain.chain_id.to_string(), tx_response)
        .unwrap();

    Ok(())
}
