use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, StdResult};

use abstract_os::dex::{DexQueryMsg, OfferAsset, SimulateSwapResponse};
use abstract_os::objects::AssetEntry;
use abstract_sdk::base::features::AbstractNameService;

use crate::contract::{DexExtension, DEX_EXTENSION};
use crate::exchanges::exchange_resolver::resolve_exchange;

pub fn query_handler(
    deps: Deps,
    env: Env,
    _app: &DexExtension,
    msg: DexQueryMsg,
) -> StdResult<Binary> {
    match msg {
        DexQueryMsg::SimulateSwap {
            offer_asset,
            ask_asset,
            dex,
        } => simulate_swap(deps, env, offer_asset, ask_asset, dex.unwrap()).map_err(Into::into),
    }
}

pub fn simulate_swap(
    deps: Deps,
    _env: Env,
    mut offer_asset: OfferAsset,
    mut ask_asset: AssetEntry,
    dex: String,
) -> StdResult<Binary> {
    let exchange = resolve_exchange(&dex).map_err(|e| StdError::generic_err(e.to_string()))?;
    let extension = DEX_EXTENSION;
    let ans = extension.name_service(deps);
    // format input
    offer_asset.name.format();
    ask_asset.format();
    // get addresses
    let swap_offer_asset = ans.query(&offer_asset)?;
    let ask_asset_info = ans.query(&ask_asset)?;
    let pair_address =
        exchange.pair_address(deps, ans.host(), &mut vec![&offer_asset.name, &ask_asset])?;
    let pool_info = exchange.pair_contract(&mut vec![&offer_asset.name, &ask_asset]);

    let (return_amount, spread_amount, commission_amount, fee_on_input) = exchange
        .simulate_swap(deps, pair_address, swap_offer_asset, ask_asset_info)
        .map_err(|e| StdError::generic_err(e.to_string()))?;
    let commission_asset = if fee_on_input {
        ask_asset
    } else {
        offer_asset.name
    };
    let resp = SimulateSwapResponse {
        pool: pool_info,
        return_amount,
        spread_amount,
        commission: (commission_asset, commission_amount),
    };
    to_binary(&resp).map_err(From::from)
}
