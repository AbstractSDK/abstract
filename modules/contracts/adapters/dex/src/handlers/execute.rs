use crate::handlers::execute::exchange_resolver::is_over_ibc;
use crate::DEX_ADAPTER_ID;

use crate::contract::{DexAdapter, DexResult};
use crate::exchanges::exchange_resolver;
use crate::msg::{DexAction, DexExecuteMsg, DexName};
use crate::state::DEX_FEES;
use abstract_core::ibc::CallbackInfo;
use abstract_core::objects::account::AccountTrace;
use abstract_core::objects::chain_name::ChainName;
use abstract_core::objects::namespace::{Namespace, ABSTRACT_NAMESPACE};
use abstract_dex_standard::msg::{ExecuteMsg, IBC_DEX_PROVIDER_ID};
use abstract_dex_standard::DexError;

use abstract_core::objects::ans_host::AnsHost;
use abstract_core::objects::{AccountId, AnsAsset};
use abstract_sdk::{features::AbstractNameService, Execution};
use abstract_sdk::{AccountVerification, IbcInterface, ModuleRegistryInterface, Resolve};
use cosmwasm_std::{
    ensure_eq, to_json_binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    adapter: DexAdapter,
    msg: DexExecuteMsg,
) -> DexResult {
    match msg {
        DexExecuteMsg::Action {
            dex: dex_name,
            action,
        } => {
            let (local_dex_name, is_over_ibc) = is_over_ibc(env.clone(), &dex_name)?;
            // if exchange is on an app-chain, execute the action on the app-chain
            if is_over_ibc {
                handle_ibc_request(&deps, info, &adapter, local_dex_name, &action)
            } else {
                // the action can be executed on the local chain
                handle_local_request(deps, env, info, adapter, action, local_dex_name)
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
    info: MessageInfo,
    adapter: DexAdapter,
    action: DexAction,
    exchange: String,
) -> DexResult {
    let exchange = exchange_resolver::resolve_exchange(&exchange)?;
    let (msgs, _) = crate::adapter::DexAdapter::resolve_dex_action(
        &adapter,
        deps.as_ref(),
        info.sender,
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
    action: &DexAction,
) -> DexResult {
    let host_chain = ChainName::from_string(dex_name.clone())?; // TODO, this is faulty

    let ans = adapter.name_service(deps.as_ref());
    let ibc_client = adapter.ibc_client(deps.as_ref());
    // get the to-be-sent assets from the action
    let coins = resolve_assets_to_transfer(deps.as_ref(), action, ans.host())?;
    // construct the ics20 call(s)
    let ics20_transfer_msg = ibc_client.ics20_transfer(host_chain.to_string(), coins)?;
    // construct the action to be called on the host
    let host_action = abstract_sdk::core::ibc_host::HostAction::Dispatch {
        manager_msg: abstract_core::manager::ExecuteMsg::ExecOnModule {
            module_id: DEX_ADAPTER_ID.to_string(),
            exec_msg: to_json_binary::<ExecuteMsg>(
                &DexExecuteMsg::Action {
                    dex: dex_name.clone(),
                    action: action.clone(),
                }
                .into(),
            )?,
        },
    };

    // If the calling entity is a contract, we provide a callback on successful swap
    let maybe_contract_info = deps.querier.query_wasm_contract_info(info.sender.clone());
    let callback = if maybe_contract_info.is_err() {
        None
    } else {
        Some(CallbackInfo {
            id: IBC_DEX_PROVIDER_ID.into(),
            msg: Some(to_json_binary(&DexExecuteMsg::Action {
                dex: dex_name.clone(),
                action: action.clone(),
            })?),
            receiver: info.sender.into_string(),
        })
    };
    let ibc_action_msg = ibc_client.host_action(host_chain.to_string(), host_action, callback)?;

    // call both messages on the proxy
    Ok(Response::new().add_messages(vec![ics20_transfer_msg, ibc_action_msg]))
}

pub(crate) fn resolve_assets_to_transfer(
    deps: Deps,
    dex_action: &DexAction,
    ans_host: &AnsHost,
) -> DexResult<Vec<Coin>> {
    // resolve asset to native asset
    let offer_to_coin = |offer: &AnsAsset| {
        offer
            .resolve(&deps.querier, ans_host)?
            .try_into()
            .map_err(DexError::from)
    };

    match dex_action {
        DexAction::ProvideLiquidity { assets, .. } => {
            let coins: Result<Vec<Coin>, _> = assets.iter().map(offer_to_coin).collect();
            coins
        }
        DexAction::ProvideLiquiditySymmetric { .. } => Err(DexError::Std(StdError::generic_err(
            "Cross-chain symmetric provide liquidity not supported.",
        ))),
        DexAction::WithdrawLiquidity { lp_token, amount } => Ok(vec![offer_to_coin(&AnsAsset {
            name: lp_token.to_owned(),
            amount: amount.to_owned(),
        })?]),
        DexAction::Swap { offer_asset, .. } => Ok(vec![offer_to_coin(offer_asset)?]),
    }
    .map_err(Into::into)
}
