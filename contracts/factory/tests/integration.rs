mod factory_helper;

use cosmwasm_std::{attr, from_slice, Addr, Decimal, StdError, Uint128};
use wyndex::asset::AssetInfo;
use wyndex::factory::{
    ConfigResponse, DefaultStakeConfig, ExecuteMsg, FeeInfoResponse, InstantiateMsg, MigrateMsg,
    PairConfig, PairType, PartialDefaultStakeConfig, QueryMsg,
};
use wyndex::fee_config::FeeConfig;
use wyndex::pair::PairInfo;
use wyndex_factory::state::Config;

use crate::factory_helper::{instantiate_token, FactoryHelper};
use cw_multi_test::{App, ContractWrapper, Executor};
use cw_placeholder::msg::InstantiateMsg as PlaceholderContractInstantiateMsg;
use wyndex::pair::ExecuteMsg as PairExecuteMsg;
fn mock_app() -> App {
    App::default()
}

fn store_placeholder_code(app: &mut App) -> u64 {
    let placeholder_contract = Box::new(ContractWrapper::new_with_empty(
        cw_placeholder::contract::execute,
        cw_placeholder::contract::instantiate,
        cw_placeholder::contract::query,
    ));

    app.store_code(placeholder_contract)
}

fn store_factory_code(app: &mut App) -> u64 {
    let factory_contract = Box::new(
        ContractWrapper::new_with_empty(
            wyndex_factory::contract::execute,
            wyndex_factory::contract::instantiate,
            wyndex_factory::contract::query,
        )
        .with_reply_empty(wyndex_factory::contract::reply)
        .with_migrate_empty(wyndex_factory::contract::migrate),
    );

    app.store_code(factory_contract)
}

fn default_stake_config() -> DefaultStakeConfig {
    DefaultStakeConfig {
        staking_code_id: 1234u64,
        tokens_per_power: Uint128::new(1000),
        min_bond: Uint128::new(1000),
        unbonding_periods: vec![1],
        max_distributions: 6,
        converter: None,
    }
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();

    let owner = Addr::unchecked("owner");

    let factory_code_id = store_factory_code(&mut app);

    let pair_configs = vec![PairConfig {
        code_id: 321,
        pair_type: PairType::Xyk {},
        fee_config: FeeConfig {
            total_fee_bps: 100,
            protocol_fee_bps: 10,
        },
        is_disabled: false,
    }];

    let msg = InstantiateMsg {
        pair_configs: pair_configs.clone(),
        token_code_id: 123,
        fee_address: None,
        owner: owner.to_string(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(),
        trading_starts: None,
    };

    let factory_instance = app
        .instantiate_contract(
            factory_code_id,
            Addr::unchecked(owner.clone()),
            &msg,
            &[],
            "factory",
            None,
        )
        .unwrap();

    let msg = QueryMsg::Config {};
    let config_res: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&factory_instance, &msg)
        .unwrap();

    assert_eq!(123, config_res.token_code_id);
    assert_eq!(pair_configs, config_res.pair_configs);
    assert_eq!(owner, config_res.owner);
}

#[test]
fn update_config() {
    let mut app = mock_app();
    let owner = Addr::unchecked("owner");
    let mut helper = FactoryHelper::init(&mut app, &owner);

    // Update config
    helper
        .update_config(
            &mut app,
            &owner,
            Some(200u64),
            Some("fee".to_string()),
            Some(false),
            Some(PartialDefaultStakeConfig {
                staking_code_id: Some(12345),
                tokens_per_power: None,
                min_bond: Some(10000u128.into()),
                unbonding_periods: None,
                max_distributions: Some(u32::MAX),
            }),
        )
        .unwrap();

    let config_res: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&helper.factory, &QueryMsg::Config {})
        .unwrap();

    assert_eq!(200u64, config_res.token_code_id);
    assert_eq!("fee", config_res.fee_address.unwrap().to_string());

    // query config raw to get default stake config
    let raw_config: Config = from_slice(
        &app.wrap()
            .query_wasm_raw(&helper.factory, "config".as_bytes())
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        DefaultStakeConfig {
            staking_code_id: 12345,
            tokens_per_power: Uint128::new(1000), // same as before
            min_bond: Uint128::new(10_000),
            unbonding_periods: vec![1, 2, 3], // same as before
            max_distributions: u32::MAX,
            converter: None,
        },
        raw_config.default_stake_config
    );

    // Unauthorized err
    let res = helper
        .update_config(
            &mut app,
            &Addr::unchecked("not_owner"),
            None,
            None,
            None,
            None,
        )
        .unwrap_err();
    assert_eq!(res.root_cause().to_string(), "Unauthorized");
}

#[test]
fn test_create_then_deregister_pair() {
    let mut app = mock_app();
    let owner = Addr::unchecked("owner");
    let mut helper = FactoryHelper::init(&mut app, &owner);

    let token1 = instantiate_token(
        &mut app,
        helper.cw20_token_code_id,
        &owner,
        "tokenX",
        Some(18),
    );
    let token2 = instantiate_token(
        &mut app,
        helper.cw20_token_code_id,
        &owner,
        "tokenY",
        Some(18),
    );
    // Create the pair which we will later delete
    let res = helper
        .create_pair(
            &mut app,
            &owner,
            PairType::Xyk {},
            [token1.as_str(), token2.as_str()],
            None,
            None,
        )
        .unwrap();

    assert_eq!(res.events[1].attributes[1], attr("action", "create_pair"));
    assert_eq!(
        res.events[1].attributes[2],
        attr("pair", format!("{}-{}", token1.as_str(), token2.as_str()))
    );
    // Verify the pair now exists
    let res: PairInfo = app
        .wrap()
        .query_wasm_smart(
            helper.factory.clone(),
            &QueryMsg::Pair {
                asset_infos: vec![
                    AssetInfo::Token(token1.to_string()),
                    AssetInfo::Token(token2.to_string()),
                ],
            },
        )
        .unwrap();

    // In multitest, contract names are counted in the order in which contracts are created
    assert_eq!("contract1", helper.factory.to_string());
    assert_eq!("contract4", res.contract_addr.to_string());
    assert_eq!("contract5", res.liquidity_token.to_string());
    // Deregsiter the pair, which removes the Pair addr and the staking contract addr from Storage
    helper
        .deregister_pool_and_staking(
            &mut app,
            &owner,
            vec![
                AssetInfo::Token(token1.to_string()),
                AssetInfo::Token(token2.to_string()),
            ],
        )
        .unwrap();

    // Verify the pair no longer exists
    let err: Result<PairInfo, StdError> = app.wrap().query_wasm_smart(
        helper.factory.clone(),
        &QueryMsg::Pair {
            asset_infos: vec![
                AssetInfo::Token(token1.to_string()),
                AssetInfo::Token(token2.to_string()),
            ],
        },
    );

    // In multitest, contract names are counted in the order in which contracts are created
    assert_eq!(
        err.unwrap_err(),
        StdError::generic_err("Querier contract error: cosmwasm_std::addresses::Addr not found")
    );
}

#[test]
fn test_valid_staking() {
    let mut app = mock_app();
    let owner = Addr::unchecked("owner");
    let mut helper = FactoryHelper::init(&mut app, &owner);

    let token1 = instantiate_token(
        &mut app,
        helper.cw20_token_code_id,
        &owner,
        "tokenX",
        Some(18),
    );
    let token2 = instantiate_token(
        &mut app,
        helper.cw20_token_code_id,
        &owner,
        "tokenY",
        Some(18),
    );

    // Verify the pair now exists, we don't need to check the bool result here as non existence returns an Error
    let is_valid: bool = app
        .wrap()
        .query_wasm_smart(
            helper.factory.clone(),
            &QueryMsg::ValidateStakingAddress {
                address: "contract6".to_string(),
            },
        )
        .unwrap();

    assert!(!is_valid);
    // Create the pair which we will later delete
    let res = helper
        .create_pair(
            &mut app,
            &owner,
            PairType::Xyk {},
            [token1.as_str(), token2.as_str()],
            None,
            None,
        )
        .unwrap();

    assert_eq!(res.events[1].attributes[1], attr("action", "create_pair"));
    assert_eq!(
        res.events[1].attributes[2],
        attr("pair", format!("{}-{}", token1.as_str(), token2.as_str()))
    );

    // Verify the pair now exists, we don't need to check the bool result here as non existence returns an Error
    let is_valid: bool = app
        .wrap()
        .query_wasm_smart(
            helper.factory.clone(),
            &QueryMsg::ValidateStakingAddress {
                address: "contract6".to_string(),
            },
        )
        .unwrap();
    assert!(is_valid);
    // Deregsiter the pair, which removes the Pair addr and the staking contract addr from Storage
    helper
        .deregister_pool_and_staking(
            &mut app,
            &owner,
            vec![
                AssetInfo::Token(token1.to_string()),
                AssetInfo::Token(token2.to_string()),
            ],
        )
        .unwrap();

    let is_valid: bool = app
        .wrap()
        .query_wasm_smart(
            helper.factory.clone(),
            &QueryMsg::ValidateStakingAddress {
                address: "contract6".to_string(),
            },
        )
        .unwrap();

    assert!(!is_valid);
}

#[test]
fn test_create_pair() {
    let mut app = mock_app();
    let owner = Addr::unchecked("owner");
    let mut helper = FactoryHelper::init(&mut app, &owner);

    let token1 = instantiate_token(
        &mut app,
        helper.cw20_token_code_id,
        &owner,
        "tokenX",
        Some(18),
    );
    let token2 = instantiate_token(
        &mut app,
        helper.cw20_token_code_id,
        &owner,
        "tokenY",
        Some(18),
    );

    let err = helper
        .create_pair(
            &mut app,
            &owner,
            PairType::Xyk {},
            [token1.as_str(), token1.as_str()],
            None,
            None,
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Doubling assets in asset infos"
    );

    let res = helper
        .create_pair(
            &mut app,
            &owner,
            PairType::Xyk {},
            [token1.as_str(), token2.as_str()],
            None,
            None,
        )
        .unwrap();

    let err = helper
        .create_pair(
            &mut app,
            &owner,
            PairType::Xyk {},
            [token1.as_str(), token2.as_str()],
            None,
            None,
        )
        .unwrap_err();
    assert_eq!(err.root_cause().to_string(), "Pair was already created");

    assert_eq!(res.events[1].attributes[1], attr("action", "create_pair"));
    assert_eq!(
        res.events[1].attributes[2],
        attr("pair", format!("{}-{}", token1.as_str(), token2.as_str()))
    );

    let res: PairInfo = app
        .wrap()
        .query_wasm_smart(
            helper.factory.clone(),
            &QueryMsg::Pair {
                asset_infos: vec![
                    AssetInfo::Token(token1.to_string()),
                    AssetInfo::Token(token2.to_string()),
                ],
            },
        )
        .unwrap();

    // In multitest, contract names are counted in the order in which contracts are created
    assert_eq!("contract1", helper.factory.to_string());
    assert_eq!("contract4", res.contract_addr.to_string());
    assert_eq!("contract5", res.liquidity_token.to_string());

    // Create disabled pair type
    app.execute_contract(
        owner.clone(),
        helper.factory.clone(),
        &ExecuteMsg::UpdatePairConfig {
            config: PairConfig {
                code_id: 0,
                pair_type: PairType::Custom("Custom".to_string()),
                fee_config: FeeConfig {
                    total_fee_bps: 100,
                    protocol_fee_bps: 40,
                },
                is_disabled: true,
            },
        },
        &[],
    )
    .unwrap();

    let token3 = instantiate_token(
        &mut app,
        helper.cw20_token_code_id,
        &owner,
        "tokenY",
        Some(18),
    );

    let err = helper
        .create_pair(
            &mut app,
            &owner,
            PairType::Custom("Custom".to_string()),
            [token1.as_str(), token3.as_str()],
            None,
            None,
        )
        .unwrap_err();
    assert_eq!(err.root_cause().to_string(), "Pair config disabled");

    // Query fee info
    let fee_info: FeeInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &helper.factory,
            &QueryMsg::FeeInfo {
                pair_type: PairType::Custom("Custom".to_string()),
            },
        )
        .unwrap();
    assert_eq!(100, fee_info.total_fee_bps);
    assert_eq!(40, fee_info.protocol_fee_bps);

    // query blacklisted pairs
    let pair_types: Vec<PairType> = app
        .wrap()
        .query_wasm_smart(&helper.factory, &QueryMsg::BlacklistedPairTypes {})
        .unwrap();
    assert_eq!(pair_types, vec![PairType::Custom("Custom".to_string())]);
}

#[test]
fn test_create_pair_permissions() {
    let mut app = mock_app();
    let owner = Addr::unchecked("owner");
    let user = Addr::unchecked("user");
    let mut helper = FactoryHelper::init(&mut app, &owner);

    let token1 = instantiate_token(
        &mut app,
        helper.cw20_token_code_id,
        &owner,
        "tokenX",
        Some(18),
    );
    let token2 = instantiate_token(
        &mut app,
        helper.cw20_token_code_id,
        &owner,
        "tokenY",
        Some(18),
    );

    let err = helper
        .create_pair(
            &mut app,
            &user,
            PairType::Xyk {},
            [token1.as_str(), token2.as_str()],
            None,
            None,
        )
        .unwrap_err();
    assert_eq!(err.root_cause().to_string(), "Unauthorized");

    // allow anyone to create pair
    helper
        .update_config(&mut app, &owner, None, None, Some(false), None)
        .unwrap();

    // now it should work
    helper
        .create_pair(
            &mut app,
            &user,
            PairType::Xyk {},
            [token1.as_str(), token2.as_str()],
            None,
            None,
        )
        .unwrap();
}

#[test]
fn test_update_pair_fee() {
    let mut app = mock_app();
    let owner = Addr::unchecked("owner");
    let mut helper = FactoryHelper::init(&mut app, &owner);

    let token1 = instantiate_token(
        &mut app,
        helper.cw20_token_code_id,
        &owner,
        "tokenX",
        Some(18),
    );
    let token2 = instantiate_token(
        &mut app,
        helper.cw20_token_code_id,
        &owner,
        "tokenY",
        Some(18),
    );

    helper
        .create_pair(
            &mut app,
            &owner,
            PairType::Xyk {},
            [token1.as_str(), token2.as_str()],
            None,
            None,
        )
        .unwrap();

    let asset_infos = vec![
        AssetInfo::Native(token1.to_string()),
        AssetInfo::Native(token2.to_string()),
    ];
    // query current fee
    let pair_res: PairInfo = app
        .wrap()
        .query_wasm_smart(
            &helper.factory,
            &QueryMsg::Pair {
                asset_infos: asset_infos.clone(),
            },
        )
        .unwrap();
    assert_eq!(
        pair_res.fee_config,
        FeeConfig {
            total_fee_bps: 100,
            protocol_fee_bps: 10
        }
    );

    // change fees
    helper
        .update_pair_fees(
            &mut app,
            &owner,
            asset_infos.clone(),
            FeeConfig {
                total_fee_bps: 1000,
                protocol_fee_bps: 10,
            },
        )
        .unwrap();
    // query updated fee
    let pair_res: PairInfo = app
        .wrap()
        .query_wasm_smart(&helper.factory, &QueryMsg::Pair { asset_infos })
        .unwrap();
    assert_eq!(
        pair_res.fee_config,
        FeeConfig {
            total_fee_bps: 1000,
            protocol_fee_bps: 10
        }
    );
}

#[test]
fn test_pair_migration() {
    let mut app = mock_app();

    let owner = Addr::unchecked("owner");
    let mut helper = FactoryHelper::init(&mut app, &owner);

    let token_instance0 =
        instantiate_token(&mut app, helper.cw20_token_code_id, &owner, "tokenX", None);
    let token_instance1 =
        instantiate_token(&mut app, helper.cw20_token_code_id, &owner, "tokenY", None);
    let token_instance2 =
        instantiate_token(&mut app, helper.cw20_token_code_id, &owner, "tokenZ", None);

    // Create pairs in factory
    let pairs = [
        helper
            .create_pair_with_addr(
                &mut app,
                &owner,
                PairType::Xyk {},
                [token_instance0.as_str(), token_instance1.as_str()],
                None,
            )
            .unwrap(),
        helper
            .create_pair_with_addr(
                &mut app,
                &owner,
                PairType::Xyk {},
                [token_instance0.as_str(), token_instance2.as_str()],
                None,
            )
            .unwrap(),
    ];

    // Change contract ownership
    let new_owner = Addr::unchecked("new_owner");

    app.execute_contract(
        owner.clone(),
        helper.factory.clone(),
        &ExecuteMsg::ProposeNewOwner {
            owner: new_owner.to_string(),
            expires_in: 100,
        },
        &[],
    )
    .unwrap();
    app.execute_contract(
        new_owner.clone(),
        helper.factory.clone(),
        &ExecuteMsg::ClaimOwnership {},
        &[],
    )
    .unwrap();

    let pair3 = helper
        .create_pair_with_addr(
            &mut app,
            &new_owner,
            PairType::Xyk {},
            [token_instance1.as_str(), token_instance2.as_str()],
            None,
        )
        .unwrap();

    // Should panic due to pairs are not migrated.
    for pair in pairs.clone() {
        let res = app
            .execute_contract(
                new_owner.clone(),
                pair,
                &PairExecuteMsg::UpdateConfig {
                    params: Default::default(),
                },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            res.root_cause().to_string(),
            "Pair is not migrated to the new admin!"
        );
    }

    // Pair is created after admin migration
    let res = app
        .execute_contract(
            Addr::unchecked("user1"),
            pair3,
            &PairExecuteMsg::UpdateConfig {
                params: Default::default(),
            },
            &[],
        )
        .unwrap_err();

    assert_ne!(res.to_string(), "Pair is not migrated to the new admin");

    let pairs_res: Vec<Addr> = app
        .wrap()
        .query_wasm_smart(&helper.factory, &QueryMsg::PairsToMigrate {})
        .unwrap();
    assert_eq!(&pairs_res, &pairs);

    // Factory owner was changed to new owner
    let err = app
        .execute_contract(
            owner,
            helper.factory.clone(),
            &ExecuteMsg::MarkAsMigrated {
                pairs: Vec::from(pairs.clone().map(String::from)),
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(err.root_cause().to_string(), "Unauthorized");

    app.execute_contract(
        new_owner,
        helper.factory.clone(),
        &ExecuteMsg::MarkAsMigrated {
            pairs: Vec::from(pairs.clone().map(String::from)),
        },
        &[],
    )
    .unwrap();

    for pair in pairs {
        let res = app
            .execute_contract(
                Addr::unchecked("user1"),
                pair,
                &PairExecuteMsg::UpdateConfig {
                    params: Default::default(),
                },
                &[],
            )
            .unwrap_err();

        assert_ne!(res.to_string(), "Pair is not migrated to the new admin!");
    }
}

#[test]
fn check_update_owner() {
    let mut app = mock_app();
    let owner = Addr::unchecked("owner");
    let helper = FactoryHelper::init(&mut app, &owner);

    let new_owner = String::from("new_owner");

    // New owner
    let msg = ExecuteMsg::ProposeNewOwner {
        owner: new_owner.clone(),
        expires_in: 100, // seconds
    };

    // Unauthed check
    let err = app
        .execute_contract(
            Addr::unchecked("not_owner"),
            helper.factory.clone(),
            &msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(err.root_cause().to_string(), "Generic error: Unauthorized");

    // Claim before proposal
    let err = app
        .execute_contract(
            Addr::unchecked(new_owner.clone()),
            helper.factory.clone(),
            &ExecuteMsg::ClaimOwnership {},
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Generic error: Ownership proposal not found"
    );

    // Propose new owner
    app.execute_contract(Addr::unchecked("owner"), helper.factory.clone(), &msg, &[])
        .unwrap();

    // Claim from invalid addr
    let err = app
        .execute_contract(
            Addr::unchecked("invalid_addr"),
            helper.factory.clone(),
            &ExecuteMsg::ClaimOwnership {},
            &[],
        )
        .unwrap_err();
    assert_eq!(err.root_cause().to_string(), "Generic error: Unauthorized");

    // Drop ownership proposal
    let err = app
        .execute_contract(
            Addr::unchecked(new_owner.clone()),
            helper.factory.clone(),
            &ExecuteMsg::DropOwnershipProposal {},
            &[],
        )
        .unwrap_err();
    // new_owner is not an owner yet
    assert_eq!(err.root_cause().to_string(), "Generic error: Unauthorized");

    app.execute_contract(
        owner.clone(),
        helper.factory.clone(),
        &ExecuteMsg::DropOwnershipProposal {},
        &[],
    )
    .unwrap();

    // Try to claim ownership
    let err = app
        .execute_contract(
            Addr::unchecked(new_owner.clone()),
            helper.factory.clone(),
            &ExecuteMsg::ClaimOwnership {},
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Generic error: Ownership proposal not found"
    );

    // Propose new owner again
    app.execute_contract(Addr::unchecked("owner"), helper.factory.clone(), &msg, &[])
        .unwrap();
    // Claim ownership
    app.execute_contract(
        Addr::unchecked(new_owner.clone()),
        helper.factory.clone(),
        &ExecuteMsg::ClaimOwnership {},
        &[],
    )
    .unwrap();

    // Let's query the contract state
    let msg = QueryMsg::Config {};
    let res: ConfigResponse = app.wrap().query_wasm_smart(&helper.factory, &msg).unwrap();

    assert_eq!(res.owner, new_owner)
}

#[test]
fn can_migrate_the_placeholder_to_a_factory_properly() {
    let mut app = mock_app();

    let owner = Addr::unchecked("owner");

    let place_holder_id = store_placeholder_code(&mut app);
    let factory_id = store_factory_code(&mut app);

    let pair_configs = vec![PairConfig {
        code_id: 321,
        pair_type: PairType::Xyk {},
        fee_config: FeeConfig {
            total_fee_bps: 100,
            protocol_fee_bps: 10,
        },
        is_disabled: false,
    }];
    // Instantiate an instance of the placeholder contract which we will migrate
    let placeholder = app
        .instantiate_contract(
            place_holder_id,
            owner.clone(),
            &PlaceholderContractInstantiateMsg {},
            &[],
            "placeholder",
            Some(owner.clone().into_string()),
        )
        .unwrap();

    let factory_msg = InstantiateMsg {
        pair_configs: pair_configs.clone(),
        token_code_id: 123,
        fee_address: None,
        owner: owner.to_string(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(),
        trading_starts: None,
    };
    // Migrate the contract
    app.migrate_contract(
        owner.clone(),
        placeholder.clone(),
        &MigrateMsg::Init(factory_msg.clone()),
        factory_id,
    )
    .unwrap();

    // Now instantiate a normal factory directly
    let factory_instance = app
        .instantiate_contract(
            factory_id,
            Addr::unchecked(owner.clone()),
            &factory_msg,
            &[],
            "factory",
            None,
        )
        .unwrap();
    // To verify we will check configs, confirming its the same ConfigResponse and the same values
    let msg = QueryMsg::Config {};
    // Query the 'placeholder' which is now a Factory
    let migrated_factory_config: ConfigResponse =
        app.wrap().query_wasm_smart(&placeholder, &msg).unwrap();
    let direct_factory_config: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&factory_instance, &msg)
        .unwrap();

    assert_eq!(123, migrated_factory_config.token_code_id);
    assert_eq!(pair_configs, migrated_factory_config.pair_configs);
    assert_eq!(owner, migrated_factory_config.owner);

    assert_eq!(123, direct_factory_config.token_code_id);
    assert_eq!(pair_configs, direct_factory_config.pair_configs);
    assert_eq!(owner, direct_factory_config.owner);
}
