use abstract_sdk::std::{profile_marketplace::*, PROFILE_MARKETPLACE};

use abstract_std::profile_marketplace::state::{
    SudoParams, ASK_HOOKS, BID_HOOKS, MAX_FEE_BPS, PROFILE_COLLECTION, PROFILE_MINTER, SALE_HOOKS, SUDO_PARAMS
};
use cosmwasm_std::{
    to_json_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;
use semver::Version;
use state::VERSION_CONTROL;

use crate::{commands::*, ContractError};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, PROFILE_MARKETPLACE, CONTRACT_VERSION)?;
    if msg.trading_fee_bps > MAX_FEE_BPS {
        return Err(ContractError::InvalidTradingFeeBps(msg.trading_fee_bps));
    }

    let params = SudoParams {
        trading_fee_percent: Decimal::percent(msg.trading_fee_bps) / Uint128::from(100u128),
        min_price: msg.min_price,
        ask_interval: msg.ask_interval,
    };

    SUDO_PARAMS.save(deps.storage, &params)?;
    // Saves the profile minter & collection to internal state
    
    PROFILE_MINTER.save(deps.storage, &msg.factory)?;
    PROFILE_COLLECTION.save(deps.storage, &msg.collection)?;
    VERSION_CONTROL.save(deps.storage, &msg.version_control)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("minter", msg.factory)
        .add_attribute("collection", msg.collection))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::SetAsk { token_id, seller, account_id } => {
            execute_set_ask(deps, env, info, &token_id, api.addr_validate(&seller)?, account_id)
        }
        ExecuteMsg::RemoveAsk { token_id } => execute_remove_ask(deps, info, &token_id),
        ExecuteMsg::UpdateAsk { token_id, seller } => {
            execute_update_ask(deps, info, &token_id, api.addr_validate(&seller)?)
        }
        ExecuteMsg::SetBid {token_id, new_gov,account_id } => execute_set_bid(deps, env, info, &token_id, new_gov,account_id),
        ExecuteMsg::RemoveBid { token_id } => execute_remove_bid(deps, env, info, &token_id),
        ExecuteMsg::AcceptBid { token_id, bidder } => {
            execute_accept_bid(deps, env, info, &token_id, api.addr_validate(&bidder)?)
        }
        ExecuteMsg::FundRenewal { token_id } => execute_fund_renewal(deps, info, &token_id),
        ExecuteMsg::RefundRenewal { token_id } => execute_refund_renewal(deps, info, &token_id),
        ExecuteMsg::ProcessRenewals { time } => execute_process_renewal(deps, env, time),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::Ask { token_id } => to_json_binary(&query_ask(deps, token_id)?),
        QueryMsg::Asks { start_after, limit } => {
            to_json_binary(&query_asks(deps, start_after, limit)?)
        }
        QueryMsg::AsksBySeller {
            seller,
            start_after,
            limit,
        } => to_json_binary(&query_asks_by_seller(
            deps,
            api.addr_validate(&seller)?,
            start_after,
            limit,
        )?),
        QueryMsg::AskCount {} => to_json_binary(&query_ask_count(deps)?),
        QueryMsg::Bid { token_id, bidder } => {
            to_json_binary(&query_bid(deps, token_id, api.addr_validate(&bidder)?)?)
        }
        QueryMsg::Bids {
            token_id,
            start_after,
            limit,
        } => to_json_binary(&query_bids(deps, token_id, start_after, limit)?),
        QueryMsg::BidsByBidder {
            bidder,
            start_after,
            limit,
        } => to_json_binary(&query_bids_by_bidder(
            deps,
            api.addr_validate(&bidder)?,
            start_after,
            limit,
        )?),
        QueryMsg::BidsSortedByPrice { start_after, limit } => {
            to_json_binary(&query_bids_sorted_by_price(deps, start_after, limit)?)
        }
        QueryMsg::ReverseBidsSortedByPrice {
            start_before,
            limit,
        } => to_json_binary(&reverse_query_bids_sorted_by_price(
            deps,
            start_before,
            limit,
        )?),
        QueryMsg::BidsForSeller {
            seller,
            start_after,
            limit,
        } => to_json_binary(&query_bids_for_seller(
            deps,
            api.addr_validate(&seller)?,
            start_after,
            limit,
        )?),
        QueryMsg::HighestBid { token_id } => to_json_binary(&query_highest_bid(deps, token_id)?),
        QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
        QueryMsg::AskHooks {} => to_json_binary(&ASK_HOOKS.query_hooks(deps)?),
        QueryMsg::BidHooks {} => to_json_binary(&BID_HOOKS.query_hooks(deps)?),
        QueryMsg::SaleHooks {} => to_json_binary(&SALE_HOOKS.query_hooks(deps)?),
        QueryMsg::RenewalQueue { time } => to_json_binary(&query_renewal_queue(deps, time)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let current_version = cw2::get_contract_version(deps.storage)?;
    if current_version.contract != PROFILE_MARKETPLACE {
        return Err(StdError::generic_err("Cannot upgrade to a different contract").into());
    }
    let version: Version = current_version
        .version
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;
    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;

    if version > new_version {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    // if same version return
    if version == new_version {
        return Ok(Response::new());
    }

    // set new contract version
    set_contract_version(deps.storage, PROFILE_MARKETPLACE, CONTRACT_VERSION)?;
    Ok(Response::new())
}
