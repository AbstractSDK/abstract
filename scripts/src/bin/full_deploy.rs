use std::sync::Arc;

use boot_core::networks::UNI_5;
use boot_core::prelude::*;

use semver::Version;
use tokio::runtime::Runtime;

use abstract_boot::{Deployment, DexExtension};

pub fn script() -> anyhow::Result<()> {
    let abstract_os_version: Version = "0.1.0-rc.3".parse().unwrap();
    let network = UNI_5;
    // let network = LOCAL_JUNO;
    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(network).build();
    let (_sender, chain) = instantiate_daemon_env(&rt, options?)?;

    let mut deployment = Deployment::new(&chain, abstract_os_version);

    deployment.deploy()?;

    let _dex = DexExtension::new("dex", &chain);
    // dex.simulate_swap()

    // let remote_network = boot_core::networks::OSMO_4;
    // let (_rt, _osmo_sender, remote_chain) = instantiate_daemon_env(remote_network)?;
    // let _host = OsmosisHost::new("osmosis_host", &remote_chain);
    // // use cw1 with stargate feature
    // let _cw_1 = Cw1::new("cw1_proxy", &remote_chain).set_variant("cw1_whitelist_stargate.wasm");
    // let _osmosis_mem = AnsHost::new("ans_host", &remote_chain);
    // // deploy_abstract(&chain, abstract_os_version)?;
    // manager.add_module(
    //     staking_api.as_instance(),
    //     Some(&extension::InstantiateMsg {
    //         app: Empty {},
    //         base: extension::BaseInstantiateMsg {
    //             ans_host_address: ans_host.address()?.into(),
    //             version_control_address: version_control.address()?.into(),
    //         },
    //     }),
    //     &staking_api.as_instance().id,
    //     abstract_os_version.to_string(),
    // )?;
    // client.upload()?;
    // client.migrate(&ibc_client::MigrateMsg{}, client.code_id()?)?;
    // host.upload()?;
    // host.migrate(&ibc_host::MigrateMsg{}, host.code_id()?)?;
    // client.instantiate(&ibc_client::InstantiateMsg{chain: "juno".into(), ans_host_address: ans_host.address()?.into(), version_control_address: version_control.address()?.into()}, Some(&sender), None)?;
    // dex_api.upload()?;
    // dex_api.instantiate(&BaseInstantiateMsg { ans_host_address: ans_host.address()?.into_string(), version_control_address: version_control.address()?.into_string() }, None, None)?;

    // cw_1.upload()?;
    // osmosis_mem.upload();
    // osmosis_mem.instantiate(&ans_host::InstantiateMsg {}, Some(&osmo_sender), None)?;
    // host.instantiate(
    //     &abstract_os::ibc_host::BaseInstantiateMsg {
    //         ans_host_address: osmosis_mem.address()?.into(),
    //         cw1_code_id: cw_1.code_id()?,
    //     },
    //     Some(&osmo_sender),
    //     None,
    // )?;

    // os_factory.create_default_os(GovernanceDetails::Monarchy {
    //         monarch: sender.to_string(),
    //     })?;
    // manager.execute(&manager::ExecuteMsg::UpdateModuleAddresses { to_add: Some(vec![(IBC_CLIENT.into(),client.address()?.into())]), to_remove: None }, None)?;
    // ans_host.update_channels()?;
    // osmosis_mem.update_channels()?;
    // manager.execute_on_module(PROXY, proxy::ExecuteMsg::AddModule { module: client.address()?.into_string() })?;
    // manager.execute_on_module(PROXY, proxy::ExecuteMsg::IbcAction { msgs: vec![ibc_client::ExecuteMsg::SendFunds { host_chain: "osmosis".into(), funds: vec![Coin::new(100,"ujunox")] }] })?;
    // let resp: ListAccountsResponse = host.query(&HostQueryMsg::Base(BaseQueryMsg::ListAccounts {  } ))?;
    // println!("{:?}", resp);
    // manager.execute_on_module(
    //     PROXY,
    //     proxy::ExecuteMsg::IbcAction {
    //         msgs: vec![ibc_client::ExecuteMsg::Register {
    //             host_chain: "osmosis".into(),
    //         }],
    //     },
    // )?;
    // manager.execute_on_module(PROXY, proxy::ExecuteMsg::IbcAction { msgs: vec![ibc_client::ExecuteMsg::SendPacket { host_chain: "osmosis".into(), action: ibc_host::HostAction::SendAllBack { }, callback_info: None, retries: 0 }] })?;
    // ans_host.update_all()?;

    // osmosis_mem.update_all();
    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    if let Err(ref err) = script() {
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
