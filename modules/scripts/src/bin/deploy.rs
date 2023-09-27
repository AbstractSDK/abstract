use reqwest::Url;
use std::{
    fs::{self, File},
    io::BufReader,
    net::TcpStream,
};
use abstract_interface::Abstract;

use abstract_module_scripts::{assert_wallet_balance, DeploymentStatus, SUPPORTED_CHAINS};
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
fn full_deploy(mut networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;

    for network in networks {
        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(network.clone())
            .build()?;
        let deployment = Abstract::load_from(chain)?;

        let sender = chain.sender();
        
    Ok(())
}

async fn ping_grpc(url_str: &str) -> anyhow::Result<()> {
    let parsed_url = Url::parse(url_str)?;

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
    let path = dirs::home_dir()
        .unwrap()
        .join(".cw-orchestrator")
        .join("chains.json");
    let status_str = serde_json::to_string_pretty(status)?;
    fs::write(path, status_str)?;
    Ok(())
}

fn read_deployment() -> anyhow::Result<DeploymentStatus> {
    let path = dirs::home_dir()
        .unwrap()
        .join(".cw-orchestrator")
        .join("chains.json");
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `DeploymentStatus`. If not present use default.
    Ok(serde_json::from_reader(reader).unwrap_or_default())
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
