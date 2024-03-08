use abstract_core::{
    ibc::CallbackInfo,
    objects::{
        account::AccountTrace,
        ans_host::AnsHost,
        chain_name::ChainName,
        namespace::{Namespace, ABSTRACT_NAMESPACE},
        AccountId,
    },
};
use abstract_moneymarket_standard::{
    ans_action::WholeMoneymarketAction, msg::ExecuteMsg, raw_action::MoneymarketRawAction,
    MoneymarketError, MONEYMARKET_ADAPTER_ID,
};
use abstract_sdk::{
    features::AbstractNameService, AccountVerification, Execution, IbcInterface,
    ModuleRegistryInterface,
};
use cosmwasm_std::{
    ensure_eq, to_json_binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
};
use cw_asset::AssetBase;

use crate::{
    contract::{MoneymarketAdapter, MoneymarketResult},
    handlers::execute::platform_resolver::is_over_ibc,
    msg::{MoneymarketExecuteMsg, MoneymarketName},
    platform_resolver,
    state::MONEYMARKET_FEES,
};

use abstract_sdk::features::AccountIdentification;

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    adapter: MoneymarketAdapter,
    msg: MoneymarketExecuteMsg,
) -> MoneymarketResult {
    match msg {
        MoneymarketExecuteMsg::AnsAction {
            moneymarket: moneymarket_name,
            action,
        } => {
            let (local_moneymarket_name, is_over_ibc) =
                is_over_ibc(env.clone(), &moneymarket_name)?;
            // We resolve the Action to a RawAction to get the actual addresses, ids and denoms
            let whole_moneymarket_action = WholeMoneymarketAction(
                platform_resolver::resolve_moneymarket(&local_moneymarket_name)?,
                action,
            );
            let ans = adapter.name_service(deps.as_ref());
            let raw_action = ans.query(&whole_moneymarket_action)?;

            // if moneymarket is on an app-chain, execute the action on the app-chain
            if is_over_ibc {
                unimplemented!()
            //  handle_ibc_request(&deps, info, &adapter, local_moneymarket_name, &raw_action)
            } else {
                // the action can be executed on the local chain
                handle_local_request(
                    deps,
                    env,
                    info,
                    &adapter,
                    local_moneymarket_name,
                    raw_action,
                )
            }
        }
        MoneymarketExecuteMsg::RawAction {
            moneymarket: moneymarket_name,
            action,
        } => {
            let (local_moneymarket_name, is_over_ibc) =
                is_over_ibc(env.clone(), &moneymarket_name)?;

            // if moneymarket is on an app-chain, execute the action on the app-chain
            if is_over_ibc {
                unimplemented!()
                // handle_ibc_request(&deps, info, &adapter, local_moneymarket_name, &action)
            } else {
                // the action can be executed on the local chain
                handle_local_request(deps, env, info, &adapter, local_moneymarket_name, action)
            }
        }
        MoneymarketExecuteMsg::UpdateFee {
            moneymarket_fee,
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
                MoneymarketError::Unauthorized {}
            );
            let mut fee = MONEYMARKET_FEES.load(deps.storage)?;

            // Update swap fee
            if let Some(swap_fee) = moneymarket_fee {
                fee.set_swap_fee_share(swap_fee)?;
            }

            // Update recipient account id
            if let Some(account_id) = recipient_account_id {
                let recipient = adapter
                    .account_registry(deps.as_ref())?
                    .proxy_address(&AccountId::new(account_id, AccountTrace::Local)?)?;
                fee.recipient = recipient;
            }

            MONEYMARKET_FEES.save(deps.storage, &fee)?;
            Ok(Response::default())
        }
    }
}

/// Handle an adapter request that can be executed on the local chain
fn handle_local_request(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    adapter: &MoneymarketAdapter,
    moneymarket: String,
    action: MoneymarketRawAction,
) -> MoneymarketResult {
    let moneymarket = platform_resolver::resolve_moneymarket(&moneymarket)?;
    let target_account = adapter.account_base(deps.as_ref())?;
    let (msgs, _) = crate::adapter::MoneymarketAdapter::resolve_moneymarket_action(
        adapter,
        deps.as_ref(),
        target_account.proxy,
        action,
        moneymarket,
    )?;
    let proxy_msg = adapter
        .executor(deps.as_ref())
        .execute(msgs.into_iter().map(Into::into).collect())?;
    Ok(Response::new().add_message(proxy_msg))
}
