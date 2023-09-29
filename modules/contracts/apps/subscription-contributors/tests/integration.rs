use abstract_core::objects::{gov_type::GovernanceDetails, AccountId};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, VCExecFns};
use abstract_subscription_contributors::{
    contract::APP_VERSION,
    msg::{ConfigResponse, ContributorsInstantiateMsg},
    state::ContributorsConfig,
    *,
};
use abstract_subscription_interface::CONTRIBUTORS_ID;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, prelude::*};

use cosmwasm_std::Addr;

// consts for testing
const ADMIN: &str = "admin";

/// Set up the test environment with the contract installed
fn setup() -> anyhow::Result<(AbstractAccount<Mock>, Abstract<Mock>, AppInterface<Mock>)> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Construct the counter interface
    let app = AppInterface::new(CONTRIBUTORS_ID, mock.clone());

    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock, sender.to_string())?;

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

    app.deploy(APP_VERSION.parse()?)?;

    account.install_app(
        app.clone(),
        &ContributorsInstantiateMsg {
            protocol_income_share: Default::default(),
            emission_user_share: Default::default(),
            max_emissions_multiple: Default::default(),
            emissions_amp_factor: Default::default(),
            emissions_offset: Default::default(),
            token_info: cw_asset::AssetInfoBase::Native("test".to_owned()),
        },
        None,
    )?;

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
            config: ContributorsConfig {
                protocol_income_share: Default::default(),
                emission_user_share: Default::default(),
                max_emissions_multiple: Default::default(),
                emissions_amp_factor: Default::default(),
                emissions_offset: Default::default(),
                token_info: cw_asset::AssetInfoBase::Native("test".to_owned())
            }
        }
    );
    Ok(())
}
