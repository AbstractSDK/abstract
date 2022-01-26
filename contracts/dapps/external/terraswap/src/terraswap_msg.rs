use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, Decimal, Deps, Empty, StdResult, WasmMsg};

use cw20::Cw20ExecuteMsg;
use pandora::tax::compute_tax;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as PairExecuteMsg;

/// Constructs the deposit msg
pub fn deposit_lp_msg(
    deps: Deps,
    mut assets: [Asset; 2],
    pair_addr: Addr,
    slippage_tolerance: Option<Decimal>,
) -> StdResult<Vec<CosmosMsg<Empty>>> {
    let mut msgs: Vec<CosmosMsg<Empty>> = vec![];
    let mut coins: Vec<Coin> = vec![];
    for asset in assets.iter_mut() {
        match &asset.info {
            AssetInfo::Token { contract_addr } => {
                msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                        spender: pair_addr.to_string(),
                        amount: asset.amount,
                        expires: None,
                    })?,
                    funds: vec![],
                }));
            }
            AssetInfo::NativeToken { .. } => {
                coins.push(asset.deduct_tax(&deps.querier)?);
                asset.amount = asset.deduct_tax(&deps.querier)?.amount;
            }
        }
    }

    let lp_msg = PairExecuteMsg::ProvideLiquidity {
        assets,
        slippage_tolerance,
        receiver: None,
    };

    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: pair_addr.to_string(),
        msg: to_binary(&lp_msg)?,
        funds: coins,
    }));

    Ok(msgs)
}

// Adapted from terraswap_router operations.rs
/// Constructs a swap msg
pub fn asset_into_swap_msg(
    deps: Deps,
    pair_contract: Addr,
    offer_asset: Asset,
    max_spread: Option<Decimal>,
    belief_price: Option<Decimal>,
    to: Option<String>,
) -> StdResult<CosmosMsg<Empty>> {
    match offer_asset.info.clone() {
        AssetInfo::NativeToken { denom } => {
            // deduct tax first
            let amount = offer_asset.amount.checked_sub(compute_tax(
                deps,
                &Coin::new(offer_asset.amount.u128(), denom.clone()),
            )?)?;

            Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: pair_contract.to_string(),
                funds: vec![Coin { denom, amount }],
                msg: to_binary(&PairExecuteMsg::Swap {
                    offer_asset: Asset {
                        amount,
                        ..offer_asset
                    },
                    belief_price,
                    max_spread,
                    to,
                })?,
            }))
        }
        AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: pair_contract.to_string(),
                amount: offer_asset.amount,
                msg: to_binary(&PairExecuteMsg::Swap {
                    offer_asset,
                    belief_price,
                    max_spread,
                    to,
                })?,
            })?,
        })),
    }
}
