use cosmwasm_std::{ entry_point, CanonicalAddr,
    from_binary, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, WasmMsg, Uint128, Decimal, SubMsg, Reply, ReplyOn
};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::Cw20HookMsg;
use terraswap::querier::{query_balance, query_token_balance, query_supply};
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;

use crate::vault_asset::VaultAsset;
use cw_storage_plus::{Map};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};

use white_whale::msg::{create_terraswap_msg, VaultQueryMsg as QueryMsg};
use white_whale::query::terraswap::simulate_swap as simulate_terraswap_swap;

use crate::msg::{HandleMsg, InitMsg, PoolResponse};
use crate::state::{State, STATE, VAULT_ASSETS, LUNA_DENOM};

use crate::response::MsgInstantiateContractResponse;


const INSTANTIATE_REPLY_ID: u64 = 1;


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    let state = State {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        traders: vec![],
    };

    STATE.save(deps.storage, &state)?;

    let vault_assets: &Map<AssetInfo, VaultAsset> = &Map::new("people");
    VAULT_ASSETS.save(deps.storage, vault_assets)?;

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
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> StdResult<Response> {
    match msg {
        HandleMsg::Receive(msg) => receive_cw20(deps, info, msg),
        HandleMsg::Swap{ amount } => try_swap(deps, info, amount),
        HandleMsg::ProvideLiquidity{ asset } => try_provide_liquidity(deps, info, asset),
        HandleMsg::SetSlippage{ slippage } => set_slippage(deps, info, slippage),
    }
}

/// try_swap attempts to perform a swap between uluna and bluna, depending on what coin is offered. 
pub fn try_swap(
    deps: DepsMut,
    info: MessageInfo,
    offer_coin: Coin,
) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;
    if deps.api.addr_canonicalize(&info.sender.to_string())? != state.trader {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let slippage = (POOL_INFO.load(deps.storage)?).slippage;
    let belief_price = Decimal::from_ratio(simulate_terraswap_swap(deps.as_ref(), deps.api.addr_humanize(&state.pool_address)?, offer_coin.clone())?, offer_coin.amount);
    
    // Sell luna and buy bluna
    let msg = if offer_coin.denom == "uluna" {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.pool_address.to_string(),
            funds: vec![offer_coin.clone()],
            msg: to_binary(&create_terraswap_msg(offer_coin, belief_price, Some(slippage)))?,
        })
    // Or sell bluna and buy luna
    } else {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.bluna_address.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send{
                contract: state.pool_address.to_string(),
                amount: offer_coin.amount,
                msg: to_binary(&create_terraswap_msg(offer_coin, belief_price, Some(slippage)))?
            })?
        })
    };

    Ok(Response::new().add_message(msg))
}

pub fn compute_total_deposits(
    deps: Deps,
    info: &PoolInfoRaw
) -> StdResult<(Uint128,Uint128)> {
    let state = STATE.load(deps.storage)?;
    let contract_address = deps.api.addr_humanize(&info.contract_addr)?;
    let deposits_in_luna = query_balance(&deps.querier, contract_address.clone(), LUNA_DENOM.to_string())?;
    let deposits_in_bluna = query_token_balance(&deps.querier, deps.api.addr_humanize(&state.bluna_address)?, contract_address)?;
    Ok((deposits_in_luna, deposits_in_bluna))
}

pub fn try_withdraw_liquidity(
    deps: DepsMut,
    sender: String,
    amount: Uint128,
) -> StdResult<Response> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let liquidity_addr = deps.api.addr_humanize(&info.liquidity_token)?;
    let asset_info: AssetInfo = info.asset_infos[1].to_normal(deps.api)?;

    let total_share: Uint128 = query_supply(&deps.querier, liquidity_addr)?;
    let (pool_luna, pool_bluna) = compute_total_deposits(deps.as_ref(), &info)?;

    let share_ratio: Decimal = Decimal::from_ratio(amount, total_share);

    // amount of luna to return
    let refund_base: Asset = Asset{
        info: AssetInfo::NativeToken{ denom: get_stable_denom(deps.as_ref())? },
        amount: pool_luna * share_ratio
    };
    // luna withdraw msg
    let refund_base_msg = {
        CosmosMsg::Bank(BankMsg::Send {
            to_address: sender.clone(),
            amount: vec![refund_base.deduct_tax(&deps.querier)?],
        })
    };

    // amount of bluna to return
    let asset_address = match asset_info {
        AssetInfo::Token{contract_addr} => contract_addr,
        _ => return Err(StdError::generic_err("Specified token is a Native Token!")),
    };
    let asset_amount = pool_bluna * share_ratio;

    let refund_asset_msg = {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: asset_address,
            msg: to_binary(&Cw20ExecuteMsg::Transfer { recipient: sender, amount: asset_amount})?,
            funds: vec![],
       })
    };

    // Burn vault lp token 
    let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: info.liquidity_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
        funds: vec![],
    });
    Ok(Response::new()
        .add_message(refund_base_msg)
        .add_message(refund_asset_msg)
        .add_message(burn_msg)
        .add_attribute("action", "withdraw_liquidity")
        .add_attribute("withdrawn_share", &amount.to_string())
        .add_attribute("refund_base", format!(" {}", refund_base))
        .add_attribute("refund_asset", format!(" {}", asset_amount))
    )
}

pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let pool_info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Swap {
            ..
        } => {
            Err(StdError::generic_err("no swaps can be performed in this pool"))
        }
        Cw20HookMsg::WithdrawLiquidity {} => {
            if deps.api.addr_canonicalize(&info.sender.to_string())? != pool_info.liquidity_token {
                return Err(StdError::generic_err("Unauthorized"));
            }

            try_withdraw_liquidity(deps, cw20_msg.sender.to_string(), cw20_msg.amount)
        }
    }
}

pub fn get_stable_denom(
    deps: Deps,
) -> StdResult<String> {
    let info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let stable_info = info.asset_infos[0].to_normal(deps.api)?;
    let stable_denom = match stable_info {
        AssetInfo::Token{..} => String::default(),
        AssetInfo::NativeToken{denom} => denom
    };
    if stable_denom == String::default() {
        return Err(StdError::generic_err("get_stable_denom failed: No native token found."));
    }

    Ok(stable_denom)
}

pub fn get_slippage_ratio(slippage: Decimal) -> StdResult<Decimal> {
    Ok(Decimal::from_ratio(Uint128::from(100u64) - Uint128::from(100u64) * slippage, Uint128::from(100u64)))
}

// In order to add to the pool liquidity, the added asset (luna) must first be sold for bluna at the pool ratio. 
pub fn try_provide_liquidity(
    deps: DepsMut,
    info: MessageInfo,
    asset: Asset
) -> StdResult<Response> {
    asset.assert_sent_native_token_balance(&info)?;

    let state = STATE.load(deps.storage)?;
    let deposit: Uint128 = asset.amount;
    let pool_info: PoolInfoRaw = POOL_INFO.load(deps.storage)?;
    let (pool_luna, pool_bluna) = compute_total_deposits(deps.as_ref(), &pool_info)?;
    let ratio = Decimal::from_ratio(pool_bluna, pool_luna - deposit);
    let luna_offer_amount = deposit * ratio;
    
    let offer_coin = match asset {
        Asset{info, amount} => Coin::new(amount.u128(), get_stable_denom(deps.as_ref())?),
        _ => return Err(StdError::generic_err("Unauthorized")),
    };
    let bluna_buy_amount = simulate_terraswap_swap(deps.as_ref(), deps.api.addr_humanize(&state.pool_address)?, offer_coin.clone())?;
    
    let total_deposits_in_luna = bluna_buy_amount;

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
    slippage: Decimal
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
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config{} => to_binary(&try_query_config(deps)?),
        QueryMsg::Pool{} => to_binary(&try_query_pool(deps)?),
        QueryMsg::Fees{} => to_binary(""),
        // TODO: Finish fee calculation and estimation 
        QueryMsg::EstimateDepositFee{ .. } => to_binary(""),
        QueryMsg::EstimateWithdrawFee{ .. } => to_binary(""),
    }
}

pub fn try_query_config(
    deps: Deps
) -> StdResult<PoolInfo> {

    let info = POOL_INFO.load(deps.storage)?;
    info.to_normal(deps)
}

pub fn try_query_pool(
    deps: Deps
) -> StdResult<PoolResponse> {
    let info = POOL_INFO.load(deps.storage)?;
    let contract_addr = deps.api.addr_humanize(&info.contract_addr)?;
    let assets: [Asset; 2] = info.query_pools(deps, contract_addr)?;
    let total_share: Uint128 =
        query_supply(&deps.querier, deps.api.addr_humanize(&info.liquidity_token)?)?;

    Ok(PoolResponse { assets, total_share })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{Api};

    fn get_test_init_msg() -> InitMsg {
        InitMsg {
            pool_address: "test_pool".to_string(),
            bluna_hub_address: "test_mm".to_string(),
            bluna_address: "test_aust".to_string(),
            slippage: Decimal::percent(1u64), token_code_id: 0u64
        }
    }


    #[test]
    fn test_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = get_test_init_msg();
        let env = mock_env();
        let info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(1, res.messages.len());
    }

    #[test]
    fn test_set_slippage() {
        let mut deps = mock_dependencies(&[]);

        let msg = get_test_init_msg();
        let env = mock_env();
        let msg_info = MessageInfo{sender: deps.api.addr_validate("creator").unwrap(), funds: vec![]};

        let res = instantiate(deps.as_mut(), env.clone(), msg_info.clone(), msg).unwrap();
        assert_eq!(1, res.messages.len());

        let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
        assert_eq!(info.slippage, Decimal::percent(1u64));

        let msg = HandleMsg::SetSlippage {
            slippage: Decimal::one()
        };
        let _res = execute(deps.as_mut(), env, msg_info, msg).unwrap();
        let info: PoolInfoRaw = POOL_INFO.load(&deps.storage).unwrap();
        assert_eq!(info.slippage, Decimal::one());
    }
}
