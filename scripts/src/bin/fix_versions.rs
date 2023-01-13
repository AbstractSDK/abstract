use std::sync::Arc;

use boot_core::networks::{NetworkInfo, UNI_5};
use boot_core::prelude::*;

use semver::Version;
use tokio::runtime::Runtime;

use abstract_boot::Deployment;
use abstract_os::objects::module::{ModuleInfo, ModuleVersion};
use abstract_os::version_control::{ExecuteMsgFns, ModulesResponse, QueryMsgFns};

const NETWORK: NetworkInfo = UNI_5;
const WRONG_VERSION: &str = "0.1.0-rc.3";
const NEW_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROVIDER: &str = "abstract";

/// Script that takes existing versions in Version control, removes them, and swaps them wit ha new version
pub fn fix_versions() -> anyhow::Result<()> {
    let abstract_os_version: Version = NEW_VERSION.parse().unwrap();
    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(NETWORK).build();
    let (_sender, chain) = instantiate_daemon_env(&rt, options?)?;

    let deployment = Deployment::new(&chain, abstract_os_version);

    let ModulesResponse { modules } = deployment.version_control.modules(None, None)?;

    for (info, reference) in modules {
        let ModuleInfo {
            version,
            name,
            provider,
        } = info.clone();
        if version.to_string() == *WRONG_VERSION && provider == *PROVIDER {
            deployment.version_control.remove_module(info)?;
            deployment.version_control.add_modules(vec![(
                ModuleInfo {
                    name,
                    provider,
                    version: ModuleVersion::from(NEW_VERSION),
                },
                reference,
            )])?;
        }
    }

    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    if let Err(ref err) = fix_versions() {
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
