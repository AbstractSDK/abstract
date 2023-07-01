use crate::error::ContractError;
use crate::state::{Config, ROUTES};
use wyndex::asset::{Asset, AssetInfo};
use wyndex::pair::PairInfo;

use cosmwasm_std::{
    to_binary, Addr, Coin, Decimal, Deps, QuerierWrapper, StdResult, SubMsg, Uint128, WasmMsg,
};
use wyndex::pair::{Cw20HookMsg, SimulationResponse};

/// The default route depth for a fee token
pub const ROUTES_INITIAL_DEPTH: u64 = 0;
/// Maximum amount of route hops to use in a multi-hop swap
pub const ROUTES_MAX_DEPTH: u64 = 2;
/// Swap execution depth limit
pub const ROUTES_EXECUTION_MAX_DEPTH: u64 = 3;
/// This amount of tokens is used in get_pool swap simulations.
/// TODO: adjust according to token's precision?
pub const SWAP_SIMULATION_AMOUNT: Uint128 = Uint128::new(1_000_000u128);

/// The function checks from<>to pool exists and creates swap message.
///
/// * **from** asset we want to swap.
///
/// * **to** asset we want to swap to.
///
/// * **amount_in** amount of tokens to swap.
pub fn try_build_swap_msg(
    querier: &QuerierWrapper,
    cfg: &Config,
    from: &AssetInfo,
    to: &AssetInfo,
    amount_in: Uint128,
    belief_price: Option<Decimal>,
) -> Result<SubMsg, ContractError> {
    let (pool, _) = get_pool(
        querier,
        &cfg.dex_factory_contract,
        from,
        to,
        Some(amount_in),
    )?;
    let msg = build_swap_msg(
        cfg.max_spread,
        &pool,
        from,
        Some(to),
        amount_in,
        belief_price,
    )?;
    Ok(msg)
}

/// This function creates swap message.
///
/// * **max_spread** max allowed spread.
///
/// * **pool** pool's information.
///
/// * **from**  asset we want to swap.
///
/// * **to** asset we want to swap to.
///
/// * **amount_in** amount of tokens to swap.
pub fn build_swap_msg(
    max_spread: Decimal,
    pool: &PairInfo,
    from: &AssetInfo,
    to: Option<&AssetInfo>,
    amount_in: Uint128,
    belief_price: Option<Decimal>,
) -> Result<SubMsg, ContractError> {
    if from.is_native_token() {
        let offer_asset = Asset {
            info: from.clone(),
            amount: amount_in,
        };

        Ok(SubMsg::new(WasmMsg::Execute {
            contract_addr: pool.contract_addr.to_string(),
            msg: to_binary(&wyndex::pair::ExecuteMsg::Swap {
                offer_asset: offer_asset.clone(),
                ask_asset_info: to.cloned(),
                belief_price,
                max_spread: Some(max_spread),
                to: None,
                referral_address: None,
                referral_commission: None,
            })?,
            funds: vec![Coin {
                denom: offer_asset.info.to_string(),
                amount: offer_asset.amount,
            }],
        }))
    } else {
        Ok(SubMsg::new(WasmMsg::Execute {
            contract_addr: from.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                contract: pool.contract_addr.to_string(),
                amount: amount_in,
                msg: to_binary(&Cw20HookMsg::Swap {
                    ask_asset_info: to.cloned(),
                    belief_price,
                    max_spread: Some(max_spread),
                    to: None,
                    referral_address: None,
                    referral_commission: None,
                })?,
            })?,
            funds: vec![],
        }))
    }
}

/// This function checks that there is a direct pool to swap to the desired token.
/// Otherwise it looks for an intermediate token to swap to desired token.
///
/// * **from_token** asset we want to swap.
///
/// * **to_token** asset we want to swap to.
///
/// * **desired token** represents $WYND.
///
/// * **depth** current recursion depth of the validation.
///
/// * **amount** is an amount of from_token.
pub fn validate_route(
    deps: Deps,
    factory_contract: &Addr,
    from_token: &AssetInfo,
    route_token: &AssetInfo,
    desired_token: &AssetInfo,
    depth: u64,
    amount: Option<Uint128>,
) -> Result<PairInfo, ContractError> {
    // Check if the route pool exists
    let (route_pool, ret_amount) = get_pool(
        &deps.querier,
        factory_contract,
        from_token,
        route_token,
        amount,
    )?;
    // Check if the route token - desired-token pool exists
    let desired_token_pool = get_pool(
        &deps.querier,
        factory_contract,
        route_token,
        desired_token,
        ret_amount,
    );
    if desired_token_pool.is_err() {
        if depth >= ROUTES_MAX_DEPTH {
            return Err(ContractError::MaxRouteDepth(depth));
        }

        // Check if next level of route exists
        let next_route_token = ROUTES
            .load(deps.storage, route_token.to_string())
            .map_err(|_| ContractError::InvalidRouteDestination(from_token.to_string()))?;

        validate_route(
            deps,
            factory_contract,
            route_token,
            &next_route_token,
            desired_token,
            depth + 1,
            ret_amount,
        )?;
    }

    Ok(route_pool)
}

/// This function checks that there a pool to swap between `from` and `to`. In case of success
/// returns [`PairInfo`] of selected pool and simulated return amount.
///
/// * **from** source asset.
///
/// * **to** destination asset.
///
/// * **amount** optional. The value is used in swap simulations to select the best pool.
pub fn get_pool(
    querier: &QuerierWrapper,
    factory_contract: &Addr,
    from: &AssetInfo,
    to: &AssetInfo,
    amount: Option<Uint128>,
) -> Result<(PairInfo, Option<Uint128>), ContractError> {
    // We use raw query to save gas
    let result = wyndex::factory::ROUTE.query(
        querier,
        factory_contract.clone(),
        (from.to_string(), to.to_string()),
    )?;
    match result {
        Some(pairs) if !pairs.is_empty() => {
            // Select the best pool by performing a swap simulation for each pool
            // in pairs and then selecting the one with the best return amount
            // return that pool and the return amount
            let (best_pair, sim_res) = pairs
                .into_iter()
                .map(|pair_contract| {
                    // Perform a simulation swap to get the return amount for each pool
                    let sim_res: SimulationResponse = querier.query_wasm_smart(
                        &pair_contract,
                        &wyndex::pair::QueryMsg::Simulation {
                            offer_asset: Asset {
                                info: from.clone(),
                                amount: amount.unwrap_or(SWAP_SIMULATION_AMOUNT),
                            },
                            ask_asset_info: Some(to.clone()),
                            referral: false,
                            referral_commission: None,
                        },
                    )?;
                    Ok((pair_contract, sim_res))
                })
                .collect::<StdResult<Vec<_>>>()?
                .into_iter()
                .max_by(|(_, sim_res1), (_, sim_res2)| {
                    // Find the best rate by comparing the return amount
                    sim_res1.return_amount.cmp(&sim_res2.return_amount)
                })
                .unwrap();
            // Return the best pair's PairInfo and the return amount
            Ok((
                querier.query_wasm_smart(&best_pair, &wyndex::pair::QueryMsg::Pair {})?,
                Some(sim_res.return_amount),
            ))
        }
        _ => Err(ContractError::InvalidRouteNoPool(
            from.to_string(),
            to.to_string(),
        )),
    }
}
