use abstract_os::common_module::traits::ProxyExecute;
use abstract_os::memory::state::PAIR_POSTFIX;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Decimal, Deps, Env, Fraction, MessageInfo, Response, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::Asset;
use terraswap::pair::{Cw20HookMsg, PoolResponse};

use abstract_os::proxy::send_to_proxy;
use abstract_os::objects::proxy_assets::get_asset_identifier;
use abstract_os::modules::apis::terraswap::cw_to_terraswap;
use abstract_os::queries::terraswap::{query_asset_balance, query_pool};

use crate::contract::{TerraswapApi, TerraswapResult};
use crate::error::TerraswapError;
use crate::terraswap_msg::{asset_into_swap_msg, deposit_lp_msg};
use crate::utils::has_sufficient_balance;

/// Constructs and forwards the terraswap provide_liquidity message
pub fn provide_liquidity(
    deps: Deps,
    _msg_info: MessageInfo,
    api: TerraswapApi,
    main_asset_id: String,
    pool_id: String,
    amount: Uint128,
) -> TerraswapResult {
    let state = api.base_state.load(deps.storage)?;
    // Check if caller is trader.

    // Get pair address using memory single query
    let pair_address = state.memory.query_contract(deps, &pool_id)?;

    // Get pool info
    let pool_info: PoolResponse = query_pool(deps, &pair_address)?;
    let asset_1 = &pool_info.assets[0];
    let asset_2 = &pool_info.assets[1];

    let ratio = Decimal::from_ratio(asset_1.amount, asset_2.amount);

    let main_asset_info = state.memory.query_asset(deps, &main_asset_id)?;
    let main_asset = Asset {
        info: cw_to_terraswap(&main_asset_info),
        amount,
    };
    let mut first_asset: Asset;
    let mut second_asset: Asset;

    // Determine second asset and required amount to do a 50/50 LP
    if asset_2.info.equal(&main_asset.info) {
        first_asset = asset_1.clone();
        first_asset.amount = ratio * amount;
        second_asset = main_asset;
    } else {
        second_asset = asset_2.clone();
        second_asset.amount = ratio.inv().unwrap_or_default() * amount;
        first_asset = main_asset;
    }

    // // Does the proxy have enough of these assets?
    // let first_asset_balance = query_asset_balance(deps, &first_asset.info, proxy_address.clone())?;
    // let second_asset_balance =
    //     query_asset_balance(deps, &second_asset.info, proxy_address.clone())?;
    // if second_asset_balance < second_asset.amount || first_asset_balance < first_asset.amount {
    //     return Err(ApiError::Broke {}.into());
    // }

    // Deposit lp msg either returns a bank send msg or an
    // increase allowance msg for each asset.
    let msgs: Vec<CosmosMsg> =
        deposit_lp_msg(deps, [second_asset, first_asset], pair_address, None)?;

    Ok(api.execute_on_proxy(deps, msgs)?)
}

/// Constructs and forwards the terraswap provide_liquidity message
/// You can provide custom asset amounts
pub fn detailed_provide_liquidity(
    deps: Deps,
    _msg_info: MessageInfo,
    api: TerraswapApi,
    assets: Vec<(String, Uint128)>,
    pool_id: String,
    slippage_tolerance: Option<Decimal>,
) -> TerraswapResult {
    let state = api.base_state.load(deps.storage)?;
    // Check if caller is trader.

    if assets.len() != 2 {
        return Err(TerraswapError::NotTwoAssets {});
    }

    let proxy_address = &api.request_destination.clone().unwrap();

    // Get pair address
    let pair_address = state.memory.query_contract(deps, &pool_id)?;

    // Get pool info
    let pool_info: PoolResponse = query_pool(deps, &pair_address)?;

    // List with assets to send
    let mut assets_to_send: Vec<Asset> = vec![];

    // Iterate over provided assets
    for asset in assets {
        let asset_info = cw_to_terraswap(&state.memory.query_asset(deps, &asset.0)?);

        // Check if pool contains the asset
        if pool_info.assets.iter().any(|a| a.info == asset_info) {
            let asset_balance = query_asset_balance(deps, &asset_info, proxy_address.clone())?;
            // Check if proxy has enough of this asset
            if asset_balance < asset.1 {
                return Err(TerraswapError::Broke {});
            }
            // Append asset to list
            assets_to_send.push(Asset {
                info: asset_info,
                amount: asset.1,
            })
        } else {
            // Error if asset info not found in pool
            return Err(TerraswapError::NotInPool {
                id: asset_info.to_string(),
            }); //pool_info.assets[1].info
        }
    }
    let asset_array: [Asset; 2] = [assets_to_send[0].clone(), assets_to_send[1].clone()];
    // Deposit lp msg either returns a bank send msg or a
    // increase allowance msg for each asset.
    let msgs: Vec<CosmosMsg> = deposit_lp_msg(deps, asset_array, pair_address, slippage_tolerance)?;

    api.execute_on_proxy(deps, msgs)
        .map_err(TerraswapError::from)
}

/// Constructs withdraw liquidity msg and forwards it to proxy
pub fn withdraw_liquidity(
    deps: Deps,
    _msg_info: MessageInfo,
    api: TerraswapApi,
    lp_token_id: String,
    amount: Uint128,
) -> TerraswapResult {
    let state = api.base_state.load(deps.storage)?;
    // Sender must be trader

    let proxy_address = &api.request_destination.clone().unwrap();

    // Get lp token address
    let lp_token = &state.memory.query_asset(deps, &lp_token_id)?;
    let lp_token_address = get_asset_identifier(lp_token);
    // Get pair address
    let pair_address = state
        .memory
        .query_contract(deps, &(lp_token_id.clone() + PAIR_POSTFIX))?;

    // Check if the proxy has enough lp tokens
    has_sufficient_balance(deps, &state.memory, &lp_token_id, proxy_address, amount)?;

    // Msg that gets called on the pair address.
    let withdraw_msg: Binary = to_binary(&Cw20HookMsg::WithdrawLiquidity {})?;

    // cw20 send message that transfers the LP tokens to the pair address
    let cw20_msg = Cw20ExecuteMsg::Send {
        contract: pair_address.into_string(),
        amount,
        msg: withdraw_msg,
    };

    // Call on LP token.
    let lp_call = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lp_token_address,
        msg: to_binary(&cw20_msg)?,
        funds: vec![],
    });

    api.execute_on_proxy(deps, vec![lp_call])
        .map_err(TerraswapError::from)
}

/// Function constructs terraswap swap messages and forwards them to the proxy
#[allow(clippy::too_many_arguments)]
pub fn terraswap_swap(
    deps: Deps,
    _env: Env,
    _msg_info: MessageInfo,
    api: TerraswapApi,
    offer_id: String,
    pool_id: String,
    amount: Uint128,
    max_spread: Option<Decimal>,
    belief_price: Option<Decimal>,
) -> TerraswapResult {
    let state = api.base_state.load(deps.storage)?;
    let proxy_address = &api.request_destination.unwrap();

    // Check if caller is trader

    // Check if proxy has enough to swap
    // has_sufficient_balance(deps, &state.memory, &offer_id, proxy_address, amount)?;

    let pair_address = state.memory.query_contract(deps, &pool_id)?;

    let offer_asset_info = state.memory.query_asset(deps, &offer_id)?;

    let swap_msg = vec![asset_into_swap_msg(
        deps,
        pair_address,
        Asset {
            info: cw_to_terraswap(&offer_asset_info),
            amount,
        },
        max_spread,
        belief_price,
        // Msg is executed by proxy so None
        None,
    )?];

    Ok(Response::new().add_message(send_to_proxy(swap_msg, proxy_address)?))
}
