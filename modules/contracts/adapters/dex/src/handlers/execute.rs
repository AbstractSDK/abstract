use abstract_adapter::sdk::{
    features::AbstractNameService, AccountVerification, Execution, IbcInterface,
    ModuleRegistryInterface,
};
use abstract_adapter::std::{
    ibc::Callback,
    objects::{
        account::AccountTrace,
        ans_host::AnsHost,
        namespace::{Namespace, ABSTRACT_NAMESPACE},
        AccountId, TruncatedChainId,
    },
};
use abstract_dex_standard::{
    ans_action::WholeDexAction, msg::ExecuteMsg, raw_action::DexRawAction, DexError, DEX_ADAPTER_ID,
};
use cosmwasm_std::{
    ensure_eq, to_json_binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
};
use cw_asset::AssetBase;

use crate::{
    contract::{DexAdapter, DexResult},
    exchanges::exchange_resolver,
    handlers::execute::exchange_resolver::is_over_ibc,
    msg::{DexExecuteMsg, DexName},
    state::DEX_FEES,
};

use abstract_adapter::sdk::features::AccountIdentification;

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    adapter: DexAdapter,
    msg: DexExecuteMsg,
) -> DexResult {
    match msg {
        DexExecuteMsg::AnsAction {
            dex: dex_name,
            action,
        } => {
            let (local_dex_name, is_over_ibc) = is_over_ibc(&env, &dex_name)?;
            // We resolve the Action to a RawAction to get the actual addresses, ids and denoms
            let whole_dex_action = WholeDexAction(local_dex_name.clone(), action);
            let ans = adapter.name_service(deps.as_ref());
            let raw_action = ans.query(&whole_dex_action)?;

            // if exchange is on an app-chain, execute the action on the app-chain
            if is_over_ibc {
                handle_ibc_request(&deps, info, &adapter, local_dex_name, &raw_action)
            } else {
                // the action can be executed on the local chain
                handle_local_request(deps, env, info, &adapter, local_dex_name, raw_action)
            }
        }
        DexExecuteMsg::RawAction {
            dex: dex_name,
            action,
        } => {
            let (local_dex_name, is_over_ibc) = is_over_ibc(&env, &dex_name)?;

            // if exchange is on an app-chain, execute the action on the app-chain
            if is_over_ibc {
                handle_ibc_request(&deps, info, &adapter, local_dex_name, &action)
            } else {
                // the action can be executed on the local chain
                handle_local_request(deps, env, info, &adapter, local_dex_name, action)
            }
        }
        DexExecuteMsg::UpdateFee {
            swap_fee,
            recipient_account: recipient_account_id,
        } => {
            // Only namespace owner (abstract) can change recipient address
            let namespace = adapter
                .module_registry(deps.as_ref())?
                .query_namespace(Namespace::new(ABSTRACT_NAMESPACE)?)?;

            // unwrap namespace, since it's unlikely to have unclaimed abstract namespace
            let namespace_info = namespace.unwrap();
            ensure_eq!(
                namespace_info.account_base,
                adapter.target_account.clone().unwrap(),
                DexError::Unauthorized {}
            );
            let mut fee = DEX_FEES.load(deps.storage)?;

            // Update swap fee
            if let Some(swap_fee) = swap_fee {
                fee.set_swap_fee_share(swap_fee)?;
            }

            // Update recipient account id
            if let Some(account_id) = recipient_account_id {
                let recipient = adapter
                    .account_registry(deps.as_ref())?
                    .proxy_address(&AccountId::new(account_id, AccountTrace::Local)?)?;
                fee.recipient = recipient;
            }

            DEX_FEES.save(deps.storage, &fee)?;
            Ok(Response::default())
        }
    }
}

/// Handle an adapter request that can be executed on the local chain
fn handle_local_request(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    adapter: &DexAdapter,
    exchange: String,
    action: DexRawAction,
) -> DexResult {
    let exchange = exchange_resolver::resolve_exchange(&exchange)?;
    let target_account = adapter.account_base(deps.as_ref())?;
    let (msgs, _) = crate::adapter::DexAdapter::resolve_dex_action(
        adapter,
        deps.as_ref(),
        target_account.proxy,
        action,
        exchange,
    )?;
    let proxy_msg = adapter
        .executor(deps.as_ref())
        .execute(msgs.into_iter().map(Into::into).collect())?;
    Ok(Response::new().add_message(proxy_msg))
}

/// Handle an adapter request that can be executed on an IBC chain
/// TODO, this doesn't work as is, would have to change this for working with IBC hooks
fn handle_ibc_request(
    deps: &DepsMut,
    info: MessageInfo,
    adapter: &DexAdapter,
    dex_name: DexName,
    action: &DexRawAction,
) -> DexResult {
    let host_chain = TruncatedChainId::from_string(dex_name.clone())?; // TODO, this is faulty

    let ans = adapter.name_service(deps.as_ref());
    let ibc_client = adapter.ibc_client(deps.as_ref());
    // get the to-be-sent assets from the action
    let coins = resolve_assets_to_transfer(deps.as_ref(), action, ans.host())?;
    // construct the ics20 call(s)
    let ics20_transfer_msg = ibc_client.ics20_transfer(host_chain.clone(), coins, None)?;
    // construct the action to be called on the host
    let host_action = abstract_adapter::std::ibc_host::HostAction::Dispatch {
        manager_msgs: vec![abstract_adapter::std::manager::ExecuteMsg::ExecOnModule {
            module_id: DEX_ADAPTER_ID.to_string(),
            exec_msg: to_json_binary::<ExecuteMsg>(
                &DexExecuteMsg::RawAction {
                    dex: dex_name.clone(),
                    action: action.clone(),
                }
                .into(),
            )?,
        }],
    };

    // If the calling entity is a contract, we provide a callback on successful swap
    let maybe_contract_info = deps.querier.query_wasm_contract_info(info.sender.clone());
    let _callback = if maybe_contract_info.is_err() {
        None
    } else {
        Some(Callback {
            msg: to_json_binary(&DexExecuteMsg::RawAction {
                dex: dex_name.clone(),
                action: action.clone(),
            })?,
        })
    };
    let ibc_action_msg = ibc_client.host_action(host_chain, host_action)?;

    // call both messages on the proxy
    Ok(Response::new().add_messages(vec![ics20_transfer_msg, ibc_action_msg]))
}

pub(crate) fn resolve_assets_to_transfer(
    deps: Deps,
    dex_action: &DexRawAction,
    _ans_host: &AnsHost,
) -> DexResult<Vec<Coin>> {
    // resolve asset to native asset
    let offer_to_coin = |offer: &AssetBase<String>| {
        offer
            .check(deps.api, None)
            .and_then(|a| a.try_into())
            .map_err(DexError::from)
    };

    match dex_action {
        DexRawAction::ProvideLiquidity { assets, .. } => {
            let coins: Result<Vec<Coin>, _> = assets.iter().map(offer_to_coin).collect();
            coins
        }
        DexRawAction::ProvideLiquiditySymmetric { .. } => Err(DexError::Std(
            StdError::generic_err("Cross-chain symmetric provide liquidity not supported."),
        )),
        DexRawAction::WithdrawLiquidity { lp_token, .. } => Ok(vec![offer_to_coin(lp_token)?]),
        DexRawAction::Swap { offer_asset, .. } => Ok(vec![offer_to_coin(offer_asset)?]),
        DexRawAction::RouteSwap { .. } => todo!(),
    }
    .map_err(Into::into)
}
