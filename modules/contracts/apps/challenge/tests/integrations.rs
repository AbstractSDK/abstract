// Use prelude to get all the necessary imports
use abstract_challenge_app::{
    contract::{CHALLENGE_APP_ID, CHALLENGE_APP_VERSION},
    msg::{AppInstantiateMsg, ConfigResponse, InstantiateMsg},
    *,
};
use abstract_core::{
    app::BaseInstantiateMsg,
    objects::{gov_type::GovernanceDetails, AssetEntry},
};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, VCExecFns, *};
use cosmwasm_std::{coin, Uint128};
use cw_orch::{anyhow, deploy::Deploy, prelude::*};
use wyndex_bundle::{WynDex, EUR, USD, WYNDEX as WYNDEX_WITHOUT_CHAIN};

// consts for testing
const ADMIN: &str = "admin";
const DENOM: &str = "TOKEN";

#[allow(unused)]
struct DeployedApps {
    challenge_app: ChallengeApp<Mock>,
}

#[allow(unused)]
struct CronCatAddrs {
    factory: Addr,
    manager: Addr,
    tasks: Addr,
    agents: Addr,
}

#[allow(clippy::type_complexity)]
fn setup() -> anyhow::Result<(Mock, AbstractAccount<Mock>, Abstract<Mock>, DeployedApps)> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);
    let mut challenge_app = ChallengeApp::new(CHALLENGE_APP_ID, mock.clone());

    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), sender.to_string())?;
    let account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            })?;

    let _ = account.install_module(
        CHALLENGE_APP_ID,
        &InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: abstr_deployment.ans_host.addr_str()?,
            },
            module: AppInstantiateMsg {
                native_asset: AssetEntry::new("denom"),
                forfeit_amount: Uint128::new(42),
            },
        },
        None,
    );

    let module_addr = account
        .manager
        .module_info(CHALLENGE_APP_ID)?
        .unwrap()
        .address;

    challenge_app.set_address(&module_addr);
    challenge_app.set_sender(&account.manager.address()?);
    mock.set_balance(
        &account.proxy.address()?,
        vec![coin(50_000_000, DENOM), coin(10_000, "eur")],
    )?;

    let deployed = DeployedApps { challenge_app };
    Ok((mock, account, abstr_deployment, deployed))
}
