use abstract_boot::CwStakingApi;
use abstract_boot::{ModuleDeployer, VCExecFns};
use abstract_sdk::os;
use abstract_sdk::os::cw_staking::CW_STAKING;
use abstract_sdk::os::objects::module::{Module, ModuleInfo, ModuleVersion};
use boot_core::{
    networks::NetworkInfo, prelude::instantiate_daemon_env, prelude::*, DaemonOptionsBuilder,
};
use cosmwasm_std::{Addr, Empty};
use semver::Version;
use std::sync::Arc;
use tokio::runtime::Runtime;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn deploy_cw_staking(
    network: NetworkInfo,
    prev_version: Option<String>,
    code_id: Option<u64>,
) -> anyhow::Result<()> {
    let module_version: Version = CONTRACT_VERSION.parse().unwrap();

    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(network).build();
    let (_sender, chain) = instantiate_daemon_env(&rt, options?)?;

    let abstract_version: Version = std::env::var("ABSTRACT_VERSION")
        .expect("Missing ABSTRACT_VERSION")
        .parse()
        .unwrap();
    let deployer = ModuleDeployer::load_from_version_control(
        chain.clone(),
        &abstract_version,
        &Addr::unchecked(std::env::var("VERSION_CONTROL").expect("VERSION_CONTROL not set")),
    )?;

    if let Some(prev_version) = prev_version {
        let Module { info, reference } = deployer.version_control.module(ModuleInfo::from_id(
            CW_STAKING,
            ModuleVersion::from(prev_version),
        )?)?;

        let new_info = ModuleInfo {
            version: ModuleVersion::from(CONTRACT_VERSION),
            ..info
        };
        deployer
            .version_control
            .add_modules(vec![(new_info, reference)])?;
    } else if let Some(code_id) = code_id {
        let mut cw_staking = CwStakingApi::new(CW_STAKING, chain);
        cw_staking.set_code_id(code_id);
        let init_msg = os::api::InstantiateMsg {
            app: Empty {},
            base: os::api::BaseInstantiateMsg {
                ans_host_address: deployer.ans_host.address()?.into(),
                version_control_address: deployer.version_control.address()?.into(),
            },
        };
        cw_staking
            .as_instance_mut()
            .instantiate(&init_msg, None, None)?;

        deployer
            .version_control
            .register_apis(vec![cw_staking.as_instance_mut()], &module_version)?;
    } else {
        log::info!("Uploading Cw staking");
        // Upload and deploy with the version
        let mut cw_staking = CwStakingApi::new(CW_STAKING, chain);

        deployer.deploy_api(cw_staking.as_instance_mut(), module_version, Empty {})?;
    }

    Ok(())
}

use clap::Parser;

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Use a previously deployed version instead of uploading the new one
    #[arg(short, long)]
    prev_version: Option<String>,
    #[arg(short, long)]
    network_id: String,
    #[arg(short, long)]
    code_id: Option<u64>,
}

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let Arguments {
        network_id,
        prev_version,
        code_id,
    } = Arguments::parse();

    let network = boot_core::networks::parse_network(&network_id);

    deploy_cw_staking(network, prev_version, code_id)
}
