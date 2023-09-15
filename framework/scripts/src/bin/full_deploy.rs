use std::{fs, net::TcpStream, path::Path};

use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_interface::Abstract;

use abstract_interface_scripts::{assert_wallet_balance, DeploymentStatus};
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

    let networks = rt.block_on(assert_wallet_balance(&networks));
    println!("number: {}", number);

    for network in networks {
        let urls = network.grpc_urls.to_vec();
        for url in urls {
            rt.block_on(ping_grpc(url))?;
        }

        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(network.clone())
            .build()?;

        let sender = chain.sender();
        let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

        let mut deployment_status = DeploymentStatus {
            chain_id: network.chain_id.to_string(),
            success: false, // Default to false
        };

        match Abstract::deploy_on(chain, sender.to_string()) {
            Ok(_) => {
                deployment_status.success = true;
                write_deployment(&deployment_status)?;
            }
            Err(e) => {
                write_deployment(&deployment_status)?;
                return Err(e.into());
            }
        }

        // Create the Abstract Account because it's needed for the fees for the dex module
        deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            })?;
    }
    Ok(())
}

async fn ping_grpc(url_str: &str) -> anyhow::Result<()> {
    let parsed_url = url::Url::parse(url_str)?;

    let host = parsed_url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("No host in url"))?;

    let port = parsed_url.port_or_known_default().ok_or_else(|| {
        anyhow::anyhow!(
            "No port in url, and no default for scheme {:?}",
            parsed_url.scheme()
        )
    })?;
    let socket_addr = format!("{}:{}", host, port);

    let _ = TcpStream::connect(socket_addr);
    Ok(())
}

fn write_deployment(status: &DeploymentStatus) -> anyhow::Result<()> {
    let path = Path::new("scripts").join("deployments.json");
    let status_str = serde_json::to_string_pretty(status)?;
    fs::write(path, status_str)?;
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
