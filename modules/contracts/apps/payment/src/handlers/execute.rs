use std::collections::HashSet;

use abstract_core::objects::{AssetEntry, DexName};
use abstract_dex_adapter::DexInterface;
use abstract_sdk::core::ans_host;
use abstract_sdk::core::ans_host::{AssetPairingFilter, PoolAddressListResponse};
use cosmwasm_std::{Addr, Storage, Uint128};

use abstract_sdk::cw_helpers::AbstractAttributes;
use abstract_sdk::features::{AbstractNameService, AccountIdentification};
use abstract_sdk::AbstractResponse;
use cosmwasm_std::{CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response};
use cw_asset::{Asset, AssetList};

use crate::contract::{AppResult, PaymentApp};

const MAX_SPREAD_PERCENT: u64 = 20;

use crate::error::AppError;
use crate::msg::AppExecuteMsg;
use crate::state::{CONFIG, TIPPERS, TIPPER_COUNT, TIP_COUNT};

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: PaymentApp,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::UpdateConfig {
            desired_asset,
            denom_asset,
            exchanges,
        } => update_config(deps, info, app, desired_asset, denom_asset, exchanges),
        AppExecuteMsg::Tip {} => tip(deps, info, app, None),
    }
}

// Called when a payment is made to the app
pub fn tip(
    deps: DepsMut,
    info: MessageInfo,
    app: PaymentApp,
    cw20_receipt: Option<Asset>,
) -> Result<Response, crate::error::AppError> {
    let mut deposited_assets = AssetList::from(info.funds);
    // if a cw20 is received, add it to the assets list.
    if let Some(cw20_deposit) = &cw20_receipt {
        deposited_assets.add(cw20_deposit)?;
    }

    // forward payment to the proxy contract
    let forward_payment_msgs = deposited_assets.transfer_msgs(app.proxy_address(deps.as_ref())?)?;

    // resp
    let app_resp = app.tag_response(
        Response::new().add_messages(forward_payment_msgs),
        "receive_tip",
    );

    // swap the asset(s) to the desired asset is set
    let config = CONFIG.load(deps.storage)?;

    // Reverse query the deposited assets
    let ans = app.name_service(deps.as_ref());
    let asset_entries = ans.query(&deposited_assets.to_vec())?;

    // If there is no desired asset specified, just forward the payment.
    let Some(desired_asset) = config.desired_asset else {
        // Tipper history will not contain any info for "amount tipped" as it doesn't really make
        // sense when there isn't a desired asset. However the number of times tipped will be
        // stored.
        for asset in asset_entries {
            update_tipper_history(deps.storage, &info.sender, &asset.name, asset.amount)?;
        }
        return Ok(app_resp);
    };

    let mut swap_msgs: Vec<CosmosMsg> = Vec::new();
    let mut attrs: Vec<(&str, String)> = Vec::new();
    let exchange_strs: HashSet<&str> = config.exchanges.iter().map(AsRef::as_ref).collect();

    // For tip history
    let mut total_amount = Uint128::zero();

    // Search for trading pairs between the deposited assets and the desired asset
    for pay_asset in asset_entries {
        // No need to swap if desired asset sent
        if pay_asset.name == desired_asset {
            total_amount += pay_asset.amount;
            continue;
        }
        // query the pools that contain the desired asset
        let query_msg = ans_host::QueryMsg::PoolList {
            filter: Some(AssetPairingFilter {
                asset_pair: Some((desired_asset.clone(), pay_asset.name.clone())),
                dex: None,
            }),
            start_after: None,
            limit: None,
        };
        let resp: PoolAddressListResponse = deps
            .querier
            .query_wasm_smart(&ans.host.address, &query_msg)?;
        // use the first pair you find to swap on
        for (pair, refs) in resp.pools {
            if !refs.is_empty() && exchange_strs.contains(&pair.dex()) {
                let dex = app.dex(deps.as_ref(), pair.dex().to_owned());
                let trigger_swap_msg = dex.swap(
                    pay_asset.clone(),
                    desired_asset.clone(),
                    Some(Decimal::percent(MAX_SPREAD_PERCENT)),
                    None,
                )?;
                swap_msgs.push(trigger_swap_msg);
                attrs.push(("swap", format!("{} for {}", pay_asset.name, desired_asset)));

                total_amount += dex
                    .simulate_swap(pay_asset, desired_asset.clone())?
                    .return_amount;

                break;
            }
        }
    }

    // Tip history
    update_tipper_history(deps.storage, &info.sender, &desired_asset, total_amount)?;

    // forward deposit and execute swaps if there are any
    Ok(app_resp
        .add_messages(swap_msgs)
        .add_abstract_attributes(attrs))
}

fn update_tipper_history(
    storage: &mut dyn Storage,
    sender: &Addr,
    asset: &AssetEntry,
    amount: Uint128,
) -> Result<(), AppError> {
    // Update total counts
    let total_count = TIP_COUNT.load(storage)?;
    TIP_COUNT.save(storage, &(total_count + 1))?;
    // Update tipper counts
    let tipper_count = TIPPER_COUNT.may_load(storage, sender)?.unwrap_or_default();
    TIPPER_COUNT.save(storage, sender, &(tipper_count + 1))?;

    // Update tipper amount
    let mut total_tipper_amount = TIPPERS
        .may_load(storage, (sender, asset))?
        .unwrap_or_default();
    total_tipper_amount += amount;
    TIPPERS.save(storage, (sender, asset), &total_tipper_amount)?;

    Ok(())
}

/// Update the configuration of the app
fn update_config(
    deps: DepsMut,
    msg_info: MessageInfo,
    app: PaymentApp,
    desired_asset: Option<AssetEntry>,
    denom_asset: Option<String>,
    exchanges: Option<Vec<DexName>>,
) -> AppResult {
    // Only the admin should be able to call this
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let name_service = app.name_service(deps.as_ref());

    let mut config = CONFIG.load(deps.storage)?;
    if let Some(desired_asset) = desired_asset {
        name_service
            .query(&desired_asset)
            .map_err(|_| AppError::DesiredAssetDoesNotExist {})?;
        config.desired_asset = Some(desired_asset)
    }
    if let Some(exchanges) = exchanges {
        let ans_dexes = name_service.registered_dexes()?;
        for dex in exchanges.iter() {
            if !ans_dexes.dexes.contains(dex) {
                return Err(AppError::DexNotRegistered(dex.to_owned()));
            }
        }
        config.exchanges = exchanges;
    }
    if let Some(denom_asset) = denom_asset {
        config.denom_asset = denom_asset;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(app.tag_response(Response::default(), "update_config"))
}
