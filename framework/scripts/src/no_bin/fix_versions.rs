use abstract_core::{
    objects::module::{Module, ModuleInfo, ModuleVersion},
    version_control::{ModuleFilter, ModulesListResponse},
};
use abstract_interface::{Abstract, VCExecFns, VCQueryFns};
use cw_orch::{networks::UNI_6, *};

use abstract_core::objects::namespace::Namespace;
use std::sync::Arc;
use tokio::runtime::Runtime;

const NETWORK: cw_orch::ChainInfo = UNI_6;
const WRONG_VERSION: &str = "0.1.0-rc.3";
const NEW_VERSION: &str = env!("CARGO_PKG_VERSION");
const NAMESPACE: &str = "abstract";

/// Script that takes existing versions in Version control, removes them, and swaps them wit ha new version
pub fn fix_versions() -> anyhow::Result<()> {
    let _rt = Arc::new(Runtime::new()?);
    let chain = DaemonBuilder::default().chain(NETWORK).build()?;

    let deployment = Abstract::new(chain);

    let ModulesListResponse { modules } = deployment.version_control.module_list(
        Some(ModuleFilter {
            namespace: Some(NAMESPACE.to_string()),
            version: Some(WRONG_VERSION.to_string()),
            ..Default::default()
        }),
        None,
        None,
    )?;

    for Module { info, reference } in modules {
        let ModuleInfo {
            version,
            name,
            namespace,
        } = info.clone();
        if version.to_string() == *WRONG_VERSION && namespace == Namespace::unchecked(NAMESPACE) {
            deployment.version_control.remove_module(info)?;
            deployment.version_control.propose_modules(vec![(
                ModuleInfo {
                    name,
                    namespace,
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
