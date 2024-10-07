use abstract_adapter::sdk::{
    features::{AbstractNameService, AccountIdentification},
    AccountVerification, Execution, ModuleRegistryInterface,
};
use abstract_adapter::std::objects::{
    account::AccountTrace,
    namespace::{Namespace, ABSTRACT_NAMESPACE},
    AccountId,
};
use abstract_money_market_standard::{
    ans_action::MoneyMarketActionResolveWrapper, raw_action::MoneyMarketRawAction, MoneyMarketError,
};
use cosmwasm_std::{ensure_eq, DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::{MoneyMarketAdapter, MoneyMarketResult},
    handlers::execute::platform_resolver::is_over_ibc,
    msg::MoneyMarketExecuteMsg,
    platform_resolver,
    state::MONEY_MARKET_FEES,
};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: MoneyMarketAdapter,
    msg: MoneyMarketExecuteMsg,
) -> MoneyMarketResult {
    match msg {
        MoneyMarketExecuteMsg::AnsAction {
            money_market: money_market_name,
            action,
        } => {
            let (local_money_market_name, is_over_ibc) = is_over_ibc(&env, &money_market_name)?;
            // We resolve the Action to a RawAction to get the actual addresses, ids and denoms
            let whole_money_market_action = MoneyMarketActionResolveWrapper(
                platform_resolver::resolve_money_market(&local_money_market_name)?,
                action,
            );
            let ans = module.name_service(deps.as_ref(), &env);
            let raw_action = ans.query(&whole_money_market_action)?;

            // if money_market is on an app-chain, execute the action on the app-chain
            if is_over_ibc {
                unimplemented!()
            //  handle_ibc_request(&deps, info, &adapter, local_money_market_name, &raw_action)
            } else {
                // the action can be executed on the local chain
                handle_local_request(
                    deps,
                    env,
                    info,
                    &module,
                    local_money_market_name,
                    raw_action,
                )
            }
        }
        MoneyMarketExecuteMsg::RawAction {
            money_market: money_market_name,
            action,
        } => {
            let (local_money_market_name, is_over_ibc) = is_over_ibc(&env, &money_market_name)?;

            // if money_market is on an app-chain, execute the action on the app-chain
            if is_over_ibc {
                unimplemented!()
                // handle_ibc_request(&deps, info, &adapter, local_money_market_name, &action)
            } else {
                // the action can be executed on the local chain
                handle_local_request(deps, env, info, &module, local_money_market_name, action)
            }
        }
        MoneyMarketExecuteMsg::UpdateFee {
            money_market_fee,
            recipient_account: recipient_account_id,
        } => {
            // Only namespace owner (abstract) can change recipient address
            let namespace = module
                .module_registry(deps.as_ref(), &env)?
                .query_namespace(Namespace::new(ABSTRACT_NAMESPACE)?)?;

            // unwrap namespace, since it's unlikely to have unclaimed abstract namespace
            let namespace_info = namespace.unwrap();
            ensure_eq!(
                namespace_info.account,
                module.target_account.clone().unwrap(),
                MoneyMarketError::Unauthorized {}
            );
            let mut fee = MONEY_MARKET_FEES.load(deps.storage)?;

            // Update swap fee
            if let Some(swap_fee) = money_market_fee {
                fee.set_share(swap_fee)?;
            }

            // Update recipient account id
            if let Some(account_id) = recipient_account_id {
                let recipient = module
                    .account_registry(deps.as_ref(), &env)?
                    .account(&AccountId::new(account_id, AccountTrace::Local)?)?;
                fee.set_recipient(recipient.into_addr());
            }

            MONEY_MARKET_FEES.save(deps.storage, &fee)?;
            Ok(Response::default())
        }
    }
}

/// Handle an adapter request that can be executed on the local chain
fn handle_local_request(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    module: &MoneyMarketAdapter,
    money_market: String,
    action: MoneyMarketRawAction,
) -> MoneyMarketResult {
    let money_market = platform_resolver::resolve_money_market(&money_market)?;
    let target_account = module.account(deps.as_ref())?;

    let (msgs, _) = crate::adapter::MoneyMarketAdapter::resolve_money_market_action(
        module,
        deps.as_ref(),
        &env,
        target_account.into_addr(),
        action,
        money_market,
    )?;
    let account_msg = module.executor(deps.as_ref()).execute(msgs)?;
    Ok(Response::new().add_message(account_msg))
}
