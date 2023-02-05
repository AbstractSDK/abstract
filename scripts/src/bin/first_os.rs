use std::sync::Arc;

use boot_core::networks::UNI_5;
use boot_core::prelude::*;

use semver::Version;
use tokio::runtime::Runtime;

use abstract_boot::Abstract;

use abstract_os::objects::gov_type::GovernanceDetails;


/// Script that registers the first OS in abstract (our OS)
pub fn first_os() -> anyhow::Result<()> {
    let abstract_os_version: Version = "0.1.0-rc.3".parse().unwrap();
    let network = UNI_5;
    // let network = LOCAL_JUNO;
    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(network).build();
    let (sender, chain) = instantiate_daemon_env(&rt, options?)?;

    let deployment = Abstract::new(chain, abstract_os_version);

    // NOTE: this assumes that the deployment has been deployed

    deployment
        .os_factory
        .create_default_os(GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        })?;

    deployment.ans_host.update_all()?;

    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    if let Err(ref err) = first_os() {
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
