use std::str::FromStr;

use cosmwasm_std::coins;
use cw_asset::{Asset, AssetInfoUnchecked};
use cw_orch::anyhow;
use cw_orch::prelude::*;
use speculoos::prelude::*;

use abstract_core::app::ExecuteMsg;
use abstract_core::manager::ExecuteMsgFns;
use abstract_core::objects::AnsAsset;
use abstract_core::{
    ans_host::ExecuteMsgFns as AnsHostExecuteMsgFns,
    app::{BaseInstantiateMsg, InstantiateMsg},
    manager::QueryMsgFns,
    objects::gov_type::GovernanceDetails,
    version_control::ExecuteMsgFns as VcExecMsgFns,
};
use abstract_gas_station_app::msg::{
    GasStationExecuteMsg, GasStationInstantiateMsg, GradeListResponse,
};
use abstract_gas_station_app::{
    contract::{GAS_STATION_APP_ID, VERSION},
    GasStationApp, GasStationExecuteMsgFns, GasStationQueryMsgFns,
};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer};
use abstract_testing::prelude::*;
use cw_orch::deploy::Deploy;

const TEST_GRADE: &str = "osmo";
struct GasStationTest<Env: CwEnv> {
    account: AbstractAccount<Env>,
    abstr: Abstract<Env>,
    gas_station: GasStationApp<Env>,
    tube: Env,
}

impl GasStationTest<OsmosisTestTube> {
    fn setup(initial_balance: Option<Vec<Coin>>) -> anyhow::Result<Self> {
        // Download the adapter wasm
        // Create the OsmosisTestTube
        let tube =
            OsmosisTestTube::new(initial_balance.unwrap_or(coins(1_000_000_000_000, GAS_DENOM)));

        let abstr = Abstract::deploy_on(tube.clone(), tube.sender().to_string()).unwrap();

        let gas_station = deploy_gas_station(&tube);

        let account = setup_new_account(&abstr, TEST_NAMESPACE)?;
        setup_default_assets(&abstr);
        account.install_module(
            GAS_STATION_APP_ID,
            &InstantiateMsg {
                base: BaseInstantiateMsg {
                    ans_host_address: abstr.ans_host.addr_str()?,
                },
                module: GasStationInstantiateMsg {},
            },
            None,
        )?;

        let modules = account.manager.module_infos(None, None)?;
        gas_station.set_address(&modules.module_infos[0].address);

        Ok(Self {
            tube,
            account,
            abstr,
            gas_station,
        })
    }

    fn sender(&self) -> Addr {
        self.tube.sender()
    }

    // execute a msg on the gas station
    fn execute_on_station(&self, msg: GasStationExecuteMsg) -> anyhow::Result<()> {
        self.account
            .manager
            .execute_on_module(
                GAS_STATION_APP_ID,
                ExecuteMsg::<GasStationExecuteMsg, Empty>::from(msg),
            )
            .unwrap();

        Ok(())
    }

    // create a new grade with fuel mix
    fn create_grade(&self, grade: &str, fuel_mix: Vec<AnsAsset>) -> anyhow::Result<()> {
        self.execute_on_station(GasStationExecuteMsg::CreateGrade {
            grade: grade.to_string(),
            fuel_mix,
        })?;

        Ok(())
    }
}
// impl GasStationTest<Mock> {
//     fn setup(initial_balance: Option<Vec<Coin>>) -> anyhow::Result<Self> {
//         // Download the adapter wasm
//         // Create the OsmosisTestTube
//         let tube = Mock::new(&Addr::unchecked(TEST_OWNER));
//
//         let abstr = Abstract::deploy_on(tube.clone(), tube.sender().to_string()).unwrap();
//
//         let gas_station = deploy_gas_station(&tube);
//
//         let account = setup_new_account(&abstr, TEST_NAMESPACE)?;
//         setup_default_assets(&abstr);
//         account.install_module(
//             GAS_STATION_APP_ID,
//             &InstantiateMsg {
//                 base: BaseInstantiateMsg {
//                     ans_host_address: abstr.ans_host.addr_str()?,
//                 },
//                 module: GasStationInstantiateMsg {
//
//                 }
//             },
//             None,
//         )?;
//
//         let modules = account.manager.module_infos(None, None)?;
//         gas_station.set_address(&modules.module_infos[0].address);
//
//         Ok(Self {
//             tube,
//             account,
//             abstr,
//             gas_station,
//         })
//     }
// }

fn setup_default_assets<Env: CwEnv>(abstr: &Abstract<Env>) {
    // register juno as an asset
    abstr
        .ans_host
        .update_asset_addresses(
            vec![(
                GAS_ANS_ID.to_string(),
                AssetInfoUnchecked::from_str(&format!("native:{}", GAS_DENOM)).unwrap(),
            )],
            vec![],
        )
        .unwrap();
}

// Uploads and returns the giftcard issuer
fn deploy_gas_station<Env: CwEnv>(tube: &Env) -> GasStationApp<Env> {
    let station = GasStationApp::new(GAS_STATION_APP_ID, tube.clone());

    // deploy the abstract gas station
    station.deploy(VERSION.parse().unwrap()).unwrap();

    station
}

const GAS_DENOM: &str = "uosmo";
const GAS_ANS_ID: &str = "osmo>osmo";

// Returns an account with the necessary setup
fn setup_new_account<Env: CwEnv>(
    abstr_deployment: &Abstract<Env>,
    namespace: impl ToString,
) -> anyhow::Result<AbstractAccount<Env>> {
    // TODO: might need to move this
    let signing_account = abstr_deployment.account_factory.get_chain().sender();

    // Create a new account to install the app onto
    let account = abstr_deployment
        .account_factory
        .create_default_account(GovernanceDetails::Monarchy {
            monarch: signing_account.into_string(),
        })
        .unwrap();

    // claim the namespace so app can be deployed
    abstr_deployment
        .version_control
        .claim_namespace(account.id().unwrap(), namespace.to_string())
        .unwrap();

    // register base asset!
    // account.proxy.call_as(&abstr_deployment.account_factory.get_chain().sender).update_assets(vec![(AssetEntry::from(ISSUE_ASSET), UncheckedPriceSource::None)], vec![]).unwrap();

    Ok(account)
}

#[test]
fn successful_install_with_no_grades() -> anyhow::Result<()> {
    // Set up the environment and contract
    let test_env = GasStationTest::setup(None)?;

    let pump_list: GradeListResponse = test_env.gas_station.grade_list()?;
    assert_that!(pump_list.grades).is_empty();
    Ok(())
}

#[test]
fn create_grade() -> anyhow::Result<()> {
    // Set up the environment and contract
    let test_env = GasStationTest::setup(None)?;

    let fuel_mix: Vec<AnsAsset> =
        vec![AnsAsset::new(GAS_ANS_ID.to_string(), 1_000_000_000_000u128)];

    // Create grade
    test_env.create_grade("osmo", fuel_mix)?;
    // let create_pump_res = test_env.gas_station.create_gas_pump(fuel_mix, "osmo".to_string())?;

    // Check grade in list
    let grade_list: GradeListResponse = test_env.gas_station.grade_list()?;
    assert_that!(grade_list.grades).has_length(1);
    let grade_info = grade_list.grades[0].clone();
    assert_that!(grade_info.grade).is_equal_to("osmo".to_string());
    assert_that!(grade_info.fuel_mix).is_equal_to(vec![Asset::native(
        GAS_DENOM,
        1_000_000_000_000u128,
    )
    .try_into()
    .unwrap()]);

    Ok(())
}

#[test]
fn activate_pass() -> anyhow::Result<()> {
    // Set up the environment and contract
    let test_env = GasStationTest::setup(None)?;

    let fuel_mix: Vec<AnsAsset> =
        vec![AnsAsset::new(GAS_ANS_ID.to_string(), 1_000_000_000_000u128)];

    // Create grade
    test_env.create_grade(TEST_GRADE, fuel_mix)?;

    let sender = test_env.sender().to_string();

    // Activate pass
    test_env.execute_on_station(GasStationExecuteMsg::ActivateGasPass {
        grade: TEST_GRADE.to_string(),
        recipient: sender.clone(),
        expiration: None,
        bypass_pass_check: false,
    })?;

    // Check pass
    let pass = test_env.gas_station.gas_pass(sender.clone())?;
    assert_that!(pass.grade).is_equal_to("osmo".to_string());
    assert_that!(pass.expiration).is_none();
    assert_that!(pass.holder).is_equal_to(sender);

    Ok(())
}

#[test]
fn deactivate_pass() -> anyhow::Result<()> {
    // Set up the environment and contract
    let test_env = GasStationTest::setup(None)?;

    let fuel_mix: Vec<AnsAsset> =
        vec![AnsAsset::new(GAS_ANS_ID.to_string(), 1_000_000_000_000u128)];

    // Create grade
    test_env.create_grade(TEST_GRADE, fuel_mix)?;

    let sender = test_env.sender().to_string();

    // Deactivate pass
    test_env.execute_on_station(GasStationExecuteMsg::DeactivateGasPass {
        holder: sender.clone(),
    })?;

    // Check pass
    let pass = test_env.gas_station.gas_pass(sender.clone())?;
    assert_that!(pass.grade).is_equal_to("osmo".to_string());
    assert_that!(pass.expiration).is_none();
    assert_that!(pass.holder).is_equal_to(sender);

    Ok(())
}
