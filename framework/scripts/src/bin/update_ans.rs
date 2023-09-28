use abstract_interface::Abstract;

use clap::Parser;
use cw_orch::{
    deploy::Deploy,
    prelude::{
        networks::{parse_network, ChainInfo},
        *,
    },
};
use tokio::runtime::Runtime;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn update_ans(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    for network in networks {
        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(network)
            .build()?;

        let deployment = Abstract::load_from(chain)?;
        // Take the assets, contracts, and pools from resources and upload them to the ans host
        let ans_host = deployment.ans_host;

        // First we get all values
        let scraped_entries = script_helpers::get_scraped_entries(&ans_host)?;
        let on_chain_entries = script_helpers::get_on_chain_entries(&ans_host)?;

        // Then we create a diff between the 2 objects
        let diff = script_helpers::diff(scraped_entries, on_chain_entries)?;

        // Finally we upload on-chain
        script_helpers::update(&ans_host, diff)?;
        ans_host.update_all_local()?;
    }
    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long)]
    network_ids: Vec<String>,
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let args = Arguments::parse();

    let networks = args
        .network_ids
        .iter()
        .map(|n| parse_network(n))
        .collect::<Vec<_>>();

    if let Err(ref err) = update_ans(networks) {
        log::error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));

        // The backtrace is not always generated. Try to run this example
        // with `$env:RUST_BACKTRACE=1`.
        //    if let Some(backtrace) = e.backtrace() {
        //        log::debug!("backtrace: {:?}", backtrace);
        //    }

        ::std::process::exit(1);
    }
}
