use abstract_boot::{OSFactory, OsFactoryQueryFns, VersionControl, OS};
use abstract_os::{manager, os_factory, proxy, MANAGER, OS_FACTORY, PROXY, VERSION_CONTROL};
use boot_core::{
    networks::{parse_network, NetworkInfo},
    prelude::*,
};
use std::sync::Arc;
use tokio::runtime::Runtime;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn migrate(network: NetworkInfo) -> anyhow::Result<()> {
    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(network).build();
    let (_sender, chain) = instantiate_daemon_env(&rt, options?)?;

    let _abstract_os_version: Version = VERSION.parse().unwrap();

    let _version_control = VersionControl::new(VERSION_CONTROL, chain.clone());

    // Upload the new core contracts
    let _os_core = OS::new(chain.clone(), None);
    // os_core.upload()?;
    // os_core.register(&version_control, VERSION)?;

    // Register the cores
    // version_control.register_cores(vec![os_core.proxy.as_instance()], &abstract_os_version)?;

    let os_factory = OSFactory::new(OS_FACTORY, chain.clone());
    let os_factory::ConfigResponse { next_os_id, .. } = OsFactoryQueryFns::config(&os_factory)?;
    let latest_os_id = next_os_id - 1;

    for os_id in 1..=latest_os_id {
        let os = OS::new(chain.clone(), Some(os_id));
        // todo: check admin

        // Upgrade manager first
        os.manager.upgrade(vec![(
            ModuleInfo::from_id_latest(MANAGER)?,
            Some(to_binary(&manager::MigrateMsg {}).unwrap()),
        )])?;

        // Then upgrade proxy
        os.manager.upgrade(vec![(
            ModuleInfo::from_id_latest(PROXY)?,
            Some(to_binary(&proxy::MigrateMsg {}).unwrap()),
        )])?;
    }

    // // Deregister the app
    // version_control.remove_module(ModuleInfo::from_id(
    //     MANAGER,
    //     ModuleVersion::Version(abstract_os_version.to_string()),
    // )?)?;

    // Register the cores
    // version_control.register_cores(vec![os_core.manager.as_instance()], &abstract_os_version)?;

    // let mut vc = VersionControl::new(VERSION_CONTROL, chain);
    //
    // vc.upload()?;
    //
    // vc.migrate(&abstract_os::version_control::MigrateMsg {}, vc.code_id()?)?;

    //     ans_host.instantiate(&ans_host::InstantiateMsg {}, Some(&sender), None)?;
    //
    //     ans_host.as_instance();
    //
    //     // ans_host.query(&abstract_os::ans_host::QueryMsg::DexPools { dex: None, asset_pair: None })?;
    //
    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long)]
    network_id: String,
}

use abstract_os::{manager::ExecuteMsgFns, objects::module::ModuleInfo};
use clap::Parser;
use cosmwasm_std::to_binary;
use semver::Version;

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
