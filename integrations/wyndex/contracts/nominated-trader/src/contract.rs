use std::collections::{HashMap, HashSet};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;

use cw20::Cw20ExecuteMsg;
use wyndex::asset::{Asset, AssetInfo};

use crate::error::ContractError;
use crate::msg::{
    AssetWithLimit, BalancesResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
};
use crate::state::{Config, CONFIG, ROUTES};
use crate::utils::{
    build_swap_msg, try_build_swap_msg, validate_route, ROUTES_EXECUTION_MAX_DEPTH,
    ROUTES_INITIAL_DEPTH,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:nominated-trader";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender,
        nominated_trader: deps.api.addr_validate(&msg.nominated_trader)?,
        beneficiary: deps.api.addr_validate(&msg.beneficiary)?,
        token_contract: msg.token_contract.validate(deps.api)?,
        dex_factory_contract: deps.api.addr_validate(&msg.dex_factory_contract)?,
        max_spread: msg.max_spread.ok_or(Decimal::zero()).unwrap(),
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Collect { assets } => collect_fees_to_base_token(deps, env, info, assets),
        ExecuteMsg::UpdateRoutes { add, remove } => update_routes(deps, info, add, remove),
        ExecuteMsg::SwapHopAssets { assets, depth } => {
            swap_hop_assets(deps, info.sender, env.contract.address, assets, depth)
        }
        ExecuteMsg::Transfer { recipient, amount } => spend(deps, info, recipient, amount),
    }
}

/// This enum describes available token types that can be used as a SwapTarget.
/// Token indicates a SwapTarget with a direct part to the specified token_contract address.
/// RouteHop indicates a SwapTarget not directly to the token_contract address but via a route, while the routes itself will always lead to token_contract this RouteHop may only be showing one hop on the route.
enum SwapTarget {
    Token(SubMsg),
    RouteHop { asset: AssetInfo, msg: SubMsg },
}

/// Allows the `Owner` to send or 'spend' an amount of tokens to a recipient.
pub fn spend(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    // Permission check
    if info.sender != cfg.owner {
        return Err(ContractError::Unauthorized {
            info: "Only the owner can submit a spend".to_string(),
        });
    }

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cfg.token_contract.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.clone(),
                amount,
            })?,
        })])
        .add_attributes(vec![
            ("action", "spend"),
            ("recipient", recipient.as_str()),
            ("amount", &amount.to_string()),
        ]))
}

/// Perform a swap of fee assets to the desired base token specified in the config.
pub fn collect_fees_to_base_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: Vec<AssetWithLimit>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let wynd: AssetInfo = cfg.token_contract.clone().into();

    // Permission check
    if info.sender != cfg.nominated_trader {
        return Err(ContractError::Unauthorized {
            info: "Only the nominated trader can collect fees".to_string(),
        });
    }

    // Check for duplicate assets
    let mut uniq = HashSet::new();
    if !assets
        .clone()
        .into_iter()
        .all(|a| uniq.insert(a.info.to_string()))
    {
        return Err(ContractError::DuplicatedAsset {});
    }

    // Swap all non WYND tokens to Wynd
    let response = swap_assets(
        deps.as_ref(),
        &env.contract.address,
        &cfg,
        assets.into_iter().filter(|a| a.info != wynd).collect(),
        None,
    )?;
    Ok(response)
}

/// Checks if all required pools and routes exists and performs a swap operation to desired base token.
///
/// * **from_token** token to swap to desired base token.
///
/// * **amount_in** amount of tokens to swap.
fn swap(
    deps: Deps,
    cfg: &Config,
    from_token: AssetInfo,
    amount_in: Uint128,
    belief_price: Option<Decimal>,
) -> Result<SwapTarget, ContractError> {
    let desired_token = AssetInfo::Token(cfg.token_contract.to_string());
    // Check if route tokens exist
    let route_token = ROUTES.load(deps.storage, from_token.to_string());
    if let Ok(route_token) = route_token {
        let route_pool = validate_route(
            deps,
            &cfg.dex_factory_contract,
            &from_token,
            &route_token,
            &desired_token,
            ROUTES_INITIAL_DEPTH,
            Some(amount_in),
        )?;

        let msg = build_swap_msg(
            cfg.max_spread,
            &route_pool,
            &from_token,
            Some(&route_token),
            amount_in,
            belief_price,
        )?;
        return Ok(SwapTarget::RouteHop {
            asset: route_token,
            msg,
        });
    }
    // Check for a direct pair with the token_contract
    let swap_to_desired_token = try_build_swap_msg(
        &deps.querier,
        cfg,
        &from_token,
        &desired_token,
        amount_in,
        belief_price,
    );
    if let Ok(msg) = swap_to_desired_token {
        return Ok(SwapTarget::Token(msg));
    }

    Err(ContractError::CannotSwap(from_token))
}

fn swap_assets(
    deps: Deps,
    contract_addr: &Addr,
    cfg: &Config,
    assets: Vec<AssetWithLimit>,
    belief_price: Option<Decimal>,
) -> Result<Response, ContractError> {
    let mut response = Response::default();
    let mut route_assets = HashMap::new();
    for asset in assets {
        // Get balance
        let mut balance = asset.info.query_pool(&deps.querier, contract_addr)?;
        if let Some(limit) = asset.limit {
            if limit < balance && limit > Uint128::zero() {
                balance = limit;
            }
        }

        if !balance.is_zero() {
            let swap_msg = swap(deps, cfg, asset.info, balance, belief_price)?;
            match swap_msg {
                SwapTarget::Token(msg) => {
                    response.messages.push(msg);
                }
                SwapTarget::RouteHop { asset, msg } => {
                    response.messages.push(msg);
                    route_assets.insert(asset.to_string(), asset);
                }
            }
        }
    }

    // There should always be some messages, if there are none - something went wrong
    if response.messages.is_empty() {
        return Err(ContractError::SwapError {});
    }
    // If we have route assets, call SwapHopAssets with them
    if !route_assets.is_empty() {
        response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&ExecuteMsg::SwapHopAssets {
                assets: route_assets.into_values().collect(),
                depth: 2,
            })?,
            funds: vec![],
        }));
    }

    Ok(response)
}

/// Swaps collected fees using route assets.
/// After a collection is done, any specified assets which do
/// not have a direct pair to the desired token will first
/// be swapped to one of the established route tokens
/// subsequent swaps to get from the route token to the desired token
/// will be done in SubMsgs using this entrypoint.
///
/// * **assets** array with fee tokens to swap as well as amount of tokens to swap.
///
/// * **depth** maximum route length used to swap a fee token.
///
/// ## Executor
/// Only the contract itself can execute this.
fn swap_hop_assets(
    deps: DepsMut,
    sender: Addr,
    contract_address: Addr,
    assets: Vec<AssetInfo>,
    depth: u64,
) -> Result<Response, ContractError> {
    if sender != contract_address {
        return Err(ContractError::Unauthorized {
            info: "User is not authorised to swap route assets".to_string(),
        });
    }

    if assets.is_empty() {
        return Ok(Response::default());
    }

    // Check that the contract doesn't call itself endlessly
    if depth >= ROUTES_EXECUTION_MAX_DEPTH {
        return Err(ContractError::MaxRouteDepth(depth));
    }

    let cfg = CONFIG.load(deps.storage)?;

    let routes = assets
        .into_iter()
        .map(|a| AssetWithLimit {
            info: a,
            limit: None,
        })
        .collect();

    let response = swap_assets(deps.as_ref(), &contract_address, &cfg, routes, None)?;

    Ok(response.add_attribute("action", "swap_route_assets"))
}

/// Adds or removes defined routes used to swap fee tokens to WYND.
///
/// * **add** array of routes defining hops needed to swap fee tokens to Wynd.
///
/// * **remove** array of routes defining hops needed to swap fee tokens to Wynd.
///
/// ## Executor
/// Only the owner can execute this.
fn update_routes(
    deps: DepsMut,
    info: MessageInfo,
    add: Option<Vec<(AssetInfo, AssetInfo)>>,
    remove: Option<Vec<AssetInfo>>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // Permission check
    if info.sender != cfg.owner {
        return Err(ContractError::Unauthorized {
            info: "Only the owner can update routes".to_string(),
        });
    }

    // Remove old routes
    if let Some(remove_routes) = remove {
        for asset in remove_routes {
            ROUTES.remove(
                deps.storage,
                deps.api.addr_validate(&asset.to_string())?.to_string(),
            );
        }
    }
    let wynd = AssetInfo::Token(cfg.token_contract.to_string());
    // Add new routes
    if let Some(routes_to_add) = add {
        for (asset, route) in routes_to_add {
            // Verify asset is not same as route
            // Check that route tokens can be swapped to WYND
            validate_route(
                deps.as_ref(),
                &cfg.dex_factory_contract,
                &asset,
                &route,
                &wynd,
                ROUTES_INITIAL_DEPTH,
                None,
            )?;
            ROUTES.save(deps.storage, asset.to_string(), &route)?;
        }
    }

    Ok(Response::default().add_attribute("action", "update_routes"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_get_config(deps)?),
        QueryMsg::Balances { assets } => to_binary(&query_get_balances(deps, env, assets)?),
        QueryMsg::Routes {} => to_binary(&query_routes(deps)?),
    }
}

/// Returns information about the Nominated Trader configuration using a [`ConfigResponse`] object.
fn query_get_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        token_contract: config.token_contract.to_string(),
        dex_factory_contract: config.dex_factory_contract.to_string(),
        max_spread: config.max_spread,
        nominated_trader: config.nominated_trader.to_string(),
        beneficiary: config.beneficiary.to_string(),
    })
}

/// Returns Traders's fee token balances for specific tokens using a [`BalancesResponse`] object.
///
/// * **assets** array with assets for which we query the Nominated Trader's balances.
fn query_get_balances(deps: Deps, env: Env, assets: Vec<AssetInfo>) -> StdResult<BalancesResponse> {
    let mut resp = BalancesResponse { balances: vec![] };

    for asset in assets {
        // Get balance
        let balance = asset.query_pool(&deps.querier, &env.contract.address)?;
        if !balance.is_zero() {
            resp.balances.push(Asset {
                info: asset,
                amount: balance,
            })
        }
    }

    Ok(resp)
}

/// Returns route tokens used for swapping fee tokens to WYND.
fn query_routes(deps: Deps) -> StdResult<Vec<(String, String)>> {
    ROUTES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|route| {
            let (route, asset) = route?;
            Ok((route, asset.to_string()))
        })
        .collect()
}
