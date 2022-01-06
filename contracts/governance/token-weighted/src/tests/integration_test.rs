#![cfg(test)]

use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{to_binary, Addr, BlockInfo, Decimal, Empty, Timestamp, Uint128};
use cw_multi_test::{App, BankKeeper, Contract, ContractWrapper, Executor};

use crate::msg::{ExecuteMsg, InstantiateMsg};

use crate::state::{Cw20HookMsg, PollExecuteMsg, VoteOption};
use crate::tests::common::{
    DEFAULT_EXPIRATION_PERIOD, DEFAULT_FIX_PERIOD, DEFAULT_QUORUM, DEFAULT_THRESHOLD,
    DEFAULT_TIMELOCK_PERIOD, DEFAULT_VOTING_PERIOD,
};

use crate::tests::tswap_mock::{contract_receiver_mock, MockInstantiateMsg};
use stablecoin_vault::contract::{execute, instantiate, query, reply};
use stablecoin_vault::pool_info::PoolInfo;
use terraswap::asset::AssetInfo;
use dao_os::ust_vault::msg::InstantiateMsg as VaultInstantiateMsg;

use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg};

// Custom Vault Instant msg func which takes code ID
pub fn instantiate_msg(token_code_id: u64) -> VaultInstantiateMsg {
    VaultInstantiateMsg {
        anchor_money_market_address: "test_mm".to_string(),
        aust_address: "test_aust".to_string(),
        profit_check_address: "test_profit_check".to_string(),
        treasury_addr: "treasury".to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        token_code_id: token_code_id,
        treasury_fee: Decimal::percent(10u64),
        flash_loan_fee: Decimal::permille(5u64),
        commission_fee: Decimal::permille(8u64),
        stable_cap: Uint128::from(100_000_000u64),
        vault_lp_token_name: None,
        vault_lp_token_symbol: None,
    }
}

pub fn contract_whale_token() -> Box<dyn Contract<Empty>> {
    // Instantiate WHALE Token Contract
    let whale_token_contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(whale_token_contract)
}

pub fn contract_gov() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn contract_stablecoin_vault() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(execute, instantiate, query).with_reply(reply);
    Box::new(contract)
}

pub fn mock_app() -> App<Empty> {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();

    App::new(api, env.block, bank, MockStorage::new())
}

#[test]
// setup all contracts needed, and attempt to simply change the stable_cap AS-THE governance contract
// this test attempts to send some WHALE token to a named address on creation
// the gov_staker address then attempts to stake some tokens by sending a Cw20ExecuteMsg which contains a Cw20HookMsg for the gov contract
// the gov_staker address then attempts to create a poll via the same method. The Poll contains the dao_os::ust_vault::msg::ExecuteMsg::SetStableCap message
// the gov_staker casts a Yes vote
// Time passing is simulated
// Poll is ended and then executed
// Verification is done to ensure the proposed change in the gov vote has been performed
fn gov_can_update_the_stable_cap_parameter_of_vault() {
    // Define the value to set in the vault through gov vote
    let new_stable_cap_value = 900_000_000u64;
    // Create the owner account
    let owner = Addr::unchecked("owner");
    // Create the gov staker account
    let gov_staker = Addr::unchecked("gov_staker");
    // Define a mock_app to be used for storing code and instantiating
    let mut router = mock_app();
    // Store the stablecoin vault as a code object
    let vault_id = router.store_code(contract_stablecoin_vault());
    // Store the gov contract as a code object
    let gov_id = router.store_code(contract_gov());

    // Set the block height and time, we will later modify this to simulate time passing
    let initial_block = BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(1000),
        chain_id: "terra-cosmwasm-testnet".to_string(),
    };
    router.set_block(initial_block);
    // Lastly, store our terrswap mock which is a slimmed down Terraswap with no real functionality
    let terraswap_id = router.store_code(contract_receiver_mock());

    // First prepare an InstantiateMsg for vault contract with the mock terraswap token_code_id
    let vault_msg = instantiate_msg(terraswap_id);
    // Next prepare the Gov contract InstantiateMsg
    let gov_msg = InstantiateMsg {
        quorum: Decimal::percent(DEFAULT_QUORUM),
        threshold: Decimal::percent(DEFAULT_THRESHOLD),
        voting_period: DEFAULT_VOTING_PERIOD,
        timelock_period: DEFAULT_TIMELOCK_PERIOD,
        expiration_period: DEFAULT_EXPIRATION_PERIOD,
        proposal_deposit: Uint128::new(1),
        snapshot_period: DEFAULT_FIX_PERIOD,
    };

    // Store whale token which is a CW20 and get its code ID
    let whale_token_id = router.store_code(contract_whale_token());

    // Create the Whale token giving gov_staker some initial balance
    let msg = cw20_base::msg::InstantiateMsg {
        name: "White Whale".to_string(),
        symbol: "WHALE".to_string(),
        decimals: 2,
        initial_balances: vec![Cw20Coin {
            address: gov_staker.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    let whale_token_instance = router
        .instantiate_contract(whale_token_id, owner.clone(), &msg, &[], "WHALE", None)
        .unwrap();

    // set up cw20 helpers
    let cash = Cw20Contract(whale_token_instance.clone());

    // get staker balance
    let staker_balance = cash.balance(&router, gov_staker.clone()).unwrap();
    // Verify the funds have been received
    assert_eq!(staker_balance, Uint128::new(5000));

    // Instantiate the Terraswap Mock, note this just has a simple init as we have removed everything except mocks
    let tswap_addr = router
        .instantiate_contract(
            terraswap_id,
            owner.clone(),
            &MockInstantiateMsg {},
            &[],
            "TSWAP",
            None,
        )
        .unwrap();

    // Setup the gov contract
    let gov_addr = router
        .instantiate_contract(gov_id, owner.clone(), &gov_msg, &[], "GOV", None)
        .unwrap();

    // Next setup the vault with the gov contract as the 'owner'
    let vault_addr = router
        .instantiate_contract(
            vault_id,
            owner.clone(),
            &vault_msg,
            &[],
            "VAULT",
            Some(owner.to_string()),
        )
        .unwrap();
    // Ensure addresses are not equal to each other
    assert_ne!(gov_addr, vault_addr);
    assert_ne!(vault_addr, tswap_addr);
    assert_ne!(gov_addr, tswap_addr);

    // Pass ownership of vault to gov
    let transfer_owner_msg = dao_os::ust_vault::msg::ExecuteMsg::SetAdmin {
        admin: gov_addr.to_string(),
    };
    let _ = router
        .execute_contract(owner.clone(), vault_addr.clone(), &transfer_owner_msg, &[])
        .unwrap();

    // Register our whale token as the gov token
    let msg = ExecuteMsg::RegisterContracts {
        whale_token: whale_token_instance.to_string(),
    };
    let _ = router
        .execute_contract(gov_staker.clone(), gov_addr.clone(), &msg, &[])
        .unwrap();

    // TODO: maybe a check here later

    // Define the stake voting tokens msg and wrap it in a Cw20ExecuteMsg
    let msg = Cw20HookMsg::StakeVotingTokens {};

    // Prepare cw20 message with our attempt to stake tokens
    let send_msg = Cw20ExecuteMsg::Send {
        contract: gov_addr.to_string(),
        amount: Uint128::new(1000),
        msg: to_binary(&msg).unwrap(),
    };
    let _ = router
        .execute_contract(
            gov_staker.clone(),
            whale_token_instance.clone(),
            &send_msg,
            &[],
        )
        .unwrap();

    // Get the current stable_cap to later compare with
    let config_msg = dao_os::ust_vault::msg::VaultQueryMsg::PoolConfig {};
    let pool_response: PoolInfo = router
        .wrap()
        .query_wasm_smart(vault_addr.clone(), &config_msg)
        .unwrap();
    let original_stable_cap: Uint128 = pool_response.stable_cap;

    // TODO: Improve such that a Poll is created with the Gov contract and the Poll contains a message to
    // change the slippage param on the vault
    // This would be the proper way to update it as it is not expected
    let stable_cap_change_msg = to_binary(&dao_os::ust_vault::msg::ExecuteMsg::SetStableCap {
        stable_cap: Uint128::from(new_stable_cap_value),
    })
    .unwrap();

    // push two execute msgs to the list
    let execute_msgs: Vec<PollExecuteMsg> = vec![PollExecuteMsg {
        order: 1u64,
        contract: vault_addr.to_string(),
        msg: stable_cap_change_msg,
    }];

    // Define the create poll msg and wrap it in a Cw20ExecuteMsg
    let create_msg = Cw20HookMsg::CreatePoll {
        title: "test".to_string(),
        description: "test".to_string(),
        link: None,
        execute_msgs: Some(execute_msgs.clone()),
    };
    let send_msg = Cw20ExecuteMsg::Send {
        contract: gov_addr.to_string(),
        amount: Uint128::new(4000),
        msg: to_binary(&create_msg).unwrap(),
    };
    let res = router
        .execute_contract(
            gov_staker.clone(),
            whale_token_instance.clone(),
            &send_msg,
            &[],
        )
        .unwrap();

    println!("{:?}", res.events);

    // Get gov staker to vote yes
    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::new(1000),
    };
    let _ = router
        .execute_contract(gov_staker.clone(), gov_addr.clone(), &msg, &[])
        .unwrap();

    // Now simulate passing of time
    // Set the block height and time, we will later modify this to simulate time passing
    let new_block = BlockInfo {
        height: DEFAULT_VOTING_PERIOD + DEFAULT_TIMELOCK_PERIOD + 1,
        time: Timestamp::from_seconds(DEFAULT_VOTING_PERIOD + DEFAULT_TIMELOCK_PERIOD + 1),
        chain_id: "terra-cosmwasm-testnet".to_string(),
    };
    router.set_block(new_block);

    // End poll
    let msg = ExecuteMsg::EndPoll { poll_id: 1 };
    let _ = router
        .execute_contract(gov_addr.clone(), gov_addr.clone(), &msg, &[])
        .unwrap();

    // Then execute
    let msg = ExecuteMsg::ExecutePoll { poll_id: 1 };
    let _ = router
        .execute_contract(owner.clone(), gov_addr.clone(), &msg, &[])
        .unwrap();

    // Get the new stable_cap
    let config_msg = dao_os::ust_vault::msg::VaultQueryMsg::PoolConfig {};
    let pool_response: PoolInfo = router
        .wrap()
        .query_wasm_smart(vault_addr.clone(), &config_msg)
        .unwrap();
    let new_stable_cap: Uint128 = pool_response.stable_cap;
    // Ensure the stable cap has been updated to a new value
    assert_ne!(
        original_stable_cap, new_stable_cap,
        "The original stable cap logged before gov proposal is the same as the new stable cap"
    );
    assert_eq!(
        new_stable_cap,
        Uint128::from(new_stable_cap_value),
        "The new stable cap is not set to the expected value"
    )
}

// Can set fee
#[test]
fn gov_can_set_fees_for_vault() {}
// Can add and remove from/to whitelist
#[test]
fn gov_can_whitelist_address_and_remove_addr_from_whitelist_through_polls() {}
// gov can update state
#[test]
fn gov_can_update_vault_config_through_polls() {
    // Create the owner account
    let owner = Addr::unchecked("owner");
    // Create the gov staker account
    let gov_staker = Addr::unchecked("gov_staker");
    // Define a mock_app to be used for storing code and instantiating
    let mut router = mock_app();
    // Store the stablecoin vault as a code object
    let vault_id = router.store_code(contract_stablecoin_vault());
    // Store the gov contract as a code object
    let gov_id = router.store_code(contract_gov());

    // Set the block height and time, we will later modify this to simulate time passing
    let initial_block = BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(1000),
        chain_id: "terra-cosmwasm-testnet".to_string(),
    };
    router.set_block(initial_block);
    // Lastly, store our terrswap mock which is a slimmed down Terraswap with no real functionality
    let terraswap_id = router.store_code(contract_receiver_mock());

    // First prepare an InstantiateMsg for vault contract with the mock terraswap token_code_id
    let vault_msg = instantiate_msg(terraswap_id);
    // Next prepare the Gov contract InstantiateMsg
    let gov_msg = InstantiateMsg {
        quorum: Decimal::percent(DEFAULT_QUORUM),
        threshold: Decimal::percent(DEFAULT_THRESHOLD),
        voting_period: DEFAULT_VOTING_PERIOD,
        timelock_period: DEFAULT_TIMELOCK_PERIOD,
        expiration_period: DEFAULT_EXPIRATION_PERIOD,
        proposal_deposit: Uint128::new(1),
        snapshot_period: DEFAULT_FIX_PERIOD,
    };

    // Store whale token which is a CW20 and get its code ID
    let whale_token_id = router.store_code(contract_whale_token());

    // Create the Whale token giving gov_staker some initial balance
    let msg = cw20_base::msg::InstantiateMsg {
        name: "White Whale".to_string(),
        symbol: "WHALE".to_string(),
        decimals: 2,
        initial_balances: vec![Cw20Coin {
            address: gov_staker.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    let whale_token_instance = router
        .instantiate_contract(whale_token_id, owner.clone(), &msg, &[], "WHALE", None)
        .unwrap();

    // set up cw20 helpers
    let cash = Cw20Contract(whale_token_instance.clone());

    // get staker balance
    let staker_balance = cash.balance(&router, gov_staker.clone()).unwrap();
    // Verify the funds have been received
    assert_eq!(staker_balance, Uint128::new(5000));

    // Instantiate the Terraswap Mock, note this just has a simple init as we have removed everything except mocks
    let tswap_addr = router
        .instantiate_contract(
            terraswap_id,
            owner.clone(),
            &MockInstantiateMsg {},
            &[],
            "TSWAP",
            None,
        )
        .unwrap();

    // Setup the gov contract
    let gov_addr = router
        .instantiate_contract(gov_id, owner.clone(), &gov_msg, &[], "GOV", None)
        .unwrap();

    // Next setup the vault with the gov contract as the 'owner'
    let vault_addr = router
        .instantiate_contract(
            vault_id,
            owner.clone(),
            &vault_msg,
            &[],
            "VAULT",
            Some(owner.to_string()),
        )
        .unwrap();
    // Ensure addresses are not equal to each other
    assert_ne!(gov_addr, vault_addr);
    assert_ne!(vault_addr, tswap_addr);
    assert_ne!(gov_addr, tswap_addr);

    // Pass ownership of vault to gov
    let transfer_owner_msg = dao_os::ust_vault::msg::ExecuteMsg::SetAdmin {
        admin: gov_addr.to_string(),
    };
    let _ = router
        .execute_contract(owner.clone(), vault_addr.clone(), &transfer_owner_msg, &[])
        .unwrap();

    // Register our whale token as the gov token
    let msg = ExecuteMsg::RegisterContracts {
        whale_token: whale_token_instance.to_string(),
    };
    let _ = router
        .execute_contract(gov_staker.clone(), gov_addr.clone(), &msg, &[])
        .unwrap();

    // TODO: maybe a check here later

    // Define the stake voting tokens msg and wrap it in a Cw20ExecuteMsg
    let msg = Cw20HookMsg::StakeVotingTokens {};

    // Prepare cw20 message with our attempt to stake tokens
    let send_msg = Cw20ExecuteMsg::Send {
        contract: gov_addr.to_string(),
        amount: Uint128::new(1000),
        msg: to_binary(&msg).unwrap(),
    };
    let _ = router
        .execute_contract(
            gov_staker.clone(),
            whale_token_instance.clone(),
            &send_msg,
            &[],
        )
        .unwrap();

    // Get the current stable_cap to later compare with
    let config_msg = dao_os::ust_vault::msg::VaultQueryMsg::State {};

    let state_response: dao_os::ust_vault::msg::StateResponse = router
        .wrap()
        .query_wasm_smart(vault_addr.clone(), &config_msg)
        .unwrap();
    let original_profit_check_addr: String = state_response.profit_check_address;

    let stable_cap_change_msg = to_binary(&dao_os::ust_vault::msg::ExecuteMsg::UpdateState {
        anchor_money_market_address: Some("market_addr".to_string()),
        aust_address: Some("aust".to_string()),
        profit_check_address: Some("profit".to_string()),
        allow_non_whitelisted: Some(false),
    })
    .unwrap();

    // push two execute msgs to the list
    let execute_msgs: Vec<PollExecuteMsg> = vec![PollExecuteMsg {
        order: 1u64,
        contract: vault_addr.to_string(),
        msg: stable_cap_change_msg,
    }];

    // Define the create poll msg and wrap it in a Cw20ExecuteMsg
    let create_msg = Cw20HookMsg::CreatePoll {
        title: "test".to_string(),
        description: "test".to_string(),
        link: None,
        execute_msgs: Some(execute_msgs.clone()),
    };
    let send_msg = Cw20ExecuteMsg::Send {
        contract: gov_addr.to_string(),
        amount: Uint128::new(4000),
        msg: to_binary(&create_msg).unwrap(),
    };
    let res = router
        .execute_contract(
            gov_staker.clone(),
            whale_token_instance.clone(),
            &send_msg,
            &[],
        )
        .unwrap();

    println!("{:?}", res.events);

    // Get gov staker to vote yes
    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::new(1000),
    };
    let _ = router
        .execute_contract(gov_staker.clone(), gov_addr.clone(), &msg, &[])
        .unwrap();

    // Now simulate passing of time
    // Set the block height and time, we will later modify this to simulate time passing
    let new_block = BlockInfo {
        height: DEFAULT_VOTING_PERIOD + DEFAULT_TIMELOCK_PERIOD + 1,
        time: Timestamp::from_seconds(DEFAULT_VOTING_PERIOD + DEFAULT_TIMELOCK_PERIOD + 1),
        chain_id: "terra-cosmwasm-testnet".to_string(),
    };
    router.set_block(new_block);

    // End poll
    let msg = ExecuteMsg::EndPoll { poll_id: 1 };
    let _ = router
        .execute_contract(gov_addr.clone(), gov_addr.clone(), &msg, &[])
        .unwrap();

    // Then execute
    let msg = ExecuteMsg::ExecutePoll { poll_id: 1 };
    let _ = router
        .execute_contract(owner.clone(), gov_addr.clone(), &msg, &[])
        .unwrap();

    // Get the new stable_cap
    let config_msg = dao_os::ust_vault::msg::VaultQueryMsg::State {};
    let state_response: dao_os::ust_vault::msg::StateResponse = router
        .wrap()
        .query_wasm_smart(vault_addr.clone(), &config_msg)
        .unwrap();
    let new_profit_check_addr: String = state_response.profit_check_address;
    // Ensure the stable cap has been updated to a new value
    assert_ne!(
        original_profit_check_addr, new_profit_check_addr,
        "The original stable cap logged before gov proposal is the same as the new stable cap"
    );
}
