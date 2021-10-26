use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::{Cw20HookMsg, ExecuteMsg as TerraswapMsg, PoolResponse};
use terraswap::querier::query_token_balance;
use white_whale::deposit_info::DepositInfo;
// Missing functions: query_pool, query_lp_token
use std::cmp::min;
use white_whale::query::terraswap::{query_lp_token, query_pool};

use crate::error::TerraswapWrapperError;
use crate::msg::{ExecuteMsg, InitMsg, QueryMsg, WithdrawableProfitsResponse};
use crate::state::{State, ADMIN, DEPOSIT_INFO, STATE, TRADER};

type TerraswapWrapperResult = Result<Response, TerraswapWrapperError>;

/*  This contract implements a way to interact with a liquidity pool.


Terraswap-wrapper contract can be used to implement protocol owned liquidity.
*/
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    if msg.max_deposit.info != msg.min_profit.info {
        return Err(StdError::generic_err(
            "Different asset infos for max_deposit and min_profit.",
        ));
    }
    let lp_token_addr = query_lp_token(
        deps.as_ref(),
        deps.api.addr_validate(&msg.terraswap_pool_addr)?,
    )?;
    let state = State {
        terraswap_pool_addr: deps.api.addr_canonicalize(&msg.terraswap_pool_addr)?,
        lp_token_addr: deps.api.addr_canonicalize(&lp_token_addr)?,
        max_deposit: msg.max_deposit.clone(),
        min_profit: msg.min_profit,
        slippage: msg.slippage,
    };

    STATE.save(deps.storage, &state)?;
    DEPOSIT_INFO.save(
        deps.storage,
        &DepositInfo {
            asset_info: msg.max_deposit.info,
        },
    )?;
    TRADER.set(deps.storage, Some(deps.api.addr_validate(&msg.trader)?))?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> TerraswapWrapperResult {
    match msg {
        ExecuteMsg::Deposit { funds } => deposit(deps, env, info, funds),
        ExecuteMsg::Withdraw { funds } => withdraw(deps, info, funds),
        ExecuteMsg::SetTrader { trader } => set_trader(deps, info, trader),
        ExecuteMsg::SetMaxDeposit { asset } => set_max_deposit(deps, info, asset),
        ExecuteMsg::SetMinProfit { asset } => set_min_profit(deps, info, asset),
        ExecuteMsg::Spend { recipient, amount } => spend(deps.as_ref(), info, recipient, amount),
    }
}

// Lets admin transfer assets that belong to this address.
fn spend(
    deps: Deps,
    info: MessageInfo,
    recipient: String,
    amount: Asset,
) -> TerraswapWrapperResult {
    ADMIN.assert_admin(deps, &info.sender)?;
    Ok(Response::new()
        .add_message(amount.into_msg(&deps.querier, deps.api.addr_validate(&recipient)?)?))
}

// Deposit asset into LP pool and receive LP tokens in return.
// Does not care about max_deposit
fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    funds: Vec<Asset>,
) -> TerraswapWrapperResult {
    TRADER.assert_trader(deps.storage, &info.sender)?;

    let state = STATE.load(deps.storage)?;
    let pools = query_pool(
        deps.as_ref(),
        deps.api.addr_humanize(&state.terraswap_pool_addr)?,
    )?
    .assets;

    // Check if both provided assets match the pool and store amounts to deposits var
    let deposits: [Uint128; 2] = [
        funds
            .iter()
            .find(|a| a.info.equal(&pools[0].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
        funds
            .iter()
            .find(|a| a.info.equal(&pools[1].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
    ];

    // If the pool is a cw20 contract, then we need to execute TransferFrom msg to move the funds to the cw20 contract
    // Flow of funds:
    // Caller of this function -> this contract -> LP pool (terraswap contract)

    let mut messages: Vec<CosmosMsg> = vec![];
    for (i, pool) in pools.iter().enumerate() {
        // If pool is cw20
        if let AssetInfo::Token { contract_addr, .. } = &pool.info {
            // Construct msg to send deposit funds from owner to this address. (Caller of this function -> this contract)
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount: deposits[i],
                })?,
                funds: vec![],
            }));

            // Allow terraswap pool to move funds from contract to pool (this contract -> LP pool)
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                    spender: deps
                        .api
                        .addr_humanize(&state.terraswap_pool_addr)?
                        .to_string(),
                    amount: deposits[i],
                    expires: None,
                })?,
                funds: vec![],
            }))
        }
    }

    let deposit_msg = TerraswapMsg::ProvideLiquidity {
        assets: [funds[0].clone(), funds[1].clone()],
        slippage_tolerance: Some(state.slippage),
        receiver: None,
    };

    // Call cw20 to take the funds and put them in the LP
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps
            .api
            .addr_humanize(&state.terraswap_pool_addr)?
            .to_string(),
        funds: info.funds,
        msg: to_binary(&deposit_msg)?,
    });
    Ok(Response::default().add_messages(messages).add_message(msg))
}

// Withdraw liquidity from pool. Can only be called by the trader.
// Should be used together with query_withdrawable_profits to withdraw profits to another contract.

fn withdraw(deps: DepsMut, info: MessageInfo, funds: Vec<Asset>) -> TerraswapWrapperResult {
    TRADER.assert_trader(deps.storage, &info.sender)?;

    let state = STATE.load(deps.storage)?;
    let pool_response = query_pool(
        deps.as_ref(),
        deps.api.addr_humanize(&state.terraswap_pool_addr)?,
    )?;
    let pools = pool_response.assets;

    // Check if requested funds match pool and store requested amounts.
    let deposits: [Uint128; 2] = [
        funds
            .iter()
            .find(|a| a.info.equal(&pools[0].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
        funds
            .iter()
            .find(|a| a.info.equal(&pools[1].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
    ];

    // Sender share of LP pool is minimum of requested/totalAsset -> Will always withdraw funds symmetrically!
    let share = min(
        Decimal::from_ratio(deposits[0], pools[0].amount),
        Decimal::from_ratio(deposits[1], pools[1].amount),
    );
    let lp_amount = pool_response.total_share * share;

    // Call the LP token address (msg) to send the funds (cw20_msg) by executing (withdraw_msg) on the LP address.
    let withdraw_msg = Cw20HookMsg::WithdrawLiquidity {};

    let cw20_msg = Cw20ExecuteMsg::Send {
        contract: deps
            .api
            .addr_humanize(&state.terraswap_pool_addr)?
            .to_string(),
        amount: lp_amount,
        msg: to_binary(&withdraw_msg)?,
    };

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.lp_token_addr)?.to_string(),
        funds: vec![],
        msg: to_binary(&cw20_msg)?,
    });

    Ok(Response::default().add_message(msg))
}

// Sets the trader
fn set_trader(deps: DepsMut, info: MessageInfo, trader: String) -> TerraswapWrapperResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    TRADER.set(deps.storage, Some(deps.api.addr_validate(&trader)?))?;
    Ok(Response::default()
        .add_attribute("action", "set_trader")
        .add_attribute("trader", trader))
}

// Sets max deposit. This sets the cap on how much liquidity the trader can add. (protocol owned liquidity)
fn set_max_deposit(deps: DepsMut, info: MessageInfo, asset: Asset) -> TerraswapWrapperResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    DEPOSIT_INFO.load(deps.storage)?.assert(&asset.info)?;
    let mut state = STATE.load(deps.storage)?;
    state.max_deposit = asset.clone();
    STATE.save(deps.storage, &state)?;

    Ok(Response::default()
        .add_attribute("action", "set_max_deposit")
        .add_attribute("asset", asset.to_string()))
}

// Set min profit.
fn set_min_profit(deps: DepsMut, info: MessageInfo, asset: Asset) -> TerraswapWrapperResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    DEPOSIT_INFO.load(deps.storage)?.assert(&asset.info)?;
    let mut state = STATE.load(deps.storage)?;
    state.min_profit = asset.clone();
    STATE.save(deps.storage, &state)?;

    Ok(Response::default()
        .add_attribute("action", "set_min_profit")
        .add_attribute("asset", asset.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::WithdrawableProfits {} => to_binary(&query_withdrawable_profits(deps, env)?),
    }
}

// Returns total value of LP, given LP is [UST, X]
fn query_value(deps: Deps, info: AssetInfo, share: Decimal) -> StdResult<Asset> {
    let state = STATE.load(deps.storage)?;
    let pool: PoolResponse = query_pool(deps, deps.api.addr_humanize(&state.terraswap_pool_addr)?)?;

    if pool.assets[0].info == info {
        let price = Decimal::from_ratio(pool.assets[0].amount, pool.assets[1].amount); // price [UST/X]
        let mut value = pool.assets[0].amount * share;
        value += pool.assets[1].amount * share * price;
        return Ok(Asset {
            info,
            amount: value,
        });
    }

    let price = Decimal::from_ratio(pool.assets[1].amount, pool.assets[0].amount); // price [X/UST]
    let mut value = pool.assets[1].amount * share;
    value += pool.assets[0].amount * share * price;
    Ok(Asset {
        info,
        amount: value,
    })
}

// Withdrawable profit is the value of the holdings - max_deposit in either the asset denom(X) or the base denom(UST)
// as set by the state variables.

fn query_withdrawable_profits(deps: Deps, env: Env) -> StdResult<WithdrawableProfitsResponse> {
    let state = STATE.load(deps.storage)?;
    let pool: PoolResponse = query_pool(deps, deps.api.addr_humanize(&state.terraswap_pool_addr)?)?;
    let lp_balance: Uint128 = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&state.lp_token_addr)?,
        env.contract.address,
    )?;
    let share = Decimal::from_ratio(lp_balance, pool.total_share);
    let total_value = query_value(deps, state.max_deposit.info.clone(), share)?;

    let mut withdrawable_profits = Uint128::zero();
    let mut withdrawable_profit_share = Decimal::zero();

    if total_value.amount > Uint128::zero()
        && total_value.amount > state.max_deposit.amount + state.min_profit.amount
    {
        withdrawable_profits = total_value.amount - state.max_deposit.amount;
        withdrawable_profit_share = Decimal::from_ratio(withdrawable_profits, total_value.amount);
    }
    Ok(WithdrawableProfitsResponse {
        amount: Asset {
            info: state.max_deposit.info,
            amount: withdrawable_profits,
        },
        lp_amount: lp_balance * withdrawable_profit_share,
    })
}
