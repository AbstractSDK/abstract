use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Decimal, ReplyOn, SubMsg, Uint128, WasmMsg,
};
use cw_utils::MsgInstantiateContractResponse;
use wyndex::fee_config::FeeConfig;

use crate::mock_querier::mock_dependencies;
use crate::state::CONFIG;
use crate::{
    contract::{execute, instantiate, query},
    error::ContractError,
};
use wyndex::asset::AssetInfo;
use wyndex::factory::{
    ConfigResponse, DefaultStakeConfig, ExecuteMsg, InstantiateMsg, PairConfig, PairType,
    PairsResponse, PartialStakeConfig, QueryMsg,
};
use wyndex::pair::PairInfo;

use crate::contract::reply;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use wyndex::pair::InstantiateMsg as PairInstantiateMsg;

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
fn pair_type_to_string() {
    assert_eq!(PairType::Xyk {}.to_string(), "xyk");
    assert_eq!(PairType::Stable {}.to_string(), "stable");
    assert_eq!(PairType::Lsd {}.to_string(), "lsd");
}

#[test]
fn proper_initialization() {
    // Validate total and protocol fee bps
    let mut deps = mock_dependencies(&[]);
    let owner = "owner0000".to_string();

    let msg = InstantiateMsg {
        pair_configs: vec![
            PairConfig {
                code_id: 123u64,
                pair_type: PairType::Xyk {},
                fee_config: FeeConfig {
                    total_fee_bps: 100,
                    protocol_fee_bps: 10,
                },
                is_disabled: false,
            },
            PairConfig {
                code_id: 325u64,
                pair_type: PairType::Xyk {},
                fee_config: FeeConfig {
                    total_fee_bps: 100,
                    protocol_fee_bps: 10,
                },
                is_disabled: false,
            },
        ],
        token_code_id: 123u64,
        fee_address: None,
        owner: owner.clone(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(),
        trading_starts: None,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    let res = instantiate(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::PairConfigDuplicate {});

    let msg = InstantiateMsg {
        pair_configs: vec![PairConfig {
            code_id: 123u64,
            pair_type: PairType::Xyk {},
            fee_config: FeeConfig {
                total_fee_bps: 10_001,
                protocol_fee_bps: 10,
            },
            is_disabled: false,
        }],
        token_code_id: 123u64,
        fee_address: None,
        owner: owner.clone(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(),
        trading_starts: None,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    let res = instantiate(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::PairConfigInvalidFeeBps {});

    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        pair_configs: vec![
            PairConfig {
                code_id: 325u64,
                pair_type: PairType::Lsd {},
                fee_config: FeeConfig {
                    total_fee_bps: 100,
                    protocol_fee_bps: 10,
                },
                is_disabled: false,
            },
            PairConfig {
                code_id: 123u64,
                pair_type: PairType::Xyk {},
                fee_config: FeeConfig {
                    total_fee_bps: 100,
                    protocol_fee_bps: 10,
                },
                is_disabled: false,
            },
        ],
        token_code_id: 123u64,
        fee_address: None,
        owner: owner.clone(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(),
        trading_starts: None,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

    let query_res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&query_res).unwrap();
    assert_eq!(123u64, config_res.token_code_id);
    assert_eq!(msg.pair_configs, config_res.pair_configs);
    assert_eq!(Addr::unchecked(owner), config_res.owner);
}

#[test]
fn trading_starts_validation() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    let owner = "owner";

    let mut msg = InstantiateMsg {
        pair_configs: vec![],
        token_code_id: 123u64,
        fee_address: None,
        owner: owner.to_string(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(),
        trading_starts: None,
    };

    // in the past
    msg.trading_starts = Some(env.block.time.seconds() - 1);
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap_err();
    assert_eq!(res, ContractError::InvalidTradingStart {});

    const SECONDS_PER_DAY: u64 = 60 * 60 * 24;
    // too late
    msg.trading_starts = Some(env.block.time.seconds() + 60 * SECONDS_PER_DAY + 1);
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap_err();
    assert_eq!(res, ContractError::InvalidTradingStart {});

    // just before too late
    msg.trading_starts = Some(env.block.time.seconds() + 60 * SECONDS_PER_DAY);
    instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    // right now
    msg.trading_starts = Some(env.block.time.seconds());
    instantiate(deps.as_mut(), env, info, msg).unwrap();
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);
    let owner = "owner0000";

    let pair_configs = vec![PairConfig {
        code_id: 123u64,
        pair_type: PairType::Xyk {},
        fee_config: FeeConfig {
            total_fee_bps: 3,
            protocol_fee_bps: 166,
        },
        is_disabled: false,
    }];

    let msg = InstantiateMsg {
        pair_configs,
        token_code_id: 123u64,
        fee_address: None,
        owner: owner.to_string(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(),
        trading_starts: None,
    };

    let env = mock_env();
    let info = mock_info(owner, &[]);

    // We can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    // Update config
    let env = mock_env();
    let info = mock_info(owner, &[]);
    let msg = ExecuteMsg::UpdateConfig {
        token_code_id: Some(200u64),
        fee_address: Some(String::from("new_fee_addr")),
        only_owner_can_create_pairs: Some(true),
        default_stake_config: None,
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // It worked, let's query the state
    let query_res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&query_res).unwrap();
    assert_eq!(200u64, config_res.token_code_id);
    assert_eq!(owner, config_res.owner);
    assert_eq!(
        String::from("new_fee_addr"),
        config_res.fee_address.unwrap()
    );

    // Unauthorized err
    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        token_code_id: None,
        fee_address: None,
        only_owner_can_create_pairs: None,
        default_stake_config: None,
    };

    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {});
}

#[test]
fn update_owner() {
    let mut deps = mock_dependencies(&[]);
    let owner = "owner0000";

    let msg = InstantiateMsg {
        pair_configs: vec![],
        token_code_id: 123u64,
        fee_address: None,
        owner: owner.to_string(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(),
        trading_starts: None,
    };

    let env = mock_env();
    let info = mock_info(owner, &[]);

    // We can just call .unwrap() to assert this was a success
    instantiate(deps.as_mut(), env, info, msg).unwrap();

    let new_owner = String::from("new_owner");

    // New owner
    let env = mock_env();
    let msg = ExecuteMsg::ProposeNewOwner {
        owner: new_owner.clone(),
        expires_in: 100, // seconds
    };

    let info = mock_info(new_owner.as_str(), &[]);

    // Unauthorized check
    let err = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap_err();
    assert_eq!(err.to_string(), "Generic error: Unauthorized");

    // Claim before proposal
    let info = mock_info(new_owner.as_str(), &[]);
    execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::ClaimOwnership {},
    )
    .unwrap_err();

    // Propose new owner
    let info = mock_info(owner, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Unauthorized ownership claim
    let info = mock_info("invalid_addr", &[]);
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::ClaimOwnership {},
    )
    .unwrap_err();
    assert_eq!(err.to_string(), "Generic error: Unauthorized");

    // Claim ownership
    let info = mock_info(new_owner.as_str(), &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::ClaimOwnership {},
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // Let's query the state
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(new_owner, config.owner);
}

#[test]
fn update_pair_config() {
    let mut deps = mock_dependencies(&[]);
    let owner = "owner0000";
    let pair_configs = vec![PairConfig {
        code_id: 123u64,
        pair_type: PairType::Xyk {},
        fee_config: FeeConfig {
            total_fee_bps: 100,
            protocol_fee_bps: 10,
        },
        is_disabled: false,
    }];

    let msg = InstantiateMsg {
        pair_configs: pair_configs.clone(),
        token_code_id: 123u64,
        fee_address: None,
        owner: owner.to_string(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(),
        trading_starts: None,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    // We can just call .unwrap() to assert this was a success
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // It worked, let's query the state
    let query_res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&query_res).unwrap();
    assert_eq!(pair_configs, config_res.pair_configs);

    // Update config
    let pair_config = PairConfig {
        code_id: 800,
        pair_type: PairType::Xyk {},
        fee_config: FeeConfig {
            total_fee_bps: 1,
            protocol_fee_bps: 2,
        },
        is_disabled: false,
    };

    // Unauthorized err
    let env = mock_env();
    let info = mock_info("wrong-addr0000", &[]);
    let msg = ExecuteMsg::UpdatePairConfig {
        config: pair_config.clone(),
    };

    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {});

    // Check validation of total and protocol fee bps
    let env = mock_env();
    let info = mock_info(owner, &[]);
    let msg = ExecuteMsg::UpdatePairConfig {
        config: PairConfig {
            code_id: 123u64,
            pair_type: PairType::Xyk {},
            fee_config: FeeConfig {
                total_fee_bps: 3,
                protocol_fee_bps: 10_001,
            },
            is_disabled: false,
        },
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(res, ContractError::PairConfigInvalidFeeBps {});

    let info = mock_info(owner, &[]);
    let msg = ExecuteMsg::UpdatePairConfig {
        config: pair_config.clone(),
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // It worked, let's query the state
    let query_res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&query_res).unwrap();
    assert_eq!(vec![pair_config.clone()], config_res.pair_configs);

    // Add second config
    let pair_config_custom = PairConfig {
        code_id: 100,
        pair_type: PairType::Custom("test".to_string()),
        fee_config: FeeConfig {
            total_fee_bps: 10,
            protocol_fee_bps: 20,
        },
        is_disabled: false,
    };

    let info = mock_info(owner, &[]);
    let msg = ExecuteMsg::UpdatePairConfig {
        config: pair_config_custom.clone(),
    };

    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // It worked, let's query the state
    let query_res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_binary(&query_res).unwrap();
    assert_eq!(
        vec![pair_config_custom, pair_config],
        config_res.pair_configs
    );
}

#[test]
fn create_pair() {
    let mut deps = mock_dependencies(&[]);

    let pair_config = PairConfig {
        code_id: 321u64,
        pair_type: PairType::Xyk {},
        fee_config: FeeConfig {
            total_fee_bps: 100,
            protocol_fee_bps: 10,
        },
        is_disabled: false,
    };

    let msg = InstantiateMsg {
        pair_configs: vec![pair_config.clone()],
        token_code_id: 123u64,
        fee_address: None,
        owner: "owner0000".to_string(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(),
        trading_starts: None,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    // We can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();

    let asset_infos = vec![
        AssetInfo::Token("asset0000".to_string()),
        AssetInfo::Token("asset0001".to_string()),
    ];

    let config = CONFIG.load(&deps.storage);
    let env = mock_env();
    let info = mock_info("owner0000", &[]);

    // Check pair creation using a non-whitelisted pair ID
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::CreatePair {
            pair_type: PairType::Lsd {},
            asset_infos: asset_infos.clone(),
            init_params: None,
            total_fee_bps: None,
            staking_config: PartialStakeConfig::default(),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::PairConfigNotFound {});

    let res = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::CreatePair {
            pair_type: PairType::Xyk {},
            asset_infos: asset_infos.clone(),
            init_params: None,
            total_fee_bps: None,
            staking_config: PartialStakeConfig::default(),
        },
    )
    .unwrap();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "create_pair"),
            attr("pair", "asset0000-asset0001")
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg {
            msg: WasmMsg::Instantiate {
                msg: to_binary(&PairInstantiateMsg {
                    factory_addr: String::from(MOCK_CONTRACT_ADDR),
                    asset_infos,
                    token_code_id: msg.token_code_id,
                    init_params: None,
                    staking_config: default_stake_config().to_stake_config(),
                    trading_starts: mock_env().block.time.seconds(),
                    fee_config: pair_config.fee_config,
                    circuit_breaker: None,
                })
                .unwrap(),
                code_id: pair_config.code_id,
                funds: vec![],
                admin: Some(config.unwrap().owner.to_string()),
                label: String::from("Wyndex pair"),
            }
            .into(),
            id: 1,
            gas_limit: None,
            reply_on: ReplyOn::Success
        }]
    );
}

#[test]
fn register() {
    let mut deps = mock_dependencies(&[]);
    let owner = "owner0000";

    let msg = InstantiateMsg {
        pair_configs: vec![PairConfig {
            code_id: 123u64,
            pair_type: PairType::Xyk {},
            fee_config: FeeConfig {
                total_fee_bps: 100,
                protocol_fee_bps: 10,
            },
            is_disabled: false,
        }],
        token_code_id: 123u64,
        fee_address: None,
        owner: owner.to_string(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(),
        trading_starts: None,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    let asset_infos = vec![
        AssetInfo::Token("asset0000".to_string()),
        AssetInfo::Token("asset0001".to_string()),
    ];

    let msg = ExecuteMsg::CreatePair {
        pair_type: PairType::Xyk {},
        asset_infos: asset_infos.clone(),
        init_params: None,
        staking_config: PartialStakeConfig::default(),
        total_fee_bps: None,
    };

    let env = mock_env();
    let info = mock_info(owner, &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let pair0_addr = "pair0000".to_string();
    let validated_asset_infos: Vec<_> = asset_infos
        .iter()
        .cloned()
        .map(|a| a.validate(&deps.api).unwrap())
        .collect();
    let pair0_info = PairInfo {
        asset_infos: validated_asset_infos.clone(),
        contract_addr: Addr::unchecked("pair0000"),
        staking_addr: Addr::unchecked("stake0000"),
        liquidity_token: Addr::unchecked("liquidity0000"),
        pair_type: PairType::Xyk {},
        fee_config: FeeConfig {
            total_fee_bps: 0,
            protocol_fee_bps: 0,
        },
    };

    let mut deployed_pairs = vec![(&pair0_addr, &pair0_info)];

    // Register an Wyndex pair querier
    deps.querier.with_wyndex_pairs(&deployed_pairs);

    let instantiate_res = MsgInstantiateContractResponse {
        contract_address: String::from("pair0000"),
        data: None,
    };

    let _res = reply::instantiate_pair(deps.as_mut(), mock_env(), instantiate_res.clone()).unwrap();

    let query_res = query(
        deps.as_ref(),
        env,
        QueryMsg::Pair {
            asset_infos: asset_infos.clone(),
        },
    )
    .unwrap();

    let pair_res: PairInfo = from_binary(&query_res).unwrap();
    assert_eq!(
        pair_res,
        PairInfo {
            liquidity_token: Addr::unchecked("liquidity0000"),
            contract_addr: Addr::unchecked("pair0000"),
            staking_addr: Addr::unchecked("stake0000"),
            asset_infos: validated_asset_infos.clone(),
            pair_type: PairType::Xyk {},
            fee_config: FeeConfig {
                total_fee_bps: 0,
                protocol_fee_bps: 0,
            },
        }
    );

    // Check pair was registered
    let res = reply::instantiate_pair(deps.as_mut(), mock_env(), instantiate_res).unwrap_err();
    assert_eq!(res, ContractError::PairWasRegistered {});

    // Store one more item to test query pairs
    let asset_infos_2 = vec![
        AssetInfo::Token("asset0000".to_string()),
        AssetInfo::Token("asset0002".to_string()),
    ];
    let validated_asset_infos_2: Vec<_> = asset_infos_2
        .iter()
        .cloned()
        .map(|a| a.validate(&deps.api).unwrap())
        .collect();

    let msg = ExecuteMsg::CreatePair {
        pair_type: PairType::Xyk {},
        asset_infos: asset_infos_2.clone(),
        init_params: None,
        staking_config: PartialStakeConfig::default(),
        total_fee_bps: None,
    };

    let env = mock_env();
    let info = mock_info(owner, &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let pair1_addr = "pair0001".to_string();
    let pair1_info = PairInfo {
        asset_infos: validated_asset_infos_2.clone(),
        contract_addr: Addr::unchecked("pair0001"),
        staking_addr: Addr::unchecked("stake0001"),
        liquidity_token: Addr::unchecked("liquidity0001"),
        pair_type: PairType::Xyk {},
        fee_config: FeeConfig {
            total_fee_bps: 0,
            protocol_fee_bps: 0,
        },
    };

    deployed_pairs.push((&pair1_addr, &pair1_info));

    // Register wyndex pair querier
    deps.querier.with_wyndex_pairs(&deployed_pairs);

    let instantiate_res = MsgInstantiateContractResponse {
        contract_address: String::from("pair0001"),
        data: None,
    };

    let _res = reply::instantiate_pair(deps.as_mut(), mock_env(), instantiate_res).unwrap();

    let query_msg = QueryMsg::Pairs {
        start_after: None,
        limit: None,
    };

    let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let pairs_res: PairsResponse = from_binary(&res).unwrap();
    assert_eq!(
        pairs_res.pairs,
        vec![
            PairInfo {
                liquidity_token: Addr::unchecked("liquidity0000"),
                contract_addr: Addr::unchecked("pair0000"),
                staking_addr: Addr::unchecked("stake0000"),
                asset_infos: validated_asset_infos.clone(),
                pair_type: PairType::Xyk {},
                fee_config: FeeConfig {
                    total_fee_bps: 0,
                    protocol_fee_bps: 0,
                },
            },
            PairInfo {
                liquidity_token: Addr::unchecked("liquidity0001"),
                contract_addr: Addr::unchecked("pair0001"),
                staking_addr: Addr::unchecked("stake0001"),
                asset_infos: validated_asset_infos_2.clone(),
                pair_type: PairType::Xyk {},
                fee_config: FeeConfig {
                    total_fee_bps: 0,
                    protocol_fee_bps: 0,
                },
            }
        ]
    );

    let query_msg = QueryMsg::Pairs {
        start_after: None,
        limit: Some(1),
    };

    let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let pairs_res: PairsResponse = from_binary(&res).unwrap();
    assert_eq!(
        pairs_res.pairs,
        vec![PairInfo {
            liquidity_token: Addr::unchecked("liquidity0000"),
            contract_addr: Addr::unchecked("pair0000"),
            staking_addr: Addr::unchecked("stake0000"),
            asset_infos: validated_asset_infos.clone(),
            pair_type: PairType::Xyk {},
            fee_config: FeeConfig {
                total_fee_bps: 0,
                protocol_fee_bps: 0,
            },
        }]
    );

    let query_msg = QueryMsg::Pairs {
        start_after: Some(asset_infos),
        limit: None,
    };

    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let pairs_res: PairsResponse = from_binary(&res).unwrap();
    assert_eq!(
        pairs_res.pairs,
        vec![PairInfo {
            liquidity_token: Addr::unchecked("liquidity0001"),
            contract_addr: Addr::unchecked("pair0001"),
            staking_addr: Addr::unchecked("stake0001"),
            asset_infos: validated_asset_infos_2,
            pair_type: PairType::Xyk {},
            fee_config: FeeConfig {
                total_fee_bps: 0,
                protocol_fee_bps: 0,
            },
        }]
    );

    // Deregister from wrong acc
    let env = mock_env();
    let info = mock_info("wrong_addr0000", &[]);
    let res = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::Deregister {
            asset_infos: asset_infos_2.clone(),
        },
    )
    .unwrap_err();

    assert_eq!(res, ContractError::Unauthorized {});

    // Proper deregister
    let env = mock_env();
    let info = mock_info(owner, &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::Deregister {
            asset_infos: asset_infos_2,
        },
    )
    .unwrap();

    assert_eq!(res.attributes[0], attr("action", "deregister"));

    let query_msg = QueryMsg::Pairs {
        start_after: None,
        limit: None,
    };

    let res = query(deps.as_ref(), env, query_msg).unwrap();
    let pairs_res: PairsResponse = from_binary(&res).unwrap();
    assert_eq!(
        pairs_res.pairs,
        vec![PairInfo {
            liquidity_token: Addr::unchecked("liquidity0000"),
            contract_addr: Addr::unchecked("pair0000"),
            staking_addr: Addr::unchecked("stake0000"),
            asset_infos: validated_asset_infos,
            pair_type: PairType::Xyk {},
            fee_config: FeeConfig {
                total_fee_bps: 0,
                protocol_fee_bps: 0,
            },
        },]
    );
}
