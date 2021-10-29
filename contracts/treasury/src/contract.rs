use cosmwasm_std::{
    entry_point, from_binary, to_binary, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Decimal,
    Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg,
    Uint128, WasmMsg,
};
use protobuf::Message;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::Cw20HookMsg;
use terraswap::querier::{query_balance, query_supply, query_token_balance};
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};

use white_whale::msg::create_terraswap_msg;
use white_whale::query::terraswap::simulate_swap as simulate_terraswap_swap;
use white_whale::ust_vault::msg::VaultQueryMsg as QueryMsg;

use crate::msg::{HandleMsg, InitMsg, PoolResponse};
use crate::pool_info::{PoolInfo, PoolInfoRaw};
use crate::response::MsgInstantiateContractResponse;
use crate::state::{State, LUNA_DENOM, POOL_INFO, STATE};

const INSTANTIATE_REPLY_ID: u64 = 1;
const DEFAULT_LP_TOKEN_NAME: &str = "White Whale Luna-bLuna Vault LP Token";
const DEFAULT_LP_TOKEN_SYMBOL: &str = "wwVbLuna";

/*  This contract implements the bLuna arbitrage vault.


The bLuna vault performs arbitrage operations on the bLuna-Luna Terraswap Pair
*/
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    let state = State {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        memory_address: deps.api.addr_canonicalize(info.sender.as_str())?,
        approved_interfaces: vec![]
        bluna_hub_address: deps.api.addr_canonicalize(&msg.bluna_hub_address)?,
        bluna_address: deps.api.addr_canonicalize(&msg.bluna_address)?,
    };

    STATE.save(deps.storage, &state)?;

    let pool_info: &PoolInfoRaw = &PoolInfoRaw {
        contract_addr: deps.api.addr_canonicalize(env.contract.address.as_str())?,
        liquidity_token: CanonicalAddr::from(vec![]),
        slippage: msg.slippage,
        asset_infos: [
            AssetInfo::Token {
                contract_addr: msg.bluna_address,
            }
            .to_raw(deps.api)?,
            AssetInfo::NativeToken {
                denom: LUNA_DENOM.to_string(),
            }
            .to_raw(deps.api)?,
        ],
    };
    POOL_INFO.save(deps.storage, pool_info)?;

    // Both the lp_token_name and symbol are Options, attempt to unwrap their value falling back to the default if not provided
    let lp_token_name: String = msg
        .vault_lp_token_name
        .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_NAME));
    let lp_token_symbol: String = msg
        .vault_lp_token_symbol
        .unwrap_or_else(|| String::from(DEFAULT_LP_TOKEN_SYMBOL));

    Ok(Response::new().add_submessage(SubMsg {
        // Create LP token
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: msg.token_code_id,
            msg: to_binary(&TokenInstantiateMsg {
                name: lp_token_name,
                symbol: lp_token_symbol,
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
            })?,
            funds: vec![],
            label: "".to_string(),
        }
        .into(),
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let data = msg.result.unwrap().data.unwrap();
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(data.as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let liquidity_token = res.get_contract_address();

    let api = deps.api;
    POOL_INFO.update(deps.storage, |mut meta| -> StdResult<_> {
        meta.liquidity_token = api.addr_canonicalize(liquidity_token)?;
        Ok(meta)
    })?;

    Ok(Response::new().add_attribute("liquidity_token_addr", liquidity_token))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: HandleMsg) -> StdResult<Response> {
    match msg {
        HandleMsg::Receive(msg) => receive_cw20(deps, info, msg),
        HandleMsg::Swap { amount } => try_swap(deps, info, amount),
        HandleMsg::ProvideLiquidity { asset } => try_provide_liquidity(deps, info, asset),
        HandleMsg::SetSlippage { slippage } => set_slippage(deps, info, slippage),
    }
}

/// try_swap attempts to perform a swap between uluna and bluna, depending on what coin is offered.
pub fn try_swap(deps: DepsMut, info: MessageInfo, offer_coin: Coin) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;
    if deps.api.addr_canonicalize(&info.sender.to_string())? != state.trader {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let slippage = (POOL_INFO.load(deps.storage)?).slippage;
    let belief_price = Decimal::from_ratio(
        simulate_terraswap_swap(
            deps.as_ref(),
            deps.api.addr_humanize(&state.pool_address)?,
            offer_coin.clone(),
        )?,
        offer_coin.amount,
    );

    // Sell luna and buy bluna
    let msg = if offer_coin.denom == "uluna" {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.pool_address.to_string(),
            funds: vec![offer_coin.clone()],
            msg: to_binary(&create_terraswap_msg(
                offer_coin,
                belief_price,
                Some(slippage),
            ))?,
        })
    // Or sell bluna and buy luna
    } else {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.bluna_address.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: state.pool_address.to_string(),
                amount: offer_coin.amount,
                msg: to_binary(&create_terraswap_msg(
                    offer_coin,
                    belief_price,
                    Some(slippage),
                ))?,
            })?,
        })
    };

    Ok(Response::new().add_message(msg))
}

// perform a computation first by getting both the deposits in luna and bluna for the contract and then sum them
pub fn compute_total_deposits(deps: Deps, info: &PoolInfoRaw) -> StdResult<Uint128> {
    let state = STATE.load(deps.storage)?;
    let contract_address = deps.api.addr_humanize(&info.contract_addr)?;
    let deposits_in_luna = query_balance(
        &deps.querier,
        contract_address.clone(),
        LUNA_DENOM.to_string(),
    )?;
    let deposits_in_bluna = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&state.bluna_address)?,
        contract_address,
    )?;
    let total_deposits_in_luna = deposits_in_luna + deposits_in_bluna;
    Ok(total_deposits_in_luna)
}

/// attempt to withdraw deposits. Fees should be calculated and deducted and the net refund is sent
pub fn try_withdraw_liquidity(
    deps: DepsMut,
    sender: String,
    amount: Uint128,
) -> StdResult<Response> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let liquidity_addr = deps.api.addr_humanize(&info.liquidity_token)?;

    let total_share: Uint128 = query_supply(&deps.querier, liquidity_addr)?;
    let total_deposits: Uint128 = compute_total_deposits(deps.as_ref(), &info)?;

    let share_ratio: Decimal = Decimal::from_ratio(amount, total_share);
    // amount of luna to return
    let refund_asset: Asset = Asset {
        info: AssetInfo::NativeToken {
            denom: get_stable_denom(deps.as_ref())?,
        },
        amount: total_deposits * share_ratio,
    };

    let refund_msg = match &refund_asset.info {
        AssetInfo::Token { contract_addr } => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.clone(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender,
                amount,
            })?,
            funds: vec![],
        }),
        AssetInfo::NativeToken { .. } => CosmosMsg::Bank(BankMsg::Send {
            to_address: sender,
            amount: vec![refund_asset.deduct_tax(&deps.querier)?],
        }),
    };
    // Burn vault lp token
    let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: info.liquidity_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
        funds: vec![],
    });
    Ok(Response::new()
        .add_message(refund_msg)
        .add_message(burn_msg)
        .add_attribute("action", "withdraw_liquidity")
        .add_attribute("withdrawn_share", &amount.to_string())
        .add_attribute("refund_asset", format!(" {}", refund_asset)))
}

/// handler function invoked when the bluna-vault contract receives
/// a transaction. This is akin to a payable function in Solidity
pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let pool_info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Swap { .. } => Err(StdError::generic_err(
            "no swaps can be performed in this pool",
        )),
        Cw20HookMsg::WithdrawLiquidity {} => {
            if deps.api.addr_canonicalize(&info.sender.to_string())? != pool_info.liquidity_token {
                return Err(StdError::generic_err("Unauthorized"));
            }

            try_withdraw_liquidity(deps, cw20_msg.sender.to_string(), cw20_msg.amount)
        }
    }
}

pub fn get_stable_denom(deps: Deps) -> StdResult<String> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let stable_info = info.asset_infos[0].to_normal(deps.api)?;
    let stable_denom = match stable_info {
        AssetInfo::Token { .. } => String::default(),
        AssetInfo::NativeToken { denom } => denom,
    };
    if stable_denom == String::default() {
        return Err(StdError::generic_err(
            "get_stable_denom failed: No native token found.",
        ));
    }

    Ok(stable_denom)
}

pub fn get_slippage_ratio(slippage: Decimal) -> StdResult<Decimal> {
    Ok(Decimal::from_ratio(
        Uint128::from(100u64) - Uint128::from(100u64) * slippage,
        Uint128::from(100u64),
    ))
}

pub fn try_provide_liquidity(
    deps: DepsMut,
    info: MessageInfo,
    asset: Asset,
) -> StdResult<Response> {
    asset.assert_sent_native_token_balance(&info)?;

    let deposit: Uint128 = asset.amount;
    let pool_info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let total_deposits_in_luna: Uint128 =
        compute_total_deposits(deps.as_ref(), &pool_info)? - deposit;

    let liquidity_token = deps.api.addr_humanize(&pool_info.liquidity_token)?;
    let total_share = query_supply(&deps.querier, liquidity_token)?;
    let share = if total_share == Uint128::zero() {
        // Initial share = collateral amount
        deposit
    } else {
        deposit.multiply_ratio(total_share, total_deposits_in_luna)
    };

    // mint LP token to sender
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: pool_info.liquidity_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: info.sender.to_string(),
            amount: share,
        })?,
        funds: vec![],
    });
    Ok(Response::new().add_message(msg))
}

pub fn set_slippage(
    deps: DepsMut,
    msg_info: MessageInfo,
    slippage: Decimal,
) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;
    if deps.api.addr_canonicalize(&msg_info.sender.to_string())? != state.owner {
        return Err(StdError::generic_err("Unauthorized."));
    }
    let mut info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    info.slippage = slippage;
    POOL_INFO.save(deps.storage, &info)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&try_query_config(deps)?),
        QueryMsg::Pool {} => to_binary(&try_query_pool(deps)?),
        QueryMsg::Fees {} => to_binary(""),
        // TODO: Finish fee calculation and estimation
        QueryMsg::EstimateDepositFee { .. } => to_binary(""),
        QueryMsg::EstimateWithdrawFee { .. } => to_binary(""),
        QueryMsg::VaultValue { .. } => to_binary(""),
    }
}

pub fn try_query_config(deps: Deps) -> StdResult<PoolInfo> {
    let info = POOL_INFO.load(deps.storage)?;
    info.to_normal(deps)
}

pub fn try_query_pool(deps: Deps) -> StdResult<PoolResponse> {
    let info = POOL_INFO.load(deps.storage)?;
    let contract_addr = deps.api.addr_humanize(&info.contract_addr)?;
    let assets: [Asset; 2] = info.query_pools(deps, contract_addr)?;
    let total_share: Uint128 = query_supply(
        &deps.querier,
        deps.api.addr_humanize(&info.liquidity_token)?,
    )?;

    Ok(PoolResponse {
        assets,
        total_share,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::Api;

    fn get_test_init_msg() -> InitMsg {
        InitMsg {
            pool_address: "test_pool".to_string(),
            bluna_hub_address: "test_mm".to_string(),
            bluna_address: "test_aust".to_string(),
            slippage: Decimal::percent(1u64),
            token_code_id: 0u64,
            vault_lp_token_name: None,
            vault_lp_token_symbol: None,
        }
    }

    #[test]
    fn test_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(1, res.messages.len());
    }

    #[test]
    fn test_init_with_non_default_vault_lp_token() {
        let mut deps = mock_dependencies(&[]);

        let custom_token_name = String::from("My LP Token");
        let custom_token_symbol = String::from("MyLP");

        // Define a custom Init Msg with the custom token info provided
        let msg = InitMsg {
            pool_address: "test_pool".to_string(),
            bluna_hub_address: "test_mm".to_string(),
            bluna_address: "test_aust".to_string(),
            slippage: Decimal::percent(1u64),
            token_code_id: 0u64,
            vault_lp_token_name: Some(custom_token_name.clone()),
            vault_lp_token_symbol: Some(custom_token_symbol.clone()),
        };

        // Prepare mock env
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let res = instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
        // Ensure we have 1 message
        assert_eq!(1, res.messages.len());
        // Verify the message is the one we expect but also that our custom provided token name and symbol were taken into account.
        assert_eq!(
            res.messages,
            vec![SubMsg {
                // Create LP token
                msg: WasmMsg::Instantiate {
                    admin: None,
                    code_id: msg.token_code_id,
                    msg: to_binary(&TokenInstantiateMsg {
                        name: custom_token_name.to_string(),
                        symbol: custom_token_symbol.to_string(),
                        decimals: 6,
                        initial_balances: vec![],
                        mint: Some(MinterResponse {
                            minter: env.contract.address.to_string(),
                            cap: None,
                        }),
                    })
                    .unwrap(),
                    funds: vec![],
                    label: "".to_string(),
                }
                .into(),
                gas_limit: None,
                id: INSTANTIATE_REPLY_ID,
                reply_on: ReplyOn::Success,
            }]
        );
    }

    #[test]
    fn test_set_slippage() {
        let mut deps = mock_dependencies(&[]);

        let msg = get_test_init_msg();
        let env = mock_env();
        let msg_info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();
        assert_eq!(1, res.messages.len());

        let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
        assert_eq!(info.slippage, Decimal::percent(1u64));

        let msg = HandleMsg::SetSlippage {
            slippage: Decimal::one(),
        };
        let _res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
        let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
        assert_eq!(info.slippage, Decimal::one());
    }
}
