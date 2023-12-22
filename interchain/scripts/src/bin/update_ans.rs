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

// Run "cargo run --example download_wasms" in the `abstract-interfaces` package before deploying!
fn full_deploy(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    for network in networks {
        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(network)
            .build()?;

        let deployment = Abstract::load_from(chain)?;
        // Take the assets, contracts, and pools from resources and upload them to the ans host
        let ans_host = deployment.ans_host;
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
        .map(|n| parse_network(n).unwrap())
        .collect::<Vec<_>>();

    if let Err(ref err) = full_deploy(networks) {
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
