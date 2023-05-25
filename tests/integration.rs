use abstract_core::{app::BaseInstantiateMsg, objects::gov_type::GovernanceDetails};
use abstract_interface::{cw_orch::deploy::Deploy, Abstract};
use app::{
    contract::APP_ID,
    msg::{AppInstantiateMsg, InstantiateMsg, QueryMsg},
    App, AppExecuteMsgFns, AppQueryMsgFns,
};
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};

use cosmwasm_std::Addr;

// consts for testing
const USER: &str = "user";
const ADMIN: &str = "admin";

/// Set up the test environment with the contract installed
fn setup() -> anyhow::Result<(App<Mock>, Abstract<Mock>)> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Construct the counter interface
    let contract = App::new(APP_ID, mock);

    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), "v1.0.0".into())?;

    Ok((contract, abstr_deployment))
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (contract, abstr) = setup()?;

    // Create a new account to install the app onto
    let account = abstr
        .account_factory
        .create_default_account(GovernanceDetails::Monarchy {
            monarch: ADMIN.to_string(),
        })?;

    account.install_module(
        APP_ID,
        InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: abstr.ans_host.addr_str()?,
            },
            app: AppInstantiateMsg {},
        },
    )?;

    Ok(())
}
