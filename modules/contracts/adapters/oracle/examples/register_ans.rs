use abstract_adapter::objects::UncheckedContractEntry;
use abstract_interface::{Abstract, ExecuteMsgFns};
use abstract_oracle_adapter::interface::deployment::pyth_addresses;
use abstract_pyth_adapter::PYTH;
use cw_orch::daemon::networks::parse_network;
use cw_orch::prelude::*;

fn deploy_oracle(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    // run for each requested network
    for network in networks {
        let chain = DaemonBuilder::new(network.clone()).build()?;
        let abstr = Abstract::load_from(chain.clone())?;

        // This works only for PYTH, we have to find a better logic for other oracles and adapters
        abstr.ans_host.update_contract_addresses(
            vec![(
                UncheckedContractEntry {
                    protocol: PYTH.to_string(),
                    contract: "oracle".to_string(),
                },
                pyth_addresses().get(network.chain_id).unwrap().to_string(),
            )],
            vec![],
        )?;
    }
    Ok(())
}

use clap::Parser;

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    network_ids: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let args = Arguments::parse();
    let networks = args
        .network_ids
        .iter()
        .map(|n| parse_network(n).unwrap())
        .collect();

    deploy_oracle(networks)
}
