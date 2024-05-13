use abstract_interface::Abstract;
use abstract_scripts::abstract_ibc::get_polytone_deployment_id;
use abstract_std::ibc_client::state::IbcInfrastructure;
use abstract_std::ibc_client::QueryMsgFns;
use abstract_std::objects::chain_name::ChainName;
use cosmos_sdk_proto::ibc::core::channel::v1::{Channel, State};
use cosmos_sdk_proto::ibc::core::connection;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure_eq, StdError};
use cw_orch::daemon::queriers::Ibc;
use cw_orch::daemon::Daemon;
use cw_orch::environment::{ChainInfo, ChainInfoOwned, ChainKind, ChainState};
use cw_orch::prelude::QuerierGetter;

use cw_orch::{
    contract::Deploy,
    daemon::{networks::SUPPORTED_NETWORKS, DaemonBuilder},
};

use cw_orch_polytone::Polytone;
use polytone_note::msg::QueryMsgFns as _;

fn get_corresponding_chain(chain_name: &ChainName, kind: &ChainKind) -> ChainInfo {
    SUPPORTED_NETWORKS
        .iter()
        .find(|n| n.kind == *kind && ChainName::from_chain_id(n.chain_id) == *chain_name)
        .unwrap()
        .clone()
}

pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();
    let networks = SUPPORTED_NETWORKS
        .iter()
        .map(|c| {
            let c: ChainInfoOwned = c.clone().into();
            c
        })
        .filter(|c| c.kind == ChainKind::Mainnet)
        // .filter(|c| c.network_info.chain_name == "terra2")
        .map(|mut c| {
            if c.chain_id == "osmosis-1" {
                c.grpc_urls = vec!["https://osmosis-grpc.lavenderfive.com:443".to_string()];
            }
            if c.chain_id == "migaloo-1" {
                c.grpc_urls = vec!["https://migaloo-grpc.lavenderfive.com:443".to_string()];
            }
            c
        });

    for network in networks {
        let src_chain = DaemonBuilder::default().chain(network.clone()).build()?;

        let deployment = Abstract::load_from(src_chain.clone());
        let deployment = if let Ok(deployment) = deployment {
            deployment
        } else {
            continue;
        };

        // Check out all the outgoing channels
        let remote_infrastructures = deployment.ibc.client.list_ibc_infrastructures()?;

        let all_infra_and_channels = remote_infrastructures
            .counterparts
            .iter()
            .flat_map(|(chain_name, infra)| {
                let channel = get_channel_from_infra(&src_chain, chain_name, infra)?;
                // assert_active(&src_chain, &channel)?;
                // assert_no_pending_packets(&src_chain, &channel)?;
                Ok::<_, anyhow::Error>((chain_name, channel))
            })
            .collect::<Vec<_>>();

        println!("From {} : {:?}", network.chain_id, all_infra_and_channels)
    }

    Ok(())
}

#[cw_serde]
pub struct PortAndChannel {
    pub port: String,
    pub channel: String,
}

fn get_channel_from_infra(
    src_chain: &Daemon,
    dst_chain_name: &ChainName,
    infra: &IbcInfrastructure,
) -> anyhow::Result<PortAndChannel> {
    let src_chain_data = &src_chain.state().chain_data;
    let dst_chain = get_corresponding_chain(dst_chain_name, &src_chain_data.kind);
    let chain_with_deployment_id = DaemonBuilder::default()
        .chain(src_chain_data.clone())
        .deployment_id(get_polytone_deployment_id(
            src_chain_data.clone(),
            dst_chain.clone(),
        ))
        .build()?;

    let polytone = Polytone::load_from(chain_with_deployment_id.clone())?;

    Ok(PortAndChannel {
        port: format!("wasm.{}", infra.polytone_note.clone()),
        channel: polytone.note.active_channel()?.unwrap(),
    })
}

fn assert_active(chain: &Daemon, channel: &PortAndChannel) -> anyhow::Result<()> {
    let ibc: Ibc = chain.querier();
    let ibc_channel = chain
        .rt_handle
        .block_on(ibc._channel(channel.port.clone(), channel.channel.clone()))?;

    let open_state: i32 = State::Open.into();

    ensure_eq!(
        ibc_channel.state,
        open_state,
        StdError::generic_err(format!("Channel {:?} no open", channel))
    );

    // For polytone, there should only be 1 hop
    ensure_eq!(
        ibc_channel.connection_hops.len(),
        1,
        StdError::generic_err(format!("Wrong number of connection hops for {:?}", channel))
    );
    let connection_hop = &ibc_channel.connection_hops[0];
    let connection = chain
        .rt_handle
        .block_on(ibc._connection_end(connection_hop))?
        .ok_or(StdError::generic_err(format!(
            "Connection doesn't exist for {:?}",
            channel
        )))?;

    let connection_open_state: i32 = connection::v1::State::Open.into();
    ensure_eq!(
        connection.state,
        connection_open_state,
        StdError::generic_err(format!("Connection {:?} no open", connection_hop))
    );

    let client = connection.client_id;
    let client_state = chain.rt_handle.block_on(ibc._client_state(client))?;
    todo!("We need to be able to deserialize the client state");
    todo!("We need to check that the client is healthy here, in the right state ?");
    Ok(())
}

fn assert_no_pending_packets(chain: &Daemon, channel: &PortAndChannel) -> anyhow::Result<()> {
    todo!();

    Ok(())
}
