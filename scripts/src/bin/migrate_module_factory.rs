use abstract_boot::{ModuleFactory, VersionControl};
use abstract_os::{MODULE_FACTORY, VERSION_CONTROL};

use boot_core::networks::{parse_network, NetworkInfo};
use boot_core::prelude::*;
use std::sync::Arc;
use tokio::runtime::Runtime;


use clap::Parser;
use semver::Version;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn migrate(network: NetworkInfo) -> anyhow::Result<()> {
    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(network).build();
    let (_sender, chain) = instantiate_daemon_env(&rt, options?)?;

    let abstract_os_version = Version::parse(VERSION)?;

    let vc = VersionControl::new(VERSION_CONTROL, chain.clone());

    let mut module_factory = ModuleFactory::new(MODULE_FACTORY, chain);

    module_factory.upload()?;
    module_factory.migrate(
        &abstract_os::module_factory::MigrateMsg {},
        module_factory.code_id()?,
    )?;

    vc.register_natives(vec![module_factory.as_instance()], &abstract_os_version)?;

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
