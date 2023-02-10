use abstract_boot::{DexApi, ModuleDeployer, VersionControl};
use abstract_os::VERSION_CONTROL;
use boot_core::prelude::*;

use boot_core::networks::{parse_network, NetworkInfo};
use cosmwasm_std::{Empty};
use semver::Version;
use std::sync::Arc;
use tokio::runtime::Runtime;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn deploy_dex(network: NetworkInfo) -> anyhow::Result<()> {
    let version: Version = CONTRACT_VERSION.parse().unwrap();

    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(network).build();
    let (_sender, chain) = instantiate_daemon_env(&rt, options?)?;

    let abstract_version: Version = version.clone();

    let vc = VersionControl::new(VERSION_CONTROL, chain.clone());

    let deployer = ModuleDeployer::load_from_version_control(
        chain.clone(),
        &abstract_version,
        &vc.address()?,
    )?;

    let mut dex = DexApi::new("abstract:dex", chain);

    deployer.deploy_api(dex.as_instance_mut(), version, Empty {})?;

    Ok(())
}

use clap::Parser;

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long)]
    network_id: String,
}

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let args = Arguments::parse();

    let network = parse_network(&args.network_id);

    deploy_dex(network)
}
