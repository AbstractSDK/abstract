use crate::error::CommunityFundError;
use crate::msg::InstantiateMsg;
use crate::state::{State, ADMIN, STATE};
use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as PairExecuteMsg;
use terraswap::querier::{query_balance, query_token_balance};
use white_whale::anchor::try_deposit_to_anchor_as_submsg;
use white_whale::community_fund::msg::{ConfigResponse, ExecuteMsg, QueryMsg};
use white_whale::denom::UST_DENOM;
use white_whale::msg::AnchorMsg;
use white_whale::query::anchor::query_aust_exchange_rate;

const ANCHOR_DEPOSIT_REPLY_ID: u64 = 2;
const ANCHOR_WITHDRAW_REPLY_ID: u64 = 3;

/*
    The Community fund holds the protocol treasury and has control over the protocol owned liquidity.
    It is controlled by the governance contract and serves to grow its holdings and give grants to proposals.
*/

type CommunityFundResult = Result<Response, CommunityFundError>;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        whale_token_addr: deps.api.addr_canonicalize(&msg.whale_token_addr)?,
        whale_pool_addr: deps.api.addr_canonicalize(&msg.whale_pair_addr)?,
        anchor_money_market_addr: deps.api.addr_canonicalize(&msg.anchor_money_market_addr)?,
        aust_addr: deps.api.addr_canonicalize(&msg.aust_addr)?,
        deposits_in_uusd: Uint128::zero(),
        last_deposit_in_uusd: Uint128::zero(),
        anchor_deposit_threshold: msg.anchor_deposit_threshold,
        anchor_withdraw_threshold: msg.anchor_withdraw_threshold,
    };

    STATE.save(deps.storage, &state)?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> CommunityFundResult {
    match msg {
        ExecuteMsg::Spend { recipient, amount } => {
            spend_whale(deps.as_ref(), info, recipient, amount)
        }
        ExecuteMsg::Burn { amount } => burn_whale(deps.as_ref(), info, amount),
        ExecuteMsg::Deposit {} => deposit_or_spend_interest(deps, &env, info),
        ExecuteMsg::UpdateAdmin { admin } => {
            let admin_addr = deps.api.addr_validate(&admin)?;
            Ok(ADMIN.execute_update_admin(deps, info, Some(admin_addr))?)
        }
        ExecuteMsg::UpdateAnchorDepositThreshold { threshold } => {
            set_anchor_deposit_threshold(deps, info, threshold)
        }
        ExecuteMsg::UpdateAnchorWithdrawThreshold { threshold } => {
            set_anchor_withdraw_threshold(deps, info, threshold)
        }
    }
}

// The deposit threshold determines the minimum amount of UST the contract has to own before it can deposit those funds into Anchor
pub fn set_anchor_deposit_threshold(
    deps: DepsMut,
    info: MessageInfo,
    threshold: Uint128,
) -> CommunityFundResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let mut state = STATE.load(deps.storage)?;
    state.anchor_deposit_threshold = threshold;
    STATE.save(deps.storage, &state)?;
    Ok(Response::default())
}

// The withdraw threshold determines the minimum amount of aUST the contract has to own before it can withdraw those funds from Anchor
pub fn set_anchor_withdraw_threshold(
    deps: DepsMut,
    info: MessageInfo,
    threshold: Uint128,
) -> CommunityFundResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let mut state = STATE.load(deps.storage)?;
    state.anchor_withdraw_threshold = threshold;
    STATE.save(deps.storage, &state)?;
    Ok(Response::default())
}

// This function allows the community fund to buy whale tokens from the terraswap market.
pub fn buy_whale(deps: Deps, env: &Env) -> CommunityFundResult {
    let state = STATE.load(deps.storage)?;
    let ust_amount = query_balance(
        &deps.querier,
        env.contract.address.clone(),
        UST_DENOM.to_string(),
    )?;
    if ust_amount == Uint128::zero() {
        return Err(CommunityFundError::NotEnoughFunds {});
    }
    let mut offer = Asset {
        info: AssetInfo::NativeToken {
            denom: UST_DENOM.to_string(),
        },
        amount: ust_amount,
    };
    let ust = offer.deduct_tax(&deps.querier)?;
    offer.amount = ust.amount;

    let buy_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.whale_pool_addr)?.to_string(),
        funds: vec![ust],
        msg: to_binary(&PairExecuteMsg::Swap {
            offer_asset: offer,
            belief_price: None,
            max_spread: None,
            to: None,
        })?,
    });

    Ok(Response::new().add_message(buy_msg))
}

// Deposits UST funds into Anchor if funds > anchor_deposit_threshold
pub fn deposit(deps: DepsMut, env: &Env) -> CommunityFundResult {
    let mut state = STATE.load(deps.storage)?;

    let deposit = deps
        .querier
        .query_balance(&env.contract.address, UST_DENOM)?;
    if deposit.amount < state.anchor_deposit_threshold {
        return Ok(Response::default());
    }

    let deposit_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: deposit.denom,
        },
        amount: deposit.amount,
    };

    state.last_deposit_in_uusd = deposit_asset.deduct_tax(&deps.querier)?.amount;
    STATE.save(deps.storage, &state)?;
    Ok(try_deposit_to_anchor_as_submsg(
        deps.api
            .addr_humanize(&state.anchor_money_market_addr)?
            .to_string(),
        deposit_asset.deduct_tax(&deps.querier)?,
        ANCHOR_DEPOSIT_REPLY_ID,
    )?)
}

//
pub fn deposit_or_spend_interest(
    deps: DepsMut,
    env: &Env,
    msg_info: MessageInfo,
) -> CommunityFundResult {
    if msg_info.funds.len() > 1 {
        return Err(CommunityFundError::DepositTooManyTokens {});
    }
    if msg_info.funds[0].denom != UST_DENOM {
        return Err(CommunityFundError::DepositOnlyUST {});
    }

    let state = STATE.load(deps.storage)?;
    let aust_value_in_uusd = get_aust_value_in_uusd(deps.as_ref(), env)?;
    // If anchor deposit value < total UST deposited + threshold then deposit more into Anchor.
    if aust_value_in_uusd < state.deposits_in_uusd + state.anchor_withdraw_threshold {
        return deposit(deps, env);
    }
    // Else, calculate earned interest and buy WHALE with it.
    spend_interest(deps, aust_value_in_uusd, msg_info.funds[0].amount)
}

pub fn get_aust_value_in_uusd(deps: Deps, env: &Env) -> StdResult<Uint128> {
    let state = STATE.load(deps.storage)?;
    let aust_amount = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&state.aust_addr)?,
        env.contract.address.clone(),
    )?;
    let aust_exchange_rate = query_aust_exchange_rate(
        deps,
        deps.api
            .addr_humanize(&state.anchor_money_market_addr)?
            .to_string(),
    )?;

    Ok(aust_exchange_rate * aust_amount)
}

// Withdraw interest earned.
pub fn spend_interest(
    deps: DepsMut,
    aust_value_in_ust: Uint128,
    deposit_amount: Uint128,
) -> CommunityFundResult {
    let state = STATE.load(deps.storage)?;
    // withdraw_amount = earned_interest - specified amount
    let withdraw_amount = (aust_value_in_ust - state.deposits_in_uusd) - deposit_amount;
    let withdraw_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.aust_addr)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Send {
            contract: deps
                .api
                .addr_humanize(&state.anchor_money_market_addr)?
                .to_string(),
            amount: withdraw_amount,
            msg: to_binary(&AnchorMsg::RedeemStable {})?,
        })?,
        funds: vec![],
    });
    Ok(Response::new().add_submessage(SubMsg {
        msg: withdraw_msg,
        gas_limit: None,
        id: ANCHOR_WITHDRAW_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
}

// Call burn on WHALE cw20 token
pub fn burn_whale(deps: Deps, info: MessageInfo, amount: Uint128) -> CommunityFundResult {
    ADMIN.assert_admin(deps, &info.sender)?;
    let state = STATE.load(deps.storage)?;

    Ok(
        Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&state.whale_token_addr)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
        })),
    )
}

// Transfer WHALE to specified recipient
pub fn spend_whale(
    deps: Deps,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> CommunityFundResult {
    ADMIN.assert_admin(deps, &info.sender)?;
    let state = STATE.load(deps.storage)?;
    Ok(
        Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&state.whale_token_addr)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer { recipient, amount })?,
        })),
    )
}

// Catches submessage calls. Either updating deposted UST amount or buying whale with earned interest.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> CommunityFundResult {
    if msg.id == ANCHOR_DEPOSIT_REPLY_ID {
        let mut state = STATE.load(deps.storage)?;
        state.deposits_in_uusd += state.last_deposit_in_uusd;
        state.last_deposit_in_uusd = Uint128::zero();
        STATE.save(deps.storage, &state)?;
        return Ok(Response::default());
    }
    if msg.id == ANCHOR_WITHDRAW_REPLY_ID {
        return buy_whale(deps.as_ref(), &env);
    }
    Ok(Response::default())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => Ok(to_binary(&ADMIN.query_admin(deps)?)?),
        QueryMsg::Config {} => query_config(deps),
    }
}

pub fn query_config(deps: Deps) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    to_binary(&ConfigResponse {
        token_addr: deps.api.addr_humanize(&state.whale_token_addr)?,
        ust_pool_addr: deps.api.addr_humanize(&state.whale_pool_addr)?,
        anchor_money_market_addr: deps.api.addr_humanize(&state.anchor_money_market_addr)?,
        aust_addr: deps.api.addr_humanize(&state.aust_addr)?,
        anchor_deposit_threshold: state.anchor_deposit_threshold,
        anchor_withdraw_threshold: state.anchor_withdraw_threshold,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{from_binary, Api};
    use cw_controllers::AdminResponse;

    fn get_test_init_msg() -> InstantiateMsg {
        InstantiateMsg {
            whale_token_addr: "whale token".to_string(),
            whale_pair_addr: "terraswap pair".to_string(),
            anchor_money_market_addr: "anchor money market".to_string(),
            aust_addr: "aust".to_string(),
            anchor_deposit_threshold: Uint128::from(1000000000u64),
            anchor_withdraw_threshold: Uint128::from(1000000000u64),
        }
    }

    #[test]
    fn proper_initialization() {
        // Set dependencies, make the message, make the message info.
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        // Simulate transaction.
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        // TODO: implement query
    }

    #[test]
    fn test_set_anchor_deposit_threshold() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        let _res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
        let state = STATE.load(&deps.storage).unwrap();
        assert_ne!(state.anchor_deposit_threshold, Uint128::from(3u64));
        let _res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::UpdateAnchorDepositThreshold {
                threshold: Uint128::from(3u64),
            },
        )
        .unwrap();
        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.anchor_deposit_threshold, Uint128::from(3u64));
    }

    #[test]
    fn test_set_anchor_withdraw_threshold() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        let _res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
        let state = STATE.load(&deps.storage).unwrap();
        assert_ne!(state.anchor_withdraw_threshold, Uint128::from(3u64));
        let _res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::UpdateAnchorWithdrawThreshold {
                threshold: Uint128::from(3u64),
            },
        )
        .unwrap();
        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.anchor_withdraw_threshold, Uint128::from(3u64));
    }

    #[test]
    fn test_only_owner_can_change_anchor_deposit_threshold() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };
        let other_info = MessageInfo {
            sender: deps.api.addr_validate("other").unwrap(),
            funds: vec![],
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
        let _res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
        let res = execute(
            deps.as_mut(),
            env,
            other_info,
            ExecuteMsg::UpdateAnchorDepositThreshold {
                threshold: Uint128::from(3u64),
            },
        );
        match res {
            Err(_) => {}
            Ok(_) => panic!("unexpected"),
        }
    }

    #[test]
    fn test_only_owner_can_change_anchor_withdraw_threshold() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };
        let other_info = MessageInfo {
            sender: deps.api.addr_validate("other").unwrap(),
            funds: vec![],
        };

        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
        let _res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
        let res = execute(
            deps.as_mut(),
            env,
            other_info,
            ExecuteMsg::UpdateAnchorWithdrawThreshold {
                threshold: Uint128::from(3u64),
            },
        );
        match res {
            Err(_) => {}
            Ok(_) => panic!("unexpected"),
        }
    }

    #[test]
    fn test_config_query() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let creator_info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let init_res = instantiate(deps.as_mut(), env.clone(), creator_info.clone(), msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let q_res: ConfigResponse =
            from_binary(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(
            q_res.token_addr,
            deps.api.addr_validate("whale token").unwrap()
        )
    }

    #[test]
    fn test_admin_query() {
        let mut deps = mock_dependencies(&[]);
        let msg = get_test_init_msg();
        let env = mock_env();
        let creator_info = MessageInfo {
            sender: deps.api.addr_validate("creator").unwrap(),
            funds: vec![],
        };

        let init_res = instantiate(deps.as_mut(), env.clone(), creator_info.clone(), msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let q_res: AdminResponse =
            from_binary(&query(deps.as_ref(), env, QueryMsg::Admin {}).unwrap()).unwrap();
        assert_eq!(
            q_res.admin.unwrap(),
            deps.api.addr_validate("creator").unwrap()
        )
    }
}
