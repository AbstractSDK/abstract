use abstract_core::objects::{gov_type::GovernanceDetails, AccountId};
use abstract_dex_adapter::{contract::CONTRACT_VERSION, msg::DexInstantiateMsg, DEX_ADAPTER_ID};
use abstract_interface::{
    Abstract, AbstractAccount, AdapterDeployer, AppDeployer, DeployStrategy, ManagerQueryFns,
    VCExecFns,
};
use payment_app::{
    contract::{APP_ID, APP_VERSION},
    msg::{AppInstantiateMsg, ConfigResponse},
    *,
};
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, prelude::*};

use cosmwasm_std::{Addr, Decimal};

// consts for testing
const ADMIN: &str = "admin";

/// Set up the test environment with the contract installed
fn setup() -> anyhow::Result<(
    AbstractAccount<Mock>,
    Abstract<Mock>,
    PaymentAppInterface<Mock>,
)> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Construct the counter interface
    let app = PaymentAppInterface::new(APP_ID, mock.clone());

    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), sender.to_string())?;

    let dex_adapter = abstract_dex_adapter::interface::DexAdapter::new(
        abstract_dex_adapter::DEX_ADAPTER_ID,
        mock.clone(),
    );
    dex_adapter.deploy(
        CONTRACT_VERSION.parse().unwrap(),
        DexInstantiateMsg {
            recipient_account: 0,
            swap_fee: Decimal::percent(1),
        },
        DeployStrategy::Try,
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
        .claim_namespace(AccountId::local(1), "my-namespace".to_string())?;

    app.deploy(APP_VERSION.parse()?, DeployStrategy::Try)?;

    // install exchange module as it's a dependency
    account.install_module(DEX_ADAPTER_ID, &Empty {}, None)?;

    account.install_app(
        app.clone(),
        &AppInstantiateMsg {
            desired_asset: None,
            exchanges: vec![],
        },
        None,
    )?;

    let modules = account.manager.module_infos(None, None)?;
    app.set_address(&modules.module_infos[1].address);

    Ok((account, abstr_deployment, app))
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
