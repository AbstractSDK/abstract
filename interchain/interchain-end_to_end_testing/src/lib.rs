use abstract_interface::{Abstract, AccountDetails, AccountI, AccountQueryFns};
use abstract_std::objects::{AccountId, AccountTrace, TruncatedChainId};
use cosmwasm_std::Coin;
use cw_orch::prelude::*;
use cw_orch_interchain::core::{IbcQueryHandler, InterchainEnv};

pub const JUNO: &str = "juno-1";
pub const STARGAZE: &str = "stargaze-1";
pub const OSMOSIS: &str = "osmosis-1";

pub const TEST_ACCOUNT_NAME: &str = "account-test";
pub const TEST_ACCOUNT_DESCRIPTION: &str = "Description of an account";
pub const TEST_ACCOUNT_LINK: &str = "https://google.com";

pub fn set_env() {
    std::env::set_var("STATE_FILE", "daemon_state.json"); // Set in code for tests
    std::env::set_var("ARTIFACTS_DIR", "../artifacts"); // Set in code for tests
}

pub fn create_test_remote_account<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
    abstr_origin: &Abstract<Chain>,
    origin_id: &str,
    remote_id: &str,
    interchain: &IBC,
    funds: Vec<Coin>,
) -> anyhow::Result<(AccountI<Chain>, AccountId)> {
    let origin_name = TruncatedChainId::from_chain_id(origin_id);
    let remote_name = TruncatedChainId::from_chain_id(remote_id);

    // Create a local account for testing
    let account_name = TEST_ACCOUNT_NAME.to_string();
    let description = Some(TEST_ACCOUNT_DESCRIPTION.to_string());
    let link = Some(TEST_ACCOUNT_LINK.to_string());
    let origin_account = AccountI::create(
        abstr_origin,
        AccountDetails {
            name: account_name.clone(),
            description: description.clone(),
            link: link.clone(),
            install_modules: vec![],
            namespace: None,
            account_id: None,
        },
        abstract_std::objects::gov_type::GovernanceDetails::Monarchy {
            monarch: abstr_origin
                .registry
                .environment()
                .sender_addr()
                .to_string(),
        },
        &funds,
    )?;

    // We need to enable ibc on the account.
    origin_account.set_ibc_status(true)?;

    // Now we send a message to the client saying that we want to create an account on the
    // host chain
    let register_tx = origin_account.register_remote_account(remote_name)?;

    interchain.await_and_check_packets(origin_id, register_tx)?;

    // After this is all ended, we return the account id of the account we just created on the remote chain
    let account_config = origin_account.config()?;
    let remote_account_id = AccountId::new(
        account_config.account_id.seq(),
        AccountTrace::Remote(vec![origin_name]),
    )?;

    Ok((origin_account, remote_account_id))
}
