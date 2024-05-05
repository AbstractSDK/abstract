use abstract_interface::Abstract;
use abstract_scripts::abstract_ibc::get_polytone_deployment_id;
use abstract_std::ibc_client::QueryMsgFns;
use abstract_std::objects::chain_name::ChainName;
use cw_orch::daemon::queriers::Ibc;
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
        let chain = DaemonBuilder::default().chain(network.clone()).build()?;

        let deployment = Abstract::load_from(chain.clone());
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
                let src_chain = &chain.state().chain_data;
                let dst_chain = get_corresponding_chain(chain_name, &src_chain.kind);
                let chain_with_deployment_id = DaemonBuilder::default()
                    .chain(network.clone())
                    .deployment_id(get_polytone_deployment_id(
                        src_chain.clone(),
                        dst_chain.clone(),
                    ))
                    .build()?;

                let polytone = Polytone::load_from(chain_with_deployment_id.clone())?;

                let ibc: Ibc = chain.querier();
                let channel = chain.rt_handle.block_on(ibc._channel(
                    format!("wasm.{}", infra.polytone_note.clone()),
                    polytone.note.active_channel()?.unwrap(),
                ))?;

                Ok::<_, anyhow::Error>((chain_name, channel))
            })
            .collect::<Vec<_>>();

        println!("From {} : {:?}", network.chain_id, all_infra_and_channels)
    }

    Ok(())
}
