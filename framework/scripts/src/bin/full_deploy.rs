use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_interface::Abstract;

use abstract_interface_scripts::SUPPORTED_CHAINS;
use clap::Parser;
use cw_orch::{
    daemon::DaemonError,
    deploy::Deploy,
    prelude::{
        networks::{parse_network, ChainInfo},
        *,
    },
};

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Run "cargo run --example download_wasms" in the `abstract-interfaces` package before deploying!
fn full_deploy(mut networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    if networks.is_empty() {
        networks = SUPPORTED_CHAINS.to_vec();
    }

    // We fill the variable with what we need

    let networks = networks
        .iter()
        .map(|n| {
            // let chain = DaemonBuilder::default()
            //     .handle(rt.handle())
            //     .chain(n.clone())
            //     .build()?;

            let chain = Mock::with_chain_id(&Addr::unchecked("sender"), n.chain_id);
            Ok::<_, DaemonError>((chain.clone(), chain.sender().to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let _deployments = Abstract::full_deploy(
        networks,
        None,
        Some(|abstr| {
            abstr
                .account_factory
                .create_default_account(GovernanceDetails::Monarchy {
                    monarch: abstr.account_factory.get_chain().sender().to_string(),
                })?;

            if abstr.account_factory.get_chain().block_info()?.chain_id == "phoenix-1"{
                anyhow::bail!("Test error to show what happens when a deployment fails does");
            }

            Ok(())
        }),
    )?;

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

    let networks = args.network_ids.iter().map(|n| parse_network(n)).collect();

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
