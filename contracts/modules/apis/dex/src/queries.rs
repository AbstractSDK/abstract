use abstract_os::{
    dex::{OfferAsset, SimulateSwapResponse},
    objects::AssetEntry,
};
use abstract_sdk::{MemoryOperation, Resolve};
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, StdResult};
use cw_asset::Asset;

use crate::{contract::resolve_exchange, contract::DEX_API};

pub fn simulate_swap(
    deps: Deps,
    _env: Env,
    offer_asset: OfferAsset,
    mut ask_asset: AssetEntry,
    dex: String,
) -> StdResult<Binary> {
    let exchange = resolve_exchange(&dex).map_err(|e| StdError::generic_err(e.to_string()))?;
    let api = DEX_API;
    let mem = api.load_memory(deps.storage)?;
    // format input
    let (mut offer_asset, offer_amount) = offer_asset;
    offer_asset.format();
    ask_asset.format();
    // get addresses
    let offer_asset_info = offer_asset.resolve(deps, &mem)?;
    let ask_asset_info = ask_asset.resolve(deps, &mem)?;
    let pair_address = exchange.pair_address(deps, &mem, &mut vec![&offer_asset, &ask_asset])?;
    let pool_info = exchange.pair_contract(&mut vec![&offer_asset, &ask_asset]);
    // create offer asset
    let swap_offer_asset: Asset = Asset::new(offer_asset_info, offer_amount);
    let (return_amount, spread_amount, commission_amount, fee_on_input) = exchange
        .simulate_swap(deps, pair_address, swap_offer_asset, ask_asset_info)
        .map_err(|e| StdError::generic_err(e.to_string()))?;
    let commission_asset = if fee_on_input { ask_asset } else { offer_asset };
    let resp = SimulateSwapResponse {
        pool: pool_info,
        return_amount,
        spread_amount,
        commission: (commission_asset, commission_amount),
    };
    to_binary(&resp).map_err(From::from)
}
