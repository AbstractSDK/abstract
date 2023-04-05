use abstract_boot::{Abstract, VCExecFns};
use abstract_core::{
    objects::module::{Module, ModuleInfo},
    version_control::{ModulesListResponse, QueryMsgFns},
};
use boot_core::{
    networks::{terra::PISCO_1, NetworkInfo},
    *,
};

use std::sync::Arc;
use tokio::runtime::Runtime;

const NETWORK: NetworkInfo = PISCO_1;
const PROVIDER: &str = "abstract";

/// Script that takes existing versions in Version control, removes them, and swaps them wit ha new version
pub fn fix_names() -> anyhow::Result<()> {
    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(NETWORK).build();
    let (_sender, chain) = instantiate_daemon_env(&rt, options?)?;

    let deployment = Abstract::new(chain);

    let ModulesListResponse { modules } =
        deployment.version_control.module_list(None, None, None)?;

    for Module { info, reference } in modules {
        let ModuleInfo {
            version,
            name,
            provider,
        } = info.clone();
        if provider == PROVIDER && name.to_string().contains('_') {
            deployment.version_control.remove_module(info)?;
            deployment.version_control.add_modules(vec![(
                ModuleInfo {
                    name: name.replace('_', "-"),
                    provider,
                    version,
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

    if let Err(ref err) = fix_names() {
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
