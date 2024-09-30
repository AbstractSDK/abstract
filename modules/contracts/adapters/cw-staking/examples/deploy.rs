use abstract_adapter::abstract_interface::{
    AdapterDeployer, AnsHost, DeployStrategy, RegistryExecFns, VersionControl,
};
use abstract_adapter::std::{
    adapter,
    objects::module::{Module, ModuleInfo, ModuleVersion},
    ANS_HOST, VERSION_CONTROL,
};
use abstract_cw_staking::{interface::CwStakingAdapter, CW_STAKING_ADAPTER_ID};
use cosmwasm_std::{Addr, Empty};
use cw_orch::{daemon::DaemonBuilder, prelude::*};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn deploy_cw_staking(
    network: ChainInfo,
    prev_version: Option<String>,
    code_id: Option<u64>,
) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let chain = DaemonBuilder::new(network).handle(rt.handle()).build()?;

    let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
    version_control.set_address(&Addr::unchecked(
        std::env::var("VERSION_CONTROL").expect("VERSION_CONTROL not set"),
    ));

    let ans_host = AnsHost::new(ANS_HOST, chain.clone());

    if let Some(prev_version) = prev_version {
        let Module { info, reference } = version_control.module(ModuleInfo::from_id(
            CW_STAKING_ADAPTER_ID,
            ModuleVersion::from(prev_version),
        )?)?;

        let new_info = ModuleInfo {
            version: ModuleVersion::from(CONTRACT_VERSION),
            ..info
        };
        version_control.propose_modules(vec![(new_info, reference)])?;
    } else if let Some(code_id) = code_id {
        let mut cw_staking = CwStakingAdapter::new(CW_STAKING_ADAPTER_ID, chain);
        cw_staking.set_code_id(code_id);
        let init_msg = adapter::InstantiateMsg {
            module: Empty {},
            base: adapter::BaseInstantiateMsg {
                ans_host_address: ans_host.address()?.into(),
                version_control_address: version_control.address()?.into(),
            },
        };
        cw_staking
            .as_instance_mut()
            .instantiate(&init_msg, None, &[])?;

        version_control.register_adapters(vec![(
            cw_staking.as_instance_mut(),
            CONTRACT_VERSION.to_string(),
        )])?;
    } else {
        log::info!("Uploading Cw staking");
        // Upload and deploy with the version
        let cw_staking = CwStakingAdapter::new(CW_STAKING_ADAPTER_ID, chain);

        cw_staking.deploy(CONTRACT_VERSION.parse()?, Empty {}, DeployStrategy::Try)?;
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

    let network = cw_orch::prelude::networks::parse_network(&network_id).unwrap();

    deploy_cw_staking(network, prev_version, code_id)
}
