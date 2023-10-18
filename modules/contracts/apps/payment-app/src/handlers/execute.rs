use abstract_dex_adapter::DexInterface;
use abstract_sdk::core::ans_host;
use abstract_sdk::core::ans_host::{AssetPairingFilter, PoolAddressListResponse};
use cosmwasm_std::Uint128;

use abstract_sdk::cw_helpers::AbstractAttributes;
use abstract_sdk::features::{AbstractNameService, AccountIdentification};
use abstract_sdk::{cw_helpers, AbstractResponse};
use cosmwasm_std::{CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response};
use cw_asset::{Asset, AssetList};

use crate::contract::{AppResult, PaymentApp};

use crate::msg::AppExecuteMsg;
use crate::state::{Tipper, CONFIG, TIPPERS, TIP_COUNT};

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: PaymentApp,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::UpdateConfig { exchanges: _ } => update_config(deps, info, app),
        AppExecuteMsg::Tip {} => tip(deps, info, app, None),
    }
}

// Called when a payment is made to the app
pub fn tip(
    deps: DepsMut<'_>,
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
    let Some(desired_asset) = config.desired_asset else {
      // just forward the payment
      return Ok(app_resp);
    };

    // Reverse query the deposited assets
    let ans = app.name_service(deps.as_ref());
    let asset_entries = ans.query(&deposited_assets.to_vec())?;

    let mut swap_msgs: Vec<CosmosMsg> = Vec::new();
    let mut attrs: Vec<(&str, String)> = Vec::new();
    let exchange_strs: Vec<&str> = config.exchanges.iter().map(AsRef::as_ref).collect();

    // For tip history
    let mut total_amount = Uint128::zero();

    // Search for trading pairs between the deposited assets and the desired asset
    for pay_asset in asset_entries {
        // query the pools that contain the desired asset
        let query = cw_helpers::wasm_smart_query(
            &ans.host.address,
            &ans_host::QueryMsg::PoolList {
                filter: Some(AssetPairingFilter {
                    asset_pair: Some((desired_asset.clone(), pay_asset.name.clone())),
                    dex: None,
                }),
                start_after: None,
                limit: None,
            },
        )?;
        let resp: PoolAddressListResponse = deps.querier.query(&query)?;
        // use the first pair you find to swap on
        for (pair, refs) in resp.pools {
            if !refs.is_empty() && exchange_strs.contains(&pair.dex()) {
                let dex = app.dex(deps.as_ref(), pair.dex().to_string());
                let trigger_swap_msg = dex.swap(
                    pay_asset.clone(),
                    desired_asset.clone(),
                    Some(Decimal::percent(20)),
                    None,
                )?;
                swap_msgs.push(trigger_swap_msg);
                attrs.push((
                    "swap",
                    format!("{} for {}", pay_asset.name, desired_asset.clone()),
                ));

                total_amount += dex
                    .simulate_swap(pay_asset, desired_asset.clone())?
                    .return_amount;

                break;
            }
        }
    }

    // Tip history
    let tip_count = TIP_COUNT.load(deps.storage).unwrap_or(0);
    TIP_COUNT.save(deps.storage, &(tip_count + 1))?;

    TIPPERS.update(
        deps.storage,
        info.sender,
        |e| -> Result<Tipper, crate::error::AppError> {
            match e {
                Some(e) => Ok(Tipper {
                    amount: total_amount + e.amount,
                    count: e.count + 1,
                }),
                None => Ok(Tipper {
                    amount: total_amount,
                    count: 1,
                }),
            }
        },
    )?;

    // forward deposit and execute swaps if there are any
    Ok(app_resp
        .add_messages(swap_msgs)
        .add_abstract_attributes(attrs))
}

/// Update the configuration of the app
fn update_config(deps: DepsMut, msg_info: MessageInfo, app: PaymentApp) -> AppResult {
    // Only the admin should be able to call this
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.tag_response(Response::default(), "update_config"))
}
