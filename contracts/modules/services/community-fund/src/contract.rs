use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terraswap::querier::query_token_balance;

use abstract_os::community_fund::msg::{ConfigResponse, ExecuteMsg, QueryMsg};

use crate::error::CommunityFundError;
use crate::msg::InstantiateMsg;
use crate::state::{State, ADMIN, STATE};

/*
    The Community fund holds the protocol proxy and has control over the protocol owned liquidity.
    It is controlled by the governance contract and serves to grow its holdings and give grants to proposals.
*/

type CommunityFundResult = Result<Response, CommunityFundError>;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.api.addr_validate(&msg.whale_token_addr)?;

    let state = State {
        whale_token_addr: deps.api.addr_canonicalize(&msg.whale_token_addr)?,
    };

    STATE.save(deps.storage, &state)?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> CommunityFundResult {
    match msg {
        ExecuteMsg::Spend { recipient, amount } => {
            spend_whale(deps.as_ref(), info, recipient, amount)
        }
        ExecuteMsg::Burn { amount } => burn_whale(deps.as_ref(), info, amount),
        ExecuteMsg::SetAdmin { admin } => {
            let new_admin_addr = deps.api.addr_validate(&admin)?;
            let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
            ADMIN.execute_update_admin::<Empty>(deps, info, Some(new_admin_addr))?;
            Ok(Response::default()
                .add_attribute("previous admin", previous_admin)
                .add_attribute("admin", admin))
        }
    }
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

    let account_addr = deps.api.addr_validate(&info.sender.to_string())?;

    let fund_whale_balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&state.whale_token_addr)?,
        account_addr,
    )?;
    if amount > fund_whale_balance {
        return Err(CommunityFundError::InsufficientFunds(
            amount,
            fund_whale_balance,
        ));
    };

    Ok(
        Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&state.whale_token_addr)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer { recipient, amount })?,
        })),
    )
}

// Call burn on WHALE cw20 token
pub fn burn_whale(deps: Deps, info: MessageInfo, amount: Uint128) -> CommunityFundResult {
    ADMIN.assert_admin(deps, &info.sender)?;
    let state = STATE.load(deps.storage)?;

    let account_addr = deps.api.addr_validate(&info.sender.to_string())?;

    let fund_whale_balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&state.whale_token_addr)?,
        account_addr,
    )?;

    if amount > fund_whale_balance {
        return Err(CommunityFundError::InsufficientFunds(
            amount,
            fund_whale_balance,
        ));
    };

    Ok(
        Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&state.whale_token_addr)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
        })),
    )
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
    })
}
