use abstract_core::{app::BaseInstantiateMsg, objects::gov_type::GovernanceDetails};
use abstract_dex_adapter::{contract::CONTRACT_VERSION, msg::DexInstantiateMsg, EXCHANGE};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, VCExecFns, *};
use abstract_payment_app::{
    contract::{APP_ID, APP_VERSION},
    msg::{AppInstantiateMsg, ConfigResponse, InstantiateMsg},
    *,
};
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, prelude::*};

use cosmwasm_std::{Addr, Decimal};

// consts for testing
const ADMIN: &str = "admin";

/// Set up the test environment with the contract installed
fn setup() -> anyhow::Result<(AbstractAccount<Mock>, Abstract<Mock>, PaymentApp<Mock>)> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Construct the counter interface
    let contract = PaymentApp::new(APP_ID, mock.clone());

    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), Empty {})?;

    let dex_adapter = abstract_dex_adapter::interface::DexAdapter::new(
        abstract_dex_adapter::EXCHANGE,
        mock.clone(),
    );
    dex_adapter.deploy(
        CONTRACT_VERSION.parse().unwrap(),
        DexInstantiateMsg {
            recipient_account: 0,
            swap_fee: Decimal::percent(1),
        },
    )?;

    // Create a new account to install the app onto
    let account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: ADMIN.to_string(),
            })?;

    // claim the namespace so app can be deployed
    abstr_deployment
        .version_control
        .claim_namespaces(1, vec!["my-namespace".to_string()])?;

    contract.deploy(APP_VERSION.parse()?)?;

    // install exchange module as it's a dependency
    account.install_module(EXCHANGE, &Empty {}, None)?;

    account.install_module(
        APP_ID,
        &InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: abstr_deployment.ans_host.addr_str()?,
            },
            module: AppInstantiateMsg {
                desired_asset: None,
                exchanges: vec![],
            },
        },
        None,
    )?;

    let modules = account.manager.module_infos(None, None)?;
    contract.set_address(&modules.module_infos[1].address);

    Ok((account, abstr_deployment, contract))
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, app) = setup()?;

    let config = app.config()?;
    assert_eq!(
        config,
        ConfigResponse {
            desired_asset: None,
            exchanges: vec![]
        }
    );
    Ok(())
}
