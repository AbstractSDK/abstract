use abstract_std::profile_marketplace::{
    state::{Ask, Bid, ASK_HOOKS, BID_HOOKS, SALE_HOOKS},
    AskHookMsg, BidHookMsg, HookAction, SaleHookMsg,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, Reply, Response, StdResult, SubMsg, WasmMsg};

use crate::ContractError;

enum HookReply {
    Ask = 1,
    Sale,
    Bid,
}

impl From<u64> for HookReply {
    fn from(item: u64) -> Self {
        match item {
            1 => HookReply::Ask,
            2 => HookReply::Sale,
            3 => HookReply::Bid,
            _ => panic!("invalid reply type"),
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match HookReply::from(msg.id) {
        HookReply::Ask => {
            let res = Response::new()
                .add_attribute("action", "ask-hook-failed")
                .add_attribute("error", msg.result.unwrap_err());
            Ok(res)
        }
        HookReply::Sale => {
            let res = Response::new()
                .add_attribute("action", "sale-hook-failed")
                .add_attribute("error", msg.result.unwrap_err());
            Ok(res)
        }
        HookReply::Bid => {
            let res = Response::new()
                .add_attribute("action", "bid-hook-failed")
                .add_attribute("error", msg.result.unwrap_err());
            Ok(res)
        }
    }
}

pub fn prepare_ask_hook(deps: Deps, ask: &Ask, action: HookAction) -> StdResult<Vec<SubMsg>> {
    let submsgs = ASK_HOOKS.prepare_hooks(deps.storage, |h| {
        let msg = AskHookMsg { ask: ask.clone() };
        let execute = WasmMsg::Execute {
            contract_addr: h.to_string(),
            msg: msg.into_json_binary(action.clone())?,
            funds: vec![],
        };
        Ok(SubMsg::reply_on_error(execute, HookReply::Ask as u64))
    })?;

    Ok(submsgs)
}

pub fn prepare_sale_hook(deps: Deps, ask: &Ask, buyer: Addr) -> StdResult<Vec<SubMsg>> {
    let submsgs = SALE_HOOKS.prepare_hooks(deps.storage, |h| {
        let msg = SaleHookMsg {
            token_id: ask.token_id.to_string(),
            seller: ask.seller.to_string(),
            buyer: buyer.to_string(),
        };
        let execute = WasmMsg::Execute {
            contract_addr: h.to_string(),
            msg: msg.into_json_binary()?,
            funds: vec![],
        };
        Ok(SubMsg::reply_on_error(execute, HookReply::Sale as u64))
    })?;

    Ok(submsgs)
}

pub fn prepare_bid_hook(deps: Deps, bid: &Bid, action: HookAction) -> StdResult<Vec<SubMsg>> {
    let submsgs = BID_HOOKS.prepare_hooks(deps.storage, |h| {
        let msg = BidHookMsg { bid: bid.clone() };
        let execute = WasmMsg::Execute {
            contract_addr: h.to_string(),
            msg: msg.into_json_binary(action.clone())?,
            funds: vec![],
        };
        Ok(SubMsg::reply_on_error(execute, HookReply::Bid as u64))
    })?;

    Ok(submsgs)
}
