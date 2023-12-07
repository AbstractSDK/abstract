use abstract_core::objects::{gov_type::GovernanceDetails, AccountId};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, VCExecFns};
use abstract_testing::OWNER;
use app::{
    contract::{APP_ID, APP_VERSION},
    error::AppError,
    msg::{AppInstantiateMsg, ConfigResponse, CountResponse},
    *,
};
use cw_controllers::AdminError;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, prelude::*};

use cosmwasm_std::Addr;

/// Set up the test environment with the contract installed
fn setup(
    count: i32,
) -> anyhow::Result<(AbstractAccount<Mock>, Abstract<Mock>, AppInterface<Mock>)> {
    // Create a sender
    let sender = Addr::unchecked(OWNER);
    // Create the mock
    let mock = Mock::new(&sender);

    // Construct the counter interface
    let app = AppInterface::new(APP_ID, mock.clone());

    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock, sender.to_string())?;

    // Create a new account to install the app onto
    let account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: OWNER.to_string(),
            })?;

    // claim the namespace so app can be deployed
    abstr_deployment
        .version_control
        .claim_namespace(AccountId::local(1), "my-namespace".to_string())?;

    app.deploy(APP_VERSION.parse()?)?;

    account.install_app(app.clone(), &AppInstantiateMsg { count }, None)?;

    Ok((account, abstr_deployment, app))
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, app) = setup(0)?;

    let config = app.config()?;
    assert_eq!(config, ConfigResponse {});
    Ok(())
}

#[test]
fn successful_increment() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, app) = setup(0)?;

    app.increment()?;
    let count: CountResponse = app.count()?;
    assert_eq!(count.count, 1);
    Ok(())
}

#[test]
fn successful_reset() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (account, _abstr, app) = setup(0)?;

    app.call_as(&account.manager.address()?).reset(42)?;
    let count: CountResponse = app.count()?;
    assert_eq!(count.count, 6);
    Ok(())
}

#[test]
fn failed_reset() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, app) = setup(0)?;

    let err: AppError = app
        .call_as(&Addr::unchecked("NotAdmin"))
        .reset(9)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, AppError::Admin(AdminError::NotAdmin {}));
    Ok(())
}
