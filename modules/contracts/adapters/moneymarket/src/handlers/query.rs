use abstract_moneymarket_standard::{
    ans_action::WholeMoneymarketAction,
    msg::{
        GenerateMessagesResponse, MoneymarketExecuteMsg, MoneymarketFeesResponse,
        MoneymarketQueryMsg,
    },
    query::{MoneymarketRawQuery, WholeMoneymarketQuery},
    MoneymarketError,
};
use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env};

use crate::{
    contract::{MoneymarketAdapter, MoneymarketResult},
    platform_resolver::{self, is_over_ibc},
    state::MONEYMARKET_FEES,
};

pub fn query_handler(
    deps: Deps,
    env: Env,
    adapter: &MoneymarketAdapter,
    mut msg: MoneymarketQueryMsg,
) -> MoneymarketResult<Binary> {
    if let MoneymarketQueryMsg::MoneymarketAnsQuery { query, moneymarket } = msg {
        let ans = adapter.name_service(deps);
        let whole_moneymarket_query =
            WholeMoneymarketQuery(platform_resolver::resolve_moneymarket(&moneymarket)?, query);
        msg = MoneymarketQueryMsg::MoneymarketRawQuery {
            query: ans.query(&whole_moneymarket_query)?,
            moneymarket,
        };
    }

    match msg {
        MoneymarketQueryMsg::GenerateMessages {
            mut message,
            addr_as_sender,
        } => {
            if let MoneymarketExecuteMsg::AnsAction {
                moneymarket,
                action,
            } = message
            {
                let ans = adapter.name_service(deps);
                let whole_moneymarket_action = WholeMoneymarketAction(
                    platform_resolver::resolve_moneymarket(&moneymarket)?,
                    action,
                );
                message = MoneymarketExecuteMsg::RawAction {
                    moneymarket,
                    action: ans.query(&whole_moneymarket_action)?,
                }
            }
            match message {
                MoneymarketExecuteMsg::RawAction {
                    moneymarket,
                    action,
                } => {
                    let (local_moneymarket_name, is_over_ibc) = is_over_ibc(env, &moneymarket)?;
                    // if exchange is on an app-chain, execute the action on the app-chain
                    if is_over_ibc {
                        return Err(MoneymarketError::IbcMsgQuery);
                    }
                    let exchange = platform_resolver::resolve_moneymarket(&local_moneymarket_name)?;
                    let addr_as_sender = deps.api.addr_validate(&addr_as_sender)?;
                    let (messages, _) =
                        crate::adapter::MoneymarketAdapter::resolve_moneymarket_action(
                            adapter,
                            deps,
                            addr_as_sender,
                            action,
                            exchange,
                        )?;
                    to_json_binary(&GenerateMessagesResponse { messages }).map_err(Into::into)
                }
                _ => Err(MoneymarketError::InvalidGenerateMessage {}),
            }
        }
        MoneymarketQueryMsg::Fees {} => fees(deps),
        MoneymarketQueryMsg::MoneymarketRawQuery { query, moneymarket } => {
            let (local_moneymarket_name, is_over_ibc) = is_over_ibc(env.clone(), &moneymarket)?;

            // if moneymarket is on an app-chain, execute the action on the app-chain
            if is_over_ibc {
                unimplemented!()
            } else {
                // the action can be executed on the local chain
                handle_local_query(deps, env, &adapter, local_moneymarket_name, query)
            }
        }
        _ => Err(MoneymarketError::IbcMsgQuery {}),
    }
}

pub fn fees(deps: Deps) -> MoneymarketResult<Binary> {
    let moneymarket_fees = MONEYMARKET_FEES.load(deps.storage)?;
    let resp = MoneymarketFeesResponse {
        moneymarket_fee: moneymarket_fees.swap_fee(),
        recipient: moneymarket_fees.recipient,
    };
    to_json_binary(&resp).map_err(Into::into)
}

/// Handle an adapter request that can be executed on the local chain
fn handle_local_query(
    deps: Deps,
    _env: Env,
    adapter: &MoneymarketAdapter,
    moneymarket: String,
    query: MoneymarketRawQuery,
) -> MoneymarketResult<Binary> {
    let moneymarket = platform_resolver::resolve_moneymarket(&moneymarket)?;

    Ok(match query {
        MoneymarketRawQuery::UserDeposit {
            user,
            asset,
            contract_addr,
        } => {
            let user = deps.api.addr_validate(&user)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            let asset = asset.check(deps.api, None)?;

            to_json_binary(&moneymarket.user_deposit(deps, contract_addr, user, asset)?)?
        }
        MoneymarketRawQuery::UserCollateral {
            user,
            collateral_asset,
            borrowed_asset,
            contract_addr,
        } => {
            let user = deps.api.addr_validate(&user)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            let collateral_asset = collateral_asset.check(deps.api, None)?;
            let borrowed_asset = borrowed_asset.check(deps.api, None)?;

            to_json_binary(&moneymarket.user_collateral(
                deps,
                contract_addr,
                user,
                borrowed_asset,
                collateral_asset,
            )?)?
        }
        MoneymarketRawQuery::UserBorrow {
            user,
            collateral_asset,
            borrowed_asset,
            contract_addr,
        } => todo!(),
        MoneymarketRawQuery::CurrentLTV {
            user,
            collateral_asset,
            borrowed_asset,
            contract_addr,
        } => todo!(),
        MoneymarketRawQuery::MaxLTV {
            user,
            collateral_asset,
            borrowed_asset,
            contract_addr,
        } => todo!(),
        MoneymarketRawQuery::Price { quote, base } => todo!(),
    })
}
