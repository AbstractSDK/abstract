use std::sync::Arc;

use boot_core::networks::UNI_5;
use boot_core::prelude::*;

use semver::Version;
use tokio::runtime::Runtime;

use abstract_boot::{Deployment, DexApi, OS};

pub fn full_deploy() -> anyhow::Result<()> {
    let abstract_os_version: Version = "0.1.0-rc.3".parse().unwrap();
    let network = UNI_5;
    // let network = LOCAL_JUNO;
    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(network).build();
    let (_sender, chain) = instantiate_daemon_env(&rt, options?)?;

    let mut os_core = OS::new(&chain, None);

    let mut deployment = Deployment::new(&chain, abstract_os_version);

    deployment.deploy(&mut os_core)?;

    let _dex = DexApi::new("dex", chain.clone());

    let ans_host = deployment.ans_host;
    ans_host.update_all()?;

    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    if let Err(ref err) = full_deploy() {
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
