use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, WasmMsg,
};

use terra_cosmwasm::{create_swap_msg, TerraMsgWrapper};
use terraswap::asset::{Asset, AssetInfo};

use terraswap::querier::query_balance;

use white_whale::denom::LUNA_DENOM;

use white_whale::msg::create_terraswap_msg;

use white_whale::deposit_info::ArbBaseAsset;
use white_whale::query::terraswap::simulate_swap as simulate_terraswap_swap;
use white_whale::tax::deduct_tax;
use white_whale::ust_vault::msg::ExecuteMsg as VaultMsg;
use white_whale::ust_vault::msg::FlashLoanPayload;

use crate::error::StableArbError;
use crate::msg::{ArbDetails, CallbackMsg, ExecuteMsg, InitMsg, QueryMsg};

use crate::querier::query_market_price;

use crate::state::{State, ADMIN, ARB_BASE_ASSET, STATE};

type VaultResult = Result<Response<TerraMsgWrapper>, StableArbError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InitMsg) -> VaultResult {
    let state = State {
        trader: deps.api.addr_canonicalize(info.sender.as_str())?,
        vault_address: deps.api.addr_canonicalize(&msg.vault_address)?,
        seignorage_address: deps.api.addr_canonicalize(&msg.seignorage_address)?,
        pool_address: deps.api.addr_canonicalize(&msg.pool_address)?,
    };

    // Store the initial config
    STATE.save(deps.storage, &state)?;
    ARB_BASE_ASSET.save(
        deps.storage,
        &ArbBaseAsset {
            asset_info: msg.asset_info,
        },
    )?;
    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> VaultResult {
    match msg {
        ExecuteMsg::ExecuteArb { details, above_peg } => {
            call_flashloan(deps, env, info, details, above_peg)
        }
        ExecuteMsg::BelowPegCallback { details } => try_arb_below_peg(deps, env, info, details),
        ExecuteMsg::AbovePegCallback { details } => try_arb_above_peg(deps, env, info, details),
        ExecuteMsg::SetAdmin { admin } => {
            let admin_addr = deps.api.addr_validate(&admin)?;
            let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
            ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
            Ok(Response::default()
                .add_attribute("previous admin", previous_admin)
                .add_attribute("admin", admin))
        }
        // TODO: We could ommit the trader entirely, lets discuss!
        ExecuteMsg::SetTrader { trader } => set_trader(deps, info, trader),
        ExecuteMsg::Callback(msg) => _handle_callback(deps, env, info, msg),
    }
}

//----------------------------------------------------------------------------------------
//  PRIVATE FUNCTIONS
//----------------------------------------------------------------------------------------

fn _handle_callback(deps: DepsMut, env: Env, info: MessageInfo, msg: CallbackMsg) -> VaultResult {
    // Callback functions can only be called this contract itself
    if info.sender != env.contract.address {
        return Err(StableArbError::NotCallback {});
    }
    match msg {
        CallbackMsg::AfterSuccessfulTradeCallback {} => after_successful_trade_callback(deps, env),
        // Possibility to add more callbacks in future.
    }
}
//----------------------------------------------------------------------------------------
//  EXECUTE FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

fn call_flashloan(
    deps: DepsMut,
    _env: Env,
    _msg_info: MessageInfo,
    details: ArbDetails,
    above_peg: bool,
) -> VaultResult {
    let state = STATE.load(deps.storage)?;
    let deposit_info = ARB_BASE_ASSET.load(deps.storage)?;

    // Check if requested asset is same as strategy base asset
    deposit_info.assert(&details.asset.info)?;

    // Construct callback msg
    let callback_msg;
    if above_peg {
        callback_msg = ExecuteMsg::AbovePegCallback {
            details: details.clone(),
        }
    } else {
        callback_msg = ExecuteMsg::BelowPegCallback {
            details: details.clone(),
        }
    }

    // Construct payload
    let payload = FlashLoanPayload {
        requested_asset: details.asset,
        callback: to_binary(&callback_msg)?,
    };

    // Call stablecoin Vault
    Ok(
        Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&state.vault_address)?.to_string(),
            msg: to_binary(&VaultMsg::FlashLoan { payload })?,
            funds: vec![],
        })),
    )
}

// Attempt to perform an arbitrage operation with the assumption that
// the currency to be arb'd is below peg. Needed funds should be provided
// by the earlier stablecoin vault flashloan call.

pub fn try_arb_below_peg(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    details: ArbDetails,
) -> VaultResult {
    let state = STATE.load(deps.storage)?;
    let deposit_info = ARB_BASE_ASSET.load(deps.storage)?;

    // Ensure the caller is the vault
    if deps.api.addr_canonicalize(&msg_info.sender.to_string())? != state.vault_address {
        return Err(StableArbError::Unauthorized {});
    }

    // Set vars
    let denom = deposit_info.get_denom()?;
    let lent_coin = deduct_tax(
        deps.as_ref(),
        Coin::new(details.asset.amount.u128(), denom.clone()),
    )?;
    let ask_denom = LUNA_DENOM.to_string();
    let response: Response<TerraMsgWrapper> = Response::new();

    // Check if we have enough funds
    let balance = query_balance(&deps.querier, env.contract.address.clone(), denom)?;
    if balance < details.asset.amount {
        return Err(StableArbError::Broke {});
    }

    // Simulate first tx with Terra Market Module
    let expected_luna_received =
        query_market_price(deps.as_ref(), lent_coin.clone(), ask_denom.clone())?;

    // TODO: We could ommit this
    let residual_luna = query_balance(
        &deps.querier,
        env.contract.address.clone(),
        ask_denom.clone(),
    )?;

    // Construct offer for Terraswap
    let offer_coin = Coin {
        denom: ask_denom.clone(),
        amount: residual_luna + expected_luna_received,
    };

    // Market swap msg, swap STABLE -> LUNA
    let swap_msg = create_swap_msg(lent_coin.clone(), ask_denom);

    // Terraswap msg, swap LUNA -> STABLE
    let terraswap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.pool_address)?.to_string(),
        funds: vec![offer_coin.clone()],
        msg: to_binary(&create_terraswap_msg(
            offer_coin,
            details.belief_price,
            Some(details.slippage),
        ))?,
    });

    let logs = vec![
        ("action", String::from("arb below peg")),
        ("offer_amount", lent_coin.amount.to_string()),
        ("expected_luna", expected_luna_received.to_string()),
    ];

    // Create callback, this will send the funds back to the vault.
    let callback_msg =
        CallbackMsg::AfterSuccessfulTradeCallback {}.to_cosmos_msg(&env.contract.address)?;

    Ok(response
        .add_attributes(logs)
        .add_message(swap_msg)
        .add_message(terraswap_msg)
        .add_message(callback_msg))
}

// Attempt to perform an arbitrage operation with the assumption that
// the currency to be arb'd is below peg. Needed funds should be provided
// by the earlier stablecoin vault flashloan call.
pub fn try_arb_above_peg(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    details: ArbDetails,
) -> VaultResult {
    let state = STATE.load(deps.storage)?;
    let deposit_info = ARB_BASE_ASSET.load(deps.storage)?;

    // Ensure the caller is the vault
    if deps.api.addr_canonicalize(&msg_info.sender.to_string())? != state.vault_address {
        return Err(StableArbError::Unauthorized {});
    }

    // Set vars
    let denom = deposit_info.get_denom()?;
    let lent_coin = deduct_tax(
        deps.as_ref(),
        Coin::new(details.asset.amount.u128(), denom.clone()),
    )?;
    let ask_denom = LUNA_DENOM.to_string();
    let response: Response<TerraMsgWrapper> = Response::new();

    // Check if we have enough funds
    let balance = query_balance(&deps.querier, env.contract.address.clone(), denom)?;
    if balance < details.asset.amount {
        return Err(StableArbError::Broke {});
    }
    // Simulate first tx with Terraswap
    let expected_luna_received = simulate_terraswap_swap(
        deps.as_ref(),
        deps.api.addr_humanize(&state.pool_address)?,
        lent_coin.clone(),
    )?;

    // TODO: We could ommit this
    let residual_luna = query_balance(
        &deps.querier,
        env.contract.address.clone(),
        LUNA_DENOM.to_string(),
    )?;

    // Construct offer for Market Swap
    let offer_coin = Coin {
        denom: ask_denom,
        amount: residual_luna + expected_luna_received,
    };

    // Terraswap msg, swap STABLE -> LUNA
    let terraswap_msg: CosmosMsg<TerraMsgWrapper> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.pool_address)?.to_string(),
        funds: vec![lent_coin.clone()],
        msg: to_binary(&create_terraswap_msg(
            lent_coin.clone(),
            details.belief_price,
            Some(details.slippage),
        ))?,
    });

    // Market swap msg, swap LUNA -> STABLE
    let swap_msg = create_swap_msg(offer_coin, lent_coin.denom);

    let logs = vec![
        ("action", String::from("arb above peg")),
        ("offer_amount", lent_coin.amount.to_string()),
        ("expected_luna", expected_luna_received.to_string()),
    ];

    // Create callback, this will send the funds back to the vault.
    let callback_msg =
        CallbackMsg::AfterSuccessfulTradeCallback {}.to_cosmos_msg(&env.contract.address)?;

    Ok(response
        .add_attributes(logs)
        .add_message(terraswap_msg)
        .add_message(swap_msg)
        .add_message(callback_msg))
}

//----------------------------------------------------------------------------------------
//  CALLBACK FUNCTION HANDLERS
//----------------------------------------------------------------------------------------

// After the arb this function returns the funds to the vault.
fn after_successful_trade_callback(deps: DepsMut, env: Env) -> VaultResult {
    let state = STATE.load(deps.storage)?;
    let stable_denom = ARB_BASE_ASSET.load(deps.storage)?.get_denom()?;
    let stables_in_contract =
        query_balance(&deps.querier, env.contract.address, stable_denom.clone())?;

    // Send asset back to vault
    let repay_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: stable_denom,
        },
        amount: stables_in_contract,
    };

    Ok(Response::new().add_message(CosmosMsg::Bank(BankMsg::Send {
        to_address: deps.api.addr_humanize(&state.vault_address)?.to_string(),
        amount: vec![repay_asset.deduct_tax(&deps.querier)?],
    })))
}

//----------------------------------------------------------------------------------------
//  GOVERNANCE CONTROLLED SETTERS
//----------------------------------------------------------------------------------------

pub fn set_trader(deps: DepsMut, msg_info: MessageInfo, trader: String) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    // Get the old trader
    let previous_trader = deps.api.addr_humanize(&state.trader)?.to_string();
    // Store the new trader, validating it is indeed an address along the way
    state.trader = deps.api.addr_canonicalize(&trader)?;
    STATE.save(deps.storage, &state)?;
    // Respond and note the previous traders address
    Ok(Response::new()
        .add_attribute("trader", trader)
        .add_attribute("previous trader", previous_trader))
}

pub fn set_vault_addr(deps: DepsMut, msg_info: MessageInfo, vault_address: String) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    // Get the old vault
    let previous_vault = deps.api.addr_humanize(&state.vault_address)?.to_string();
    // Store the new vault addr
    state.vault_address = deps.api.addr_canonicalize(&vault_address)?;
    STATE.save(deps.storage, &state)?;
    // Respond and note the previous vault address
    Ok(Response::new()
        .add_attribute("new vault", vault_address)
        .add_attribute("previous vault", previous_vault))
}

//----------------------------------------------------------------------------------------
//  QUERY HANDLERS
//----------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&try_query_config(deps)?),
    }
}

pub fn try_query_config(deps: Deps) -> StdResult<ArbBaseAsset> {
    let info: ArbBaseAsset = ARB_BASE_ASSET.load(deps.storage)?;
    Ok(info)
}

//----------------------------------------------------------------------------------------
//  TESTS -> MOVE TO OTHER FILE
//----------------------------------------------------------------------------------------

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::testing::mock_dependencies;
//     use cosmwasm_std::testing::mock_env;
//     use cosmwasm_std::{Api, Uint128};
//     use terra_cosmwasm::TerraRoute;
//     use terraswap::asset::AssetInfo;

//     fn get_test_init_msg() -> InitMsg {
//         InitMsg {
//             pool_address: "test_pool".to_string(),
//             anchor_money_market_address: "test_mm".to_string(),
//             aust_address: "test_aust".to_string(),
//             seignorage_address: "test_seignorage".to_string(),
//             profit_check_address: "test_profit_check".to_string(),
//             community_fund_addr: "community_fund".to_string(),
//             warchest_addr: "warchest".to_string(),
//             asset_info: AssetInfo::NativeToken {
//                 denom: "uusd".to_string(),
//             },
//             token_code_id: 0u64,
//             warchest_fee: Decimal::percent(10u64),
//             community_fund_fee: Decimal::permille(5u64),
//             max_community_fund_fee: Uint128::from(1000000u64),
//             stable_cap: Uint128::from(100_000_000u64),
//             vault_lp_token_name: None,
//             vault_lp_token_symbol: None,
//         }
//     }

//     #[test]
//     fn test_initialization() {
//         let mut deps = mock_dependencies(&[]);

//         let msg = get_test_init_msg();
//         let env = mock_env();
//         let info = MessageInfo {
//             sender: deps.api.addr_validate("creator").unwrap(),
//             funds: vec![],
//         };

//         let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
//         assert_eq!(1, res.messages.len());
//     }

//     #[test]
//     fn test_init_with_non_default_vault_lp_token() {
//         let mut deps = mock_dependencies(&[]);

//         let custom_token_name = String::from("My LP Token");
//         let custom_token_symbol = String::from("MyLP");

//         // Define a custom Init Msg with the custom token info provided
//         let msg = InitMsg {
//             pool_address: "test_pool".to_string(),
//             anchor_money_market_address: "test_mm".to_string(),
//             aust_address: "test_aust".to_string(),
//             seignorage_address: "test_seignorage".to_string(),
//             profit_check_address: "test_profit_check".to_string(),
//             community_fund_addr: "community_fund".to_string(),
//             warchest_addr: "warchest".to_string(),
//             asset_info: AssetInfo::NativeToken {
//                 denom: "uusd".to_string(),
//             },
//             token_code_id: 10u64,
//             warchest_fee: Decimal::percent(10u64),
//             community_fund_fee: Decimal::permille(5u64),
//             max_community_fund_fee: Uint128::from(1000000u64),
//             stable_cap: Uint128::from(1000_000_000u64),
//             vault_lp_token_name: Some(custom_token_name.clone()),
//             vault_lp_token_symbol: Some(custom_token_symbol.clone()),
//         };

//         // Prepare mock env
//         let env = mock_env();
//         let info = MessageInfo {
//             sender: deps.api.addr_validate("creator").unwrap(),
//             funds: vec![],
//         };

//         let res = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
//         // Ensure we have 1 message
//         assert_eq!(1, res.messages.len());
//         // Verify the message is the one we expect but also that our custom provided token name and symbol were taken into account.
//         assert_eq!(
//             res.messages,
//             vec![SubMsg {
//                 // Create LP token
//                 msg: WasmMsg::Instantiate {
//                     admin: None,
//                     code_id: msg.token_code_id,
//                     msg: to_binary(&TokenInstantiateMsg {
//                         name: custom_token_name.to_string(),
//                         symbol: custom_token_symbol.to_string(),
//                         decimals: 6,
//                         initial_balances: vec![],
//                         mint: Some(MinterResponse {
//                             minter: env.contract.address.to_string(),
//                             cap: None,
//                         }),
//                     })
//                     .unwrap(),
//                     funds: vec![],
//                     label: "".to_string(),
//                 }
//                 .into(),
//                 gas_limit: None,
//                 id: u64::from(INSTANTIATE_REPLY_ID),
//                 reply_on: ReplyOn::Success,
//             }]
//         );
//     }

//     #[test]
//     fn test_set_slippage() {
//         let mut deps = mock_dependencies(&[]);

//         let msg = get_test_init_msg();
//         let env = mock_env();
//         let msg_info = MessageInfo {
//             sender: deps.api.addr_validate("creator").unwrap(),
//             funds: vec![],
//         };

//         let res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();
//         assert_eq!(1, res.messages.len());

//         let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
//         assert_eq!(info.stable_cap, Uint128::from(100_000_000u64));

//         let msg = ExecuteMsg::SetStableCap {
//             stable_cap: Uint128::from(100_000u64),
//         };
//         let _res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
//         let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
//         assert_eq!(info.stable_cap, Uint128::from(100_000u64));
//     }

//     #[test]
//     fn test_set_warchest_fee() {
//         let mut deps = mock_dependencies(&[]);

//         let msg = get_test_init_msg();
//         let env = mock_env();
//         let msg_info = MessageInfo {
//             sender: deps.api.addr_validate("creator").unwrap(),
//             funds: vec![],
//         };

//         let res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();
//         assert_eq!(1, res.messages.len());

//         let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
//         assert_eq!(info.stable_cap, Uint128::from(100_000_000u64));

//         let warchest_fee = FEE.load(&deps.storage).unwrap().warchest_fee.share;
//         let new_fee = Decimal::permille(1u64);
//         assert_ne!(warchest_fee, new_fee);
//         let msg = ExecuteMsg::SetFee {
//             community_fund_fee: None,
//             warchest_fee: Some(Fee { share: new_fee }),
//         };
//         let _res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
//         let warchest_fee = FEE.load(&deps.storage).unwrap().warchest_fee.share;
//         assert_eq!(warchest_fee, new_fee);
//     }

//     #[test]
//     fn test_set_community_fund_fee() {
//         let mut deps = mock_dependencies(&[]);

//         let msg = get_test_init_msg();
//         let env = mock_env();
//         let msg_info = MessageInfo {
//             sender: deps.api.addr_validate("creator").unwrap(),
//             funds: vec![],
//         };

//         let res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();
//         assert_eq!(1, res.messages.len());

//         let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
//         assert_eq!(info.stable_cap, Uint128::from(100_000u64));

//         let community_fund_fee = FEE
//             .load(&deps.storage)
//             .unwrap()
//             .community_fund_fee
//             .fee
//             .share;
//         let new_fee = Decimal::permille(1u64);
//         let new_max_fee = Uint128::from(42u64);
//         assert_ne!(community_fund_fee, new_fee);
//         let msg = ExecuteMsg::SetFee {
//             community_fund_fee: Some(CappedFee {
//                 fee: Fee { share: new_fee },
//                 max_fee: new_max_fee,
//             }),
//             warchest_fee: None,
//         };
//         let _res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
//         let community_fund_fee = FEE
//             .load(&deps.storage)
//             .unwrap()
//             .community_fund_fee
//             .fee
//             .share;
//         let community_fund_max_fee = FEE.load(&deps.storage).unwrap().community_fund_fee.max_fee;
//         assert_eq!(community_fund_fee, new_fee);
//         assert_eq!(community_fund_max_fee, new_max_fee);
//     }

//     #[test]
//     fn when_given_a_below_peg_msg_then_handle_returns_first_a_mint_then_a_terraswap_msg() {
//         let mut deps = mock_dependencies(&[]);

//         let msg = get_test_init_msg();
//         let env = mock_env();
//         let msg_info = MessageInfo {
//             sender: deps.api.addr_validate("creator").unwrap(),
//             funds: vec![],
//         };

//         let _res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();

//         let msg = ExecuteMsg::BelowPeg {
//             amount: Coin {
//                 denom: "uusd".to_string(),
//                 amount: Uint128::from(1000000u64),
//             },
//             slippage: Decimal::percent(1u64),
//             belief_price: Decimal::from_ratio(Uint128::new(320), Uint128::new(10)),
//         };

//         let res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
//         assert_eq!(4, res.messages.len());
//         let second_msg = res.messages[1].msg.clone();
//         match second_msg {
//             CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
//             CosmosMsg::Custom(t) => assert_eq!(TerraRoute::Market, t.route),
//             CosmosMsg::Wasm(_wasm_msg) => panic!("unexpected"),
//             _ => panic!("unexpected"),
//         }
//         let second_msg = res.messages[2].msg.clone();
//         match second_msg {
//             CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
//             CosmosMsg::Custom(_t) => panic!("unexpected"),
//             CosmosMsg::Wasm(_wasm_msg) => {}
//             _ => panic!("unexpected"),
//         }
//     }

//     #[test]
//     fn when_given_an_above_peg_msg_then_handle_returns_first_a_terraswap_then_a_mint_msg() {
//         let mut deps = mock_dependencies(&[]);

//         let msg = get_test_init_msg();
//         let env = mock_env();
//         let msg_info = MessageInfo {
//             sender: deps.api.addr_validate("creator").unwrap(),
//             funds: vec![],
//         };

//         let _res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();

//         let msg = ExecuteMsg::AbovePeg {
//             amount: Coin {
//                 denom: "uusd".to_string(),
//                 amount: Uint128::from(1000000u64),
//             },
//             slippage: Decimal::percent(1u64),
//             belief_price: Decimal::from_ratio(Uint128::new(320), Uint128::new(10)),
//         };

//         let res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
//         assert_eq!(4, res.messages.len());
//         let second_msg = res.messages[1].msg.clone();
//         match second_msg {
//             CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
//             CosmosMsg::Custom(_t) => panic!("unexpected"),
//             CosmosMsg::Wasm(_wasm_msg) => {}
//             _ => panic!("unexpected"),
//         }
//         let third_msg = res.messages[2].msg.clone();
//         match third_msg {
//             CosmosMsg::Bank(_bank_msg) => panic!("unexpected"),
//             CosmosMsg::Custom(t) => assert_eq!(TerraRoute::Market, t.route),
//             CosmosMsg::Wasm(_wasm_msg) => panic!("unexpected"),
//             _ => panic!("unexpected"),
//         }
//     }
// }

// TODO:
// - Deposit when 0 in pool -> fix by requiring one UST one init
// - Add config for deposit amounts

//----------------------------------------------------------------------------------------
//  WIP
//----------------------------------------------------------------------------------------

// const COMMISSION_RATE: &str = "0.003";
// fn compute_swap(
//     offer_pool: Uint128,
//     ask_pool: Uint128,
//     offer_amount: Uint128,
// ) -> StdResult<(Uint128, Uint128, Uint128)> {
//     // offer => ask
//     // ask_amount = (ask_pool - cp / (offer_pool + offer_amount)) * (1 - commission_rate)
//     let cp = Uint128(offer_pool.u128() * ask_pool.u128());
//     let return_amount = (ask_pool - cp.multiply_ratio(1u128, offer_pool + offer_amount))?;

//     // calculate spread & commission
//     let spread_amount: Uint128 = (offer_amount * Decimal::from_ratio(ask_pool, offer_pool)
//         - return_amount)
//         .unwrap_or_else(|_| Uint128::zero());
//     let commission_amount: Uint128 = return_amount * Decimal::from_str(&COMMISSION_RATE).unwrap();

//     // commission will be absorbed to pool
//     let return_amount: Uint128 = (return_amount - commission_amount).unwrap();

//     Ok((return_amount, spread_amount, commission_amount))
// }
