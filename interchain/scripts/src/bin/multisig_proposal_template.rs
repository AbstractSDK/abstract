//! Template to create proposal as abstract multisig

use abstract_interface::Abstract;
use clap::Parser;
use cw_orch::prelude::{
    networks::{parse_network, ChainInfo},
    *,
};
use cw_plus_orch::cw3_flex_multisig::ExecuteMsgInterfaceFns;
use tokio::runtime::Runtime;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(unused)]
fn migrate(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    for network in networks {
        let chain = DaemonBuilder::new(network).handle(rt.handle()).build()?;

        let deployment = Abstract::load_from(chain.clone())?;

        let mut msgs = vec![];

        // Example of abstract action
        msgs.extend(deployment.multisig.propose_on_ans_msgs(
            &deployment.ans_host,
            vec![abstract_std::ans_host::ExecuteMsg::UpdateAssetAddresses {
                to_add: vec![],
                to_remove: vec![],
            }],
        )?);

        msgs.push(todo!());

        let title: &str = todo!();
        let description: &str = todo!();
        let latest = None;
        deployment
            .multisig
            .cw3
            .propose(description, msgs, title, latest, &[])?;
    }

    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
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

    if let Err(ref err) = migrate(networks) {
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
