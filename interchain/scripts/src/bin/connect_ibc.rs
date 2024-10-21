use abstract_interface::Abstract;
use clap::Parser;
use cw_orch::daemon::DaemonState;
use cw_orch::prelude::*;

use cw_orch_interchain::prelude::*;
use networks::parse_network;

fn get_daemon(
    chain: ChainInfo,
    mnemonic: Option<String>,
    deployment_id: Option<String>,
    state: Option<DaemonState>,
) -> cw_orch::anyhow::Result<Daemon> {
    let mut builder = DaemonBuilder::new(chain);
    if let Some(state) = state {
        builder.state(state);
    }
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
) -> cw_orch::anyhow::Result<()> {
    let src_daemon = get_daemon(src_chain.clone(), src_mnemonic.clone(), None, None)?;
    let dst_daemon = get_daemon(
        dst_chain.clone(),
        dst_mnemonic,
        None,
        Some(src_daemon.state()),
    )?;

    let src_abstract = Abstract::load_from(src_daemon.clone())?;
    let dst_abstract = Abstract::load_from(dst_daemon.clone())?;

    let interchain =
        DaemonInterchain::from_daemons(vec![src_daemon, dst_daemon], &ChannelCreationValidator);
    src_abstract.connect_to(&dst_abstract, &interchain)?;

    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    src: String,

    #[arg(short, long)]
    dst: String,
}

fn main() {
    dotenv::dotenv().unwrap();
    env_logger::init();

    let args = Arguments::parse();

    // let networks = vec![abstract_scripts::ROLLKIT_TESTNET];

    let src_network = parse_network(&args.src).unwrap();
    let dst_network = parse_network(&args.dst).unwrap();

    if let Err(ref err) = connect((src_network, None), (dst_network, None)) {
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
