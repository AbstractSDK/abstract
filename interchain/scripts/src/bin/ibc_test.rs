use abstract_client::AbstractClient;
use clap::Parser;

use cosmwasm_std::CosmosMsg;
use cw_orch::daemon::DaemonState;
use cw_orch::environment::ChainState;
use cw_orch::prelude::*;

use cw_orch_interchain::prelude::*;
use networks::parse_network;

fn get_daemon(
    chain: ChainInfo,
    mnemonic: Option<String>,
    state: Option<DaemonState>,
    deployment_id: Option<String>,
) -> cw_orch::anyhow::Result<Daemon> {
    let mut builder = DaemonBuilder::new(chain);
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

fn test(
    (src_chain, src_mnemonic): (ChainInfo, Option<String>),
    (dst_chain, dst_mnemonic): (ChainInfo, Option<String>),
) -> cw_orch::anyhow::Result<()> {
    let src_daemon = get_daemon(src_chain.clone(), src_mnemonic.clone(), None, None)?;
    let dst_daemon = get_daemon(
        dst_chain.clone(),
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

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    src: String,

    #[arg(short, long)]
    dst: String,
}

/// Test IBC connection between two chains.
fn main() {
    dotenv::dotenv().unwrap();
    env_logger::init();

    let args = Arguments::parse();

    // let networks = vec![abstract_scripts::ROLLKIT_TESTNET];

    let src_network = parse_network(&args.src).unwrap();
    let dst_network = parse_network(&args.dst).unwrap();

    if let Err(ref err) = test((src_network.clone(), None), (dst_network.clone(), None))
        .or(test((dst_network, None), (src_network, None)))
    {
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
