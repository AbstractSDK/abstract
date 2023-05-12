#![allow(unused)]
/// is unused if no features are enabled (which won't be the case)
use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, StdResult, WasmMsg};
use cw20::Cw20ExecuteMsg;
use cw_asset::{Asset, AssetInfo};

pub(crate) fn cw_approve_msgs(assets: &[Asset], spender: &Addr) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];
    for asset in assets {
        if let AssetInfo::Cw20(addr) = &asset.info {
            let msg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: spender.to_string(),
                amount: asset.amount,
                expires: None,
            };
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            }))
        }
    }
    Ok(msgs)
}

pub(crate) fn coins_in_assets(assets: &[Asset]) -> Vec<Coin> {
    let mut coins = vec![];
    for asset in assets {
        if let AssetInfo::Native(denom) = &asset.info {
            coins.push(Coin::new(asset.amount.u128(), denom.clone()));
        }
    }
    coins
}
