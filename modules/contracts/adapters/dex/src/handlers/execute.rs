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
use abstract_dex_standard::{action::DexAction, msg::ExecuteMsg, DexError, DEX_ADAPTER_ID};
use cosmwasm_std::{ensure_eq, to_json_binary, Coin, Deps, DepsMut, Env, MessageInfo, Response};
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
    module: DexAdapter,
    msg: DexExecuteMsg,
) -> DexResult {
    match msg {
        DexExecuteMsg::Action {
            dex: dex_name,
            action,
        } => {
            let (local_dex_name, is_over_ibc) = is_over_ibc(&env, &dex_name)?;

            // if exchange is on an app-chain, execute the action on the app-chain
            if is_over_ibc {
                handle_ibc_request(&deps, info, &module, local_dex_name, &action)
            } else {
                // the action can be executed on the local chain
                handle_local_request(deps, info, &module, local_dex_name, action)
            }
        }
        DexExecuteMsg::UpdateFee {
            swap_fee,
            recipient_account: recipient_account_id,
        } => {
            // Only namespace owner (abstract) can change recipient address
            let namespace = module
                .module_registry(deps.as_ref())?
                .query_namespace(Namespace::new(ABSTRACT_NAMESPACE)?)?;

            // unwrap namespace, since it's unlikely to have unclaimed abstract namespace
            let namespace_info = namespace.unwrap();
            ensure_eq!(
                namespace_info.account,
                module.target_account.clone().unwrap(),
                DexError::Unauthorized {}
            );
            let mut fee = DEX_FEES.load(deps.storage)?;

            // Update swap fee
            if let Some(swap_fee) = swap_fee {
                fee.set_swap_fee_share(swap_fee)?;
            }

            // Update recipient account id
            if let Some(account_id) = recipient_account_id {
                let recipient = module
                    .account_registry(deps.as_ref())?
                    .account(&AccountId::new(account_id, AccountTrace::Local)?)?;
                fee.recipient = recipient.into_addr();
            }

            DEX_FEES.save(deps.storage, &fee)?;
            Ok(Response::default())
        }
    }
}

/// Handle an adapter request that can be executed on the local chain
fn handle_local_request(
    deps: DepsMut,
    _info: MessageInfo,
    module: &DexAdapter,
    exchange: String,
    action: DexAction,
) -> DexResult {
    let exchange = exchange_resolver::resolve_exchange(&exchange)?;
    let target_account = module.account(deps.as_ref())?;
    let (msgs, _) = crate::adapter::DexAdapter::resolve_dex_action(
        module,
        deps.as_ref(),
        target_account.into_addr(),
        action,
        exchange,
    )?;
    let account_msg = module.executor(deps.as_ref()).execute(msgs)?;
    Ok(Response::new().add_message(account_msg))
}

/// Handle an adapter request that can be executed on an IBC chain
fn handle_ibc_request(
    deps: &DepsMut,
    info: MessageInfo,
    module: &DexAdapter,
    dex_name: DexName,
    action: &DexAction,
) -> DexResult {
    let host_chain = TruncatedChainId::from_string(dex_name.clone())?;

    let ans = module.name_service(deps.as_ref());
    let ibc_client = module.ibc_client(deps.as_ref());
    // get the to-be-sent assets from the action
    let coins = resolve_assets_to_transfer(deps.as_ref(), action, ans.host())?;
    // construct the ics20 call(s)
    let ics20_transfer_msg = ibc_client.ics20_transfer(host_chain.clone(), coins, None, None)?;
    // construct the action to be called on the host
    let host_action = abstract_adapter::std::ibc_host::HostAction::Dispatch {
        account_msgs: vec![
            abstract_adapter::std::account::ExecuteMsg::ExecuteOnModule {
                module_id: DEX_ADAPTER_ID.to_string(),
                exec_msg: to_json_binary::<ExecuteMsg>(
                    &DexExecuteMsg::Action {
                        dex: dex_name.clone(),
                        action: action.clone(),
                    }
                    .into(),
                )?,
                funds: vec![],
            },
        ],
    };

    // If the calling entity is a contract, we provide a callback on successful swap
    let maybe_contract_info = deps.querier.query_wasm_contract_info(info.sender.clone());
    let _callback = if maybe_contract_info.is_err() {
        None
    } else {
        Some(Callback {
            msg: to_json_binary(&DexExecuteMsg::Action {
                dex: dex_name.clone(),
                action: action.clone(),
            })?,
        })
    };
    let ibc_action_msg = ibc_client.host_action(host_chain, host_action)?;

    // call both messages on the account
    Ok(Response::new().add_messages(vec![ics20_transfer_msg, ibc_action_msg]))
}

pub(crate) fn resolve_assets_to_transfer(
    deps: Deps,
    dex_action: &DexAction,
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
        DexAction::RouteSwap { offer_asset, .. } => Ok(vec![offer_to_coin(offer_asset)?]),
        DexAction::Swap { offer_asset, .. } => Ok(vec![offer_to_coin(offer_asset)?]),
        DexAction::ProvideLiquidity { assets, .. } => {
            let coins: Result<Vec<Coin>, _> = assets.iter().map(offer_to_coin).collect();
            coins
        }
        DexAction::WithdrawLiquidity { lp_token, .. } => Ok(vec![offer_to_coin(lp_token)?]),
    }
    .map_err(Into::into)
}
