use abstract_core::{MANAGER, VERSION_CONTROL};
use abstract_interface::{Manager, VersionControl};
use clap::Parser;
use cw_orch::{networks::parse_network, *};
use semver::Version;
use std::sync::Arc;
use tokio::runtime::Runtime;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn migrate(network: cw_orch::ChainInfo) -> anyhow::Result<()> {
    let rt = Arc::new(Runtime::new()?);
    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(network)
        .build()?;

    let abstract_version = Version::parse(VERSION)?;

    let vc = VersionControl::new(VERSION_CONTROL, chain.clone());

    let manager = Manager::new(MANAGER, chain);
    manager.upload()?;

    // Register the new manager
    vc.register_account_mods(vec![manager.as_instance()], &abstract_version)?;

    Ok(())
}

// TODO: base arguments
#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long)]
    network_id: String,
}

//
fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let args = Arguments::parse();

    let network = parse_network(&args.network_id);

    if let Err(ref err) = migrate(network) {
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
