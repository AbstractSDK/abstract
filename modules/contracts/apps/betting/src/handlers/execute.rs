use abstract_sdk::{
    *,
    core::objects::fee::Fee, features::AbstractResponse,
};
use cosmwasm_std::{
    DepsMut, Env, MessageInfo, Response,
};
use cw_storage_plus::Item;

use crate::contract::{EtfApp, EtfResult};
use crate::msg::BetExecuteMsg;
use crate::state::*;
use crate::state::COTFIG_2;

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: EtfApp,
    msg: BetExecuteMsg,
) -> EtfResult {
    match msg {
        BetExecuteMsg::CreateTrack(track) => {
            app.admin.assert_admin(deps.as_ref(), &info.sender)?;
            // Check asset
            create_track(deps, info, app, track)
        }
        BetExecuteMsg::UpdateConfig { rake } => {
            app.admin.assert_admin(deps.as_ref(), &info.sender)?;
            // TODO: use config constant, not sure why this is not working.
            let mut config: Config = Item::new("config").load(deps.storage)?;
            let mut attrs = vec![];

            if let Some(rake) = rake {
                config.rake = Fee::new(rake)?;
                attrs.push(("rake", rake.to_string()));
            };

            Ok(app.custom_tag_response(
                Response::default(),
                "update_config",
                attrs,
            ))
        },
        _ => panic!("Unsupported execute message"),
    }
}

pub fn create_track(
    deps: DepsMut,
    msg_info: MessageInfo,
    app: EtfApp,
    track: Track
) -> EtfResult {
    let mut state = STATE.load(deps.storage)?;

    // Check track
    track.validate()?;

    TRACKS.save(deps.storage, state.next_track_id, &track)?;

    // Update and save the state
    STATE.update(deps.storage, |mut state | -> EtfResult<_> {
        state.next_track_id += 1;
        Ok(state)
    })?;
    Ok(app.custom_tag_response(Response::default(), "create_track", vec![("track_id", state.next_track_id.to_string())]))
}

// /// Called when either providing liquidity with a native token or when providing liquidity
// /// with a CW20.
// pub fn try_provide_liquidity(
//     deps: DepsMut,
//     msg_info: MessageInfo,
//     app: EtfApp,
//     asset: Asset,
//     // optional sender address
//     // set if called from CW20 hook
//     sender: Option<String>,
// ) -> EtfResult {
//     let state = STATE.load(deps.storage)?;
//     // Get the depositor address
//     let depositor = match sender {
//         Some(addr) => deps.api.addr_validate(&addr)?,
//         None => {
//             // Check if deposit matches claimed deposit.
//             match asset.info {
//                 AssetInfo::Native(..) => {
//                     // If native token, assert claimed amount is correct
//                     let coin = msg_info.funds.last();
//                     if coin.is_none() {
//                         return Err(BetError::WrongNative {});
//                     }
//
//                     let coin = coin.unwrap().clone();
//                     if Asset::native(coin.denom, coin.amount) != asset {
//                         return Err(BetError::WrongNative {});
//                     }
//                     msg_info.sender
//                 }
//                 AssetInfo::Cw20(_) => return Err(BetError::NotUsingCW20Hook {}),
//                 _ => return Err(BetError::UnsupportedAssetType(asset.info.to_string())),
//             }
//         }
//     };
//     // Get vault API for the account
//     let vault = app.accountant(deps.as_ref());
//     // Construct deposit info
//     let deposit_info = DepositInfo {
//         asset_info: vault.base_asset()?.base_asset,
//     };
//
//     // Assert deposited info and claimed asset info are the same
//     deposit_info.assert(&asset.info)?;
//
//     // Init vector for logging
//     let attrs = vec![
//         ("action", String::from("deposit_to_vault")),
//         ("Received funds:", asset.to_string()),
//     ];
//
//     // Received deposit to vault
//     let deposit: Uint128 = asset.amount;
//
//     // Get total value in Vault
//     let account_value = vault.query_total_value()?;
//     let total_value = account_value.total_value.amount;
//     // Get total supply of LP tokens and calculate share
//     let total_share = query_supply(&deps.querier, state.share_token_address.clone())?;
//
//     let share = if total_share == Uint128::zero() || total_value.is_zero() {
//         // Initial share = deposit amount
//         deposit
//     } else {
//         // lt: liquidity token
//         // lt_to_receive = deposit * lt_price
//         // lt_to_receive = deposit * lt_supply / previous_total_vault_value )
//         // lt_to_receive = deposit * ( lt_supply / ( current_total_vault_value - deposit ) )
//         let value_increase = Decimal::from_ratio(total_value + deposit, total_value);
//         (total_share * value_increase) - total_share
//     };
//
//     // mint LP token to depositor
//     let mint_lp = CosmosMsg::Wasm(WasmMsg::Execute {
//         contract_addr: state.share_token_address.to_string(),
//         msg: to_binary(&Cw20ExecuteMsg::Mint {
//             recipient: depositor.to_string(),
//             amount: share,
//         })?,
//         funds: vec![],
//     });
//
//     // Send received asset to the vault.
//     let send_to_vault = app.bank(deps.as_ref()).deposit(vec![asset])?;
//
//     let response = app
//         .custom_tag_response(Response::default(), "provide_liquidity", attrs)
//         .add_message(mint_lp)
//         .add_messages(send_to_vault);
//
//     Ok(response)
// }
//
// /// Attempt to withdraw deposits. Fees are calculated and deducted in liquidity tokens.
// /// This allows the owner to accumulate a stake in the vault.
// pub fn try_withdraw_liquidity(
//     deps: DepsMut,
//     _env: Env,
//     app: EtfApp,
//     sender: Addr,
//     amount: Uint128,
// ) -> EtfResult {
//     let state: State = STATE.load(deps.storage)?;
//     let base_state: AppState = app.load_state(deps.storage)?;
//     let fee: Fee = RAKE.load(deps.storage)?;
//     let bank = app.bank(deps.as_ref());
//     // Get assets
//     let assets: AssetsInfoResponse = app.accountant(deps.as_ref()).assets_list()?;
//
//     // Logging var
//     let mut attrs = vec![("liquidity_tokens", amount.to_string())];
//
//     // Calculate share of pool and requested pool value
//     let total_share: Uint128 = query_supply(&deps.querier, state.share_token_address.clone())?;
//
//     // Get manager fee in LP tokens
//     let manager_fee = fee.compute(amount);
//
//     // Share with fee deducted.
//     let share_ratio: Decimal = Decimal::from_ratio(amount - manager_fee, total_share);
//
//     let mut msgs: Vec<CosmosMsg> = vec![];
//     if !manager_fee.is_zero() {
//         // LP token fee
//         let lp_token_manager_fee = Asset {
//             info: AssetInfo::Cw20(state.share_token_address.clone()),
//             amount: manager_fee,
//         };
//         // Construct manager fee msg
//         let manager_fee_msg = fee.msg(lp_token_manager_fee, state.manager_addr.clone())?;
//
//         // Transfer fee
//         msgs.push(manager_fee_msg);
//     }
//     attrs.push(("treasury_fee", manager_fee.to_string()));
//
//     // Get asset holdings of vault and calculate amount to return
//     let mut shares_assets: Vec<Asset> = vec![];
//     for (info, _) in assets.assets.into_iter() {
//         // query asset held in proxy
//         let asset_balance = info.query_balance(&deps.querier, base_state.proxy_address.clone())?;
//         shares_assets.push(Asset {
//             info: info.clone(),
//             amount: share_ratio * asset_balance,
//         });
//     }
//
//     // Construct repay msg by transferring the assets back to the sender
//     let refund_msg = app
//         .executor(deps.as_ref())
//         .execute(vec![bank.transfer(shares_assets, &sender)?])?;
//
//     // LP burn msg
//     let burn_msg: CosmosMsg = wasm_execute(
//         state.share_token_address,
//         // Burn excludes fee
//         &Cw20ExecuteMsg::Burn {
//             amount: (amount - manager_fee),
//         },
//         vec![],
//     )?
//     .into();
//
//     Ok(app
//         .custom_tag_response(Response::default(), "withdraw_liquidity", attrs)
//         // Burn LP tokens
//         .add_message(burn_msg)
//         // Send proxy funds to owner
//         .add_message(refund_msg))
// }


// /// helper for CW20 supply query
// fn query_supply(querier: &QuerierWrapper, contract_addr: Addr) -> StdResult<Uint128> {
//     let res: TokenInfoResponse = querier.query(&wasm_smart_query(
//         String::from(contract_addr),
//         &Cw20QueryMsg::TokenInfo {},
//     )?)?;
//     Ok(res.total_supply)
// }
