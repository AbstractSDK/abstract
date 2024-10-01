use std::collections::HashSet;

use abstract_app::sdk::{
    cw_helpers::{AbstractAttributes, Clearable},
    features::AbstractNameService,
    AbstractResponse, TransferInterface,
};
use abstract_app::std::{
    ans_host::AssetPairingFilter,
    ans_host::AssetPairingMapEntry,
    objects::{AnsAsset, AssetEntry, DexName},
};
use abstract_dex_adapter::DexInterface;
use cosmwasm_std::{
    Addr, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Storage, Uint128,
};
use cw_asset::{Asset, AssetList};

use crate::contract::{AppResult, PaymentApp};

const MAX_SPREAD_PERCENT: u64 = 20;

use crate::{
    error::AppError,
    msg::AppExecuteMsg,
    state::{CONFIG, TIPPERS, TIPPER_COUNT, TIP_COUNT},
};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: PaymentApp,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::UpdateConfig {
            desired_asset,
            denom_asset,
            exchanges,
        } => update_config(
            deps,
            env,
            info,
            module,
            desired_asset,
            denom_asset,
            exchanges,
        ),
        AppExecuteMsg::Tip {} => tip(deps, env, info, module, None),
    }
}

// Called when a payment is made to the app
pub fn tip(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: PaymentApp,
    cw20_receipt: Option<Asset>,
) -> Result<Response, crate::error::AppError> {
    let mut deposited_assets = AssetList::from(info.funds);
    // if a cw20 is received, add it to the assets list.
    if let Some(cw20_deposit) = &cw20_receipt {
        deposited_assets.add(cw20_deposit)?;
    }

    // forward payment to the proxy contract
    let forward_payment_msgs = module
        .bank(deps.as_ref())
        .deposit(deposited_assets.to_vec())?;

    // resp
    let app_resp = module
        .response("receive_tip")
        .add_messages(forward_payment_msgs);

    // swap the asset(s) to the desired asset is set
    let config = CONFIG.load(deps.storage)?;

    // Reverse query the deposited assets
    let ans = module.name_service(deps.as_ref());
    let asset_entries = ans.query(&deposited_assets.to_vec())?;

    // If there is no desired asset specified, just forward the payment.
    let Some(desired_asset) = config.desired_asset else {
        // Add assets as is
        update_tipper_history(deps.storage, &info.sender, &asset_entries, env.block.height)?;
        return Ok(app_resp);
    };

    let mut swap_msgs: Vec<CosmosMsg> = Vec::new();
    let mut attrs: Vec<(&str, String)> = Vec::new();
    let exchange_strs: HashSet<&str> = config.exchanges.iter().map(AsRef::as_ref).collect();

    // For tip history
    let mut desired_asset_amount = Uint128::zero();
    // For updating tipper history
    let mut assets_to_add = vec![];

    // Search for trading pairs between the deposited assets and the desired asset
    for pay_asset in asset_entries {
        // No need to swap if desired asset sent
        if pay_asset.name == desired_asset {
            desired_asset_amount += pay_asset.amount;
            continue;
        }
        // query the pools that contain the desired asset
        let resp: Vec<AssetPairingMapEntry> = ans.pool_list(
            Some(AssetPairingFilter {
                asset_pair: Some((desired_asset.clone(), pay_asset.name.clone())),
                dex: None,
            }),
            None,
            None,
        )?;

        // use the first pair you find to swap on
        if let Some((pair, _refs)) = resp
            .into_iter()
            .find(|(pair, refs)| !refs.is_empty() && exchange_strs.contains(&pair.dex()))
        {
            let dex = module.ans_dex(deps.as_ref(), pair.dex().to_owned());
            let trigger_swap_msg = dex.swap(
                pay_asset.clone(),
                desired_asset.clone(),
                Some(Decimal::percent(MAX_SPREAD_PERCENT)),
                None,
            )?;
            swap_msgs.push(trigger_swap_msg);
            attrs.push(("swap", format!("{} for {}", pay_asset.name, desired_asset)));

            desired_asset_amount += dex
                .simulate_swap(pay_asset.clone(), desired_asset.clone())?
                .return_amount;
        } else {
            // If swap not found just accept payment
            assets_to_add.push(pay_asset);
        }
    }

    // Add desired asset after swaps
    if !desired_asset_amount.is_zero() {
        assets_to_add.push(AnsAsset {
            name: desired_asset,
            amount: desired_asset_amount,
        })
    }

    // Tip history
    update_tipper_history(deps.storage, &info.sender, &assets_to_add, env.block.height)?;

    // forward deposit and execute swaps if there are any
    Ok(app_resp
        .add_messages(swap_msgs)
        .add_abstract_attributes(attrs))
}

fn update_tipper_history(
    storage: &mut dyn Storage,
    sender: &Addr,
    assets: &[AnsAsset],
    height: u64,
) -> Result<(), AppError> {
    // Update total counts
    let total_count = TIP_COUNT.load(storage)?;
    TIP_COUNT.save(storage, &(total_count + 1))?;
    // Update tipper counts
    let tipper_count = TIPPER_COUNT.may_load(storage, sender)?.unwrap_or_default();
    TIPPER_COUNT.save(storage, sender, &(tipper_count + 1), height)?;

    // Update tipper amount
    for asset in assets {
        let mut total_tipper_amount = TIPPERS
            .may_load(storage, (sender, &asset.name))?
            .unwrap_or_default();
        total_tipper_amount += &asset.amount;
        TIPPERS.save(storage, (sender, &asset.name), &total_tipper_amount, height)?;
    }

    Ok(())
}

/// Update the configuration of the app
fn update_config(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    module: PaymentApp,
    desired_asset: Option<Clearable<AssetEntry>>,
    denom_asset: Option<String>,
    exchanges: Option<Vec<DexName>>,
) -> AppResult {
    // Only the admin should be able to call this
    module
        .admin
        .assert_admin(deps.as_ref(), &env, &msg_info.sender)?;
    let name_service = module.name_service(deps.as_ref());

    let mut config = CONFIG.load(deps.storage)?;
    if let Some(desired_asset) = desired_asset {
        if let Clearable::Set(desired_asset) = &desired_asset {
            name_service
                .query(desired_asset)
                .map_err(|_| AppError::DesiredAssetDoesNotExist {})?;
        }
        config.desired_asset = desired_asset.into()
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

    Ok(module.response("update_config"))
}
