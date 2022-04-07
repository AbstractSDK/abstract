use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{attr, Addr, Timestamp, Uint128};
use cw_multi_test::{App, BankKeeper, ContractWrapper, Executor};
use pandora_os::tokenomics::airdrop::{
    ClaimResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse,
    UserInfoResponse,
};

fn mock_app() -> App {
    let api = MockApi::default();
    let env = mock_env();
    let bank = BankKeeper::new();
    let storage = MockStorage::new();

    App::new(api, env.block, bank, storage)
}

fn init_contracts(app: &mut App) -> (Addr, Addr, InstantiateMsg) {
    let owner = Addr::unchecked("contract_owner");

    // Instantiate WHALE Token Contract
    let whale_token_contract = Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));

    let whale_token_code_id = app.store_code(whale_token_contract);

    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("Whale token"),
        symbol: String::from("WHALE"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let whale_token_instance = app
        .instantiate_contract(
            whale_token_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("WHALE"),
            None,
        )
        .unwrap();

    // Instantiate Airdrop Contract
    let airdrop_contract = Box::new(ContractWrapper::new(
        whale_airdrop::contract::execute,
        whale_airdrop::contract::instantiate,
        whale_airdrop::contract::query,
    ));

    let airdrop_code_id = app.store_code(airdrop_contract);

    let aidrop_instantiate_msg = InstantiateMsg {
        owner: Some(owner.clone().to_string()),
        whale_token_address: whale_token_instance.clone().into_string(),
        merkle_roots: Some(vec!["merkle_roots".to_string()]),
        from_timestamp: Some(1_000_00),
        to_timestamp: 100_000_00,
        total_airdrop_size: Uint128::new(100_000_000_000),
    };

    // Init contract
    let airdrop_instance = app
        .instantiate_contract(
            airdrop_code_id,
            owner.clone(),
            &aidrop_instantiate_msg,
            &[],
            "airdrop",
            None,
        )
        .unwrap();

    (
        airdrop_instance,
        whale_token_instance,
        aidrop_instantiate_msg,
    )
}

fn mint_some_whale(
    app: &mut App,
    owner: Addr,
    whale_token_instance: Addr,
    amount: Uint128,
    to: String,
) {
    let msg = cw20::Cw20ExecuteMsg::Mint {
        recipient: to.clone(),
        amount: amount,
    };
    let res = app
        .execute_contract(owner.clone(), whale_token_instance.clone(), &msg, &[])
        .unwrap();
    assert_eq!(res.events[1].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[1].attributes[2], attr("to", to));
    assert_eq!(res.events[1].attributes[3], attr("amount", amount));
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let (airdrop_instance, _, init_msg) = init_contracts(&mut app);

    let resp: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&airdrop_instance, &QueryMsg::Config {})
        .unwrap();

    // Check config
    assert_eq!(init_msg.whale_token_address, resp.whale_token_address);
    assert_eq!(init_msg.owner.unwrap(), resp.owner);
    assert_eq!(init_msg.merkle_roots.unwrap(), resp.merkle_roots);
    assert_eq!(init_msg.from_timestamp.unwrap(), resp.from_timestamp);
    assert_eq!(init_msg.to_timestamp, resp.to_timestamp);

    // Check state
    let resp: StateResponse = app
        .wrap()
        .query_wasm_smart(&airdrop_instance, &QueryMsg::State {})
        .unwrap();

    assert_eq!(init_msg.total_airdrop_size, resp.total_airdrop_size);
    assert_eq!(init_msg.total_airdrop_size, resp.unclaimed_tokens);
}

#[test]
fn update_config() {
    let mut app = mock_app();
    let (airdrop_instance, _, init_msg) = init_contracts(&mut app);

    // Only owner can update
    let err = app
        .execute_contract(
            Addr::unchecked("wrong_owner"),
            airdrop_instance.clone(),
            &ExecuteMsg::UpdateConfig {
                owner: None,
                merkle_roots: None,
                from_timestamp: None,
                to_timestamp: None,
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "Generic error: Only owner can update configuration"
    );

    let new_owner = String::from("new_owner");
    let merkle_roots = vec!["new_merkle_roots".to_string()];
    let from_timestamp = 2_000_00;
    let to_timestamp = 200_000_00;

    let update_msg = ExecuteMsg::UpdateConfig {
        owner: Some(new_owner.clone()),
        merkle_roots: Some(merkle_roots.clone()),
        from_timestamp: Some(from_timestamp),
        to_timestamp: Some(to_timestamp),
    };

    // should be a success
    app.execute_contract(
        Addr::unchecked(init_msg.owner.unwrap()),
        airdrop_instance.clone(),
        &update_msg,
        &[],
    )
    .unwrap();

    let resp: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&airdrop_instance, &QueryMsg::Config {})
        .unwrap();

    // Check config and make sure all fields are updated
    assert_eq!(new_owner, resp.owner);
    assert_eq!(merkle_roots, resp.merkle_roots);
    assert_eq!(from_timestamp, resp.from_timestamp);
    assert_eq!(to_timestamp, resp.to_timestamp);
}

#[cfg(test)]
#[test]
fn test_transfer_unclaimed_tokens() {
    let mut app = mock_app();
    let (airdrop_instance, whale_instance, init_msg) = init_contracts(&mut app);

    // mint WHALE for to Airdrop Contract
    mint_some_whale(
        &mut app,
        Addr::unchecked(init_msg.owner.clone().unwrap()),
        whale_instance.clone(),
        Uint128::new(100_000_000_000),
        airdrop_instance.clone().to_string(),
    );

    // Check Airdrop Contract balance
    let bal_resp: cw20::BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            &whale_instance,
            &cw20::Cw20QueryMsg::Balance {
                address: airdrop_instance.clone().to_string(),
            },
        )
        .unwrap();
    assert_eq!(Uint128::from(100_000_000_000u64), bal_resp.balance);

    // Can only be called by the owner
    let err = app
        .execute_contract(
            Addr::unchecked("wrong_owner"),
            airdrop_instance.clone(),
            &ExecuteMsg::TransferUnclaimedTokens {
                recepient: "recepient".to_string(),
                amount: Uint128::from(1000000 as u64),
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(err.to_string(), "Generic error: Sender not authorized!");

    // claim period is not over
    app.update_block(|b| {
        b.height += 17280;
        b.time = Timestamp::from_seconds(1_000_00)
    });

    // Can only be called after the claim period is over
    let err = app
        .execute_contract(
            Addr::unchecked(init_msg.owner.clone().unwrap()),
            airdrop_instance.clone(),
            &ExecuteMsg::TransferUnclaimedTokens {
                recepient: "recepient".to_string(),
                amount: Uint128::from(1000000 as u64),
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "Generic error: 9900000 seconds left before unclaimed tokens can be transferred"
    );

    // claim period is over
    app.update_block(|b| {
        b.height += 17280;
        b.time = Timestamp::from_seconds(1_000_00_00)
    });

    // Amount needs to be less than unclaimed_tokens balance
    let err = app
        .execute_contract(
            Addr::unchecked(init_msg.owner.clone().unwrap()),
            airdrop_instance.clone(),
            &ExecuteMsg::TransferUnclaimedTokens {
                recepient: "recepient".to_string(),
                amount: Uint128::from(100_000_000_0000 as u64),
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "Generic error: Amount cannot exceed unclaimed token balance"
    );

    // Should successfully transfer and update state
    app.execute_contract(
        Addr::unchecked(init_msg.owner.clone().unwrap()),
        airdrop_instance.clone(),
        &ExecuteMsg::TransferUnclaimedTokens {
            recepient: "recepient".to_string(),
            amount: Uint128::from(100_000_00 as u64),
        },
        &[],
    )
    .unwrap();

    let state_resp: StateResponse = app
        .wrap()
        .query_wasm_smart(&airdrop_instance, &QueryMsg::State {})
        .unwrap();

    // Check config and make sure all fields are updated
    assert_eq!(
        Uint128::from(100_000_000_000u64),
        state_resp.total_airdrop_size
    );
    assert_eq!(Uint128::from(99990000000u64), state_resp.unclaimed_tokens);
}

#[cfg(test)]
#[test]
fn test_claim() {
    let mut app = mock_app();
    let (airdrop_instance, whale_instance, init_msg) = init_contracts(&mut app);

    // mint WHALE for to Airdrop Contract
    mint_some_whale(
        &mut app,
        Addr::unchecked(init_msg.owner.clone().unwrap()),
        whale_instance.clone(),
        Uint128::new(100_000_000_000),
        airdrop_instance.clone().to_string(),
    );

    // Check Airdrop Contract balance
    let bal_resp: cw20::BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            &whale_instance,
            &cw20::Cw20QueryMsg::Balance {
                address: airdrop_instance.clone().to_string(),
            },
        )
        .unwrap();
    assert_eq!(Uint128::from(100_000_000_000u64), bal_resp.balance);

    let merkle_roots =
        vec!["cdcdfad1c342f5f55a2639dcae7321a64cd000807fa24c2c4ddaa944fd52d34e".to_string()];
    let update_msg = ExecuteMsg::UpdateConfig {
        owner: None,
        merkle_roots: Some(merkle_roots.clone()),
        from_timestamp: None,
        to_timestamp: None,
    };

    // Update Config :: should be a success
    app.execute_contract(
        Addr::unchecked(init_msg.owner.clone().unwrap()),
        airdrop_instance.clone(),
        &update_msg,
        &[],
    )
    .unwrap();

    let resp: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&airdrop_instance, &QueryMsg::Config {})
        .unwrap();

    // Check config and make sure all fields are updated
    assert_eq!(init_msg.owner.clone().unwrap(), resp.owner);
    assert_eq!(merkle_roots, resp.merkle_roots);
    assert_eq!(init_msg.from_timestamp.unwrap(), resp.from_timestamp);
    assert_eq!(init_msg.to_timestamp, resp.to_timestamp);

    let claim_msg = ExecuteMsg::Claim {
        claim_amount: Uint128::from(250000000 as u64),
        merkle_proof: vec![
            "7719b79a65e5aa0bbfd144cf5373138402ab1c374d9049e490b5b61c23d90065".to_string(),
            "60368f2058e0fb961a7721a241f9b973c3dd6c57e10a627071cd81abca6aa490".to_string(),
        ],
        root_index: 0,
    };
    let claim_msg_wrong_amount = ExecuteMsg::Claim {
        claim_amount: Uint128::from(210000000 as u64),
        merkle_proof: vec![
            "7719b79a65e5aa0bbfd144cf5373138402ab1c374d9049e490b5b61c23d90065".to_string(),
            "60368f2058e0fb961a7721a241f9b973c3dd6c57e10a627071cd81abca6aa490".to_string(),
        ],
        root_index: 0,
    };
    let claim_msg_incorrect_proof = ExecuteMsg::Claim {
        claim_amount: Uint128::from(250000000 as u64),
        merkle_proof: vec![
            "7719b79a65e4aa0bbfd144cf5373138402ab1c374d9049e490b5b61c23d90065".to_string(),
            "60368f2058e0fb961a7721a241f9b973c3dd6c57e10a627071cd81abca6aa490".to_string(),
        ],
        root_index: 0,
    };

    // Claim period has not started yet
    app.update_block(|b| {
        b.height += 17280;
        b.time = Timestamp::from_seconds(1_000)
    });

    // **** "Claim not allowed" Error should be returned ****
    let claim_f = app
        .execute_contract(
            Addr::unchecked("terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp".to_string()),
            airdrop_instance.clone(),
            &ExecuteMsg::Claim {
                claim_amount: Uint128::from(250000000 as u64),
                merkle_proof: vec![
                    "7719b79a65e4aa0bbfd144cf5373138402ab1c374d9049e490b5b61c23d90065".to_string(),
                    "60368f2058e0fb961a7721a241f9b973c3dd6c57e10a627071cd81abca6aa490".to_string(),
                ],
                root_index: 5,
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(claim_f.to_string(), "Generic error: Claim not allowed");

    // Claim period has started
    app.update_block(|b| {
        b.height += 17280;
        b.time = Timestamp::from_seconds(1_000_01)
    });

    // **** "Incorrect Merkle Root Index" Error should be returned ****
    let mut claim_f = app
        .execute_contract(
            Addr::unchecked("terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp".to_string()),
            airdrop_instance.clone(),
            &ExecuteMsg::Claim {
                claim_amount: Uint128::from(250000000 as u64),
                merkle_proof: vec![
                    "7719b79a65e4aa0bbfd144cf5373138402ab1c374d9049e490b5b61c23d90065".to_string(),
                    "60368f2058e0fb961a7721a241f9b973c3dd6c57e10a627071cd81abca6aa490".to_string(),
                ],
                root_index: 5,
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        claim_f.to_string(),
        "Generic error: Incorrect Merkle Root Index"
    );

    // **** "Incorrect Merkle Proof" Error should be returned ****
    claim_f = app
        .execute_contract(
            Addr::unchecked("terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp".to_string()),
            airdrop_instance.clone(),
            &claim_msg_incorrect_proof,
            &[],
        )
        .unwrap_err();

    assert_eq!(claim_f.to_string(), "Generic error: Incorrect Merkle Proof");

    // **** "Incorrect Merkle Proof" Error should be returned ****
    claim_f = app
        .execute_contract(
            Addr::unchecked("terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp".to_string()),
            airdrop_instance.clone(),
            &claim_msg_wrong_amount,
            &[],
        )
        .unwrap_err();

    assert_eq!(claim_f.to_string(), "Generic error: Incorrect Merkle Proof");

    // **** User should successfully claim the Airdrop ****

    // Check :: User hasn't yet claimed the airdrop
    let resp: ClaimResponse = app
        .wrap()
        .query_wasm_smart(
            &airdrop_instance,
            &QueryMsg::HasUserClaimed {
                address: "terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp".to_string(),
            },
        )
        .unwrap();
    assert_eq!(false, resp.is_claimed);

    // Should be a success
    let success_ = app
        .execute_contract(
            Addr::unchecked("terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp".to_string()),
            airdrop_instance.clone(),
            &claim_msg,
            &[],
        )
        .unwrap();

    assert_eq!(success_.events[1].attributes[1], attr("action", "Claim"));
    assert_eq!(
        success_.events[1].attributes[2],
        attr("addr", "terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp")
    );
    assert_eq!(
        success_.events[1].attributes[3],
        attr("airdrop", "250000000")
    );

    // Check :: User successfully claimed the airdrop
    let claim_query_resp: ClaimResponse = app
        .wrap()
        .query_wasm_smart(
            &airdrop_instance,
            &QueryMsg::HasUserClaimed {
                address: "terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp".to_string(),
            },
        )
        .unwrap();
    assert_eq!(true, claim_query_resp.is_claimed);

    // Check :: User state
    let user_info_query_resp: UserInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &airdrop_instance,
            &QueryMsg::UserInfo {
                address: "terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        Uint128::from(250000000u64),
        user_info_query_resp.airdrop_amount
    );

    // Check :: Contract state
    let state_query_resp: StateResponse = app
        .wrap()
        .query_wasm_smart(&airdrop_instance, &QueryMsg::State {})
        .unwrap();
    assert_eq!(
        Uint128::from(100_000_000_000u64),
        state_query_resp.total_airdrop_size
    );
    assert_eq!(
        Uint128::from(99750000000u64),
        state_query_resp.unclaimed_tokens
    );

    // **** "Already claimed" Error should be returned ****

    claim_f = app
        .execute_contract(
            Addr::unchecked("terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp".to_string()),
            airdrop_instance.clone(),
            &claim_msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(claim_f.to_string(), "Generic error: Already claimed");
}
