//! Currently you can run only 1 test at a time: `cargo mt`
//! Otherwise you will have too many requests

use abstract_app::mock::MockInitMsg;
use abstract_client::AbstractClient;
use abstract_core::{
    objects::{gov_type::GovernanceDetails, module::ModuleInfo},
    ABSTRACT_EVENT_TYPE, MANAGER, PROXY,
};
use abstract_integration_tests::manager::mock_app::{MockApp, APP_VERSION};
use abstract_interface::{
    Abstract, AbstractAccount, AppDeployer, DeployStrategy, ManagerExecFns, VCExecFns,
};
use abstract_testing::prelude::*;
use anyhow::Ok;
use cosmwasm_std::{to_json_binary, Addr};
use cw_asset::AssetInfoUnchecked;
use cw_orch::{
    daemon::networks::{JUNO_1, NEUTRON_1},
    prelude::*,
};
use cw_orch_clone_testing::CloneTesting;

// testnet addr of abstract
const SENDER: &str = "neutron14cl2dthqamgucg9sfvv4relp3aa83e4039dt7e";

/// Returns a shared tokio runtime for all tests
fn rt() -> &'static tokio::runtime::Runtime {
    lazy_static::lazy_static! {
        static ref RT: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Should create a tokio runtime");
    }
    &RT
}

fn setup_allowed_direct_module_registration(
) -> anyhow::Result<(AbstractClient<CloneTesting>, CloneTesting)> {
    let mut chain_info = NEUTRON_1;
    let sender = Addr::unchecked(SENDER);
    let mut chain = CloneTesting::new(rt(), chain_info)?;
    chain.set_sender(sender.clone());

    let abstr_deployment = AbstractClient::new(chain.clone())?;

    // deployment.migrate_if_version_changed()?;
    abstr_deployment
        .version_control()
        .update_config(None, Some(true), None)?;
    Ok((abstr_deployment, chain))
}

#[test]
fn check() -> anyhow::Result<()> {
    let (abstr, sender) = setup_allowed_direct_module_registration()?;
    Ok(())
}
