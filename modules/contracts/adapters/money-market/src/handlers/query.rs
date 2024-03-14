use abstract_money_market_standard::{
    ans_action::ActionOnMoneymarket,
    msg::{
        GenerateMessagesResponse, MoneyMarketExecuteMsg, MoneyMarketFeesResponse,
        MoneyMarketQueryMsg,
    },
    query::{MoneyMarketRawQuery, WholeMoneyMarketQuery},
    MoneyMarketError,
};
use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env};

use crate::{
    contract::{MoneyMarketAdapter, MoneyMarketResult},
    platform_resolver::{self, is_over_ibc},
    state::MONEYMARKET_FEES,
};

pub fn query_handler(
    deps: Deps,
    env: Env,
    adapter: &MoneyMarketAdapter,
    mut msg: MoneyMarketQueryMsg,
) -> MoneyMarketResult<Binary> {
    if let MoneyMarketQueryMsg::MoneyMarketAnsQuery {
        query,
        money_market,
    } = msg
    {
        let ans = adapter.name_service(deps);
        let whole_money_market_query = WholeMoneyMarketQuery(
            platform_resolver::resolve_money_market(&money_market)?,
            query,
        );
        msg = MoneyMarketQueryMsg::MoneyMarketRawQuery {
            query: ans.query(&whole_money_market_query)?,
            money_market,
        };
    }

    match msg {
        MoneyMarketQueryMsg::GenerateMessages {
            mut message,
            addr_as_sender,
        } => {
            if let MoneyMarketExecuteMsg::AnsAction {
                money_market,
                action,
            } = message
            {
                let ans = adapter.name_service(deps);
                let whole_money_market_action = ActionOnMoneymarket(
                    platform_resolver::resolve_money_market(&money_market)?,
                    action,
                );
                message = MoneyMarketExecuteMsg::RawAction {
                    money_market,
                    action: ans.query(&whole_money_market_action)?,
                }
            }
            match message {
                MoneyMarketExecuteMsg::RawAction {
                    money_market,
                    action,
                } => {
                    let (local_money_market_name, is_over_ibc) = is_over_ibc(env, &money_market)?;
                    // if exchange is on an app-chain, execute the action on the app-chain
                    if is_over_ibc {
                        return Err(MoneyMarketError::IbcMsgQuery);
                    }
                    let exchange =
                        platform_resolver::resolve_money_market(&local_money_market_name)?;
                    let addr_as_sender = deps.api.addr_validate(&addr_as_sender)?;
                    let (messages, _) =
                        crate::adapter::MoneyMarketAdapter::resolve_money_market_action(
                            adapter,
                            deps,
                            addr_as_sender,
                            action,
                            exchange,
                        )?;
                    to_json_binary(&GenerateMessagesResponse { messages }).map_err(Into::into)
                }
                _ => Err(MoneyMarketError::InvalidGenerateMessage {}),
            }
        }
        MoneyMarketQueryMsg::Fees {} => fees(deps),
        MoneyMarketQueryMsg::MoneyMarketRawQuery {
            query,
            money_market,
        } => {
            let (local_money_market_name, is_over_ibc) = is_over_ibc(env.clone(), &money_market)?;

            // if money_market is on an app-chain, execute the action on the app-chain
            if is_over_ibc {
                unimplemented!()
            } else {
                // the action can be executed on the local chain
                handle_local_query(deps, env, local_money_market_name, adapter, query)
            }
        }
        _ => Err(MoneyMarketError::IbcMsgQuery {}),
    }
}

pub fn fees(deps: Deps) -> MoneyMarketResult<Binary> {
    let money_market_fees = MONEYMARKET_FEES.load(deps.storage)?;
    let resp = MoneyMarketFeesResponse {
        money_market_fee: money_market_fees.swap_fee(),
        recipient: money_market_fees.recipient,
    };
    to_json_binary(&resp).map_err(Into::into)
}

/// Handle an adapter request that can be executed on the local chain
fn handle_local_query(
    deps: Deps,
    _env: Env,
    money_market: String,
    adapter: &MoneyMarketAdapter,
    query: MoneyMarketRawQuery,
) -> MoneyMarketResult<Binary> {
    let mut money_market = platform_resolver::resolve_money_market(&money_market)?;
    let ans_host = adapter.ans_host(deps)?;
    money_market.fetch_data(&deps.querier, &ans_host)?;
    Ok(match query {
        MoneyMarketRawQuery::UserDeposit {
            user,
            asset,
            contract_addr,
        } => {
            let user = deps.api.addr_validate(&user)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            let asset = asset.check(deps.api, None)?;

            to_json_binary(&money_market.user_deposit(deps, contract_addr, user, asset)?)?
        }
        MoneyMarketRawQuery::UserCollateral {
            user,
            collateral_asset,
            borrowed_asset,
            contract_addr,
        } => {
            let user = deps.api.addr_validate(&user)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            let collateral_asset = collateral_asset.check(deps.api, None)?;
            let borrowed_asset = borrowed_asset.check(deps.api, None)?;

            to_json_binary(&money_market.user_collateral(
                deps,
                contract_addr,
                user,
                borrowed_asset,
                collateral_asset,
            )?)?
        }
        MoneyMarketRawQuery::UserBorrow {
            user,
            collateral_asset,
            borrowed_asset,
            contract_addr,
        } => {
            let user = deps.api.addr_validate(&user)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            let collateral_asset = collateral_asset.check(deps.api, None)?;
            let borrowed_asset = borrowed_asset.check(deps.api, None)?;

            to_json_binary(&money_market.user_borrow(
                deps,
                contract_addr,
                user,
                borrowed_asset,
                collateral_asset,
            )?)?
        }
        MoneyMarketRawQuery::CurrentLTV {
            user,
            collateral_asset,
            borrowed_asset,
            contract_addr,
        } => {
            let user = deps.api.addr_validate(&user)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            let collateral_asset = collateral_asset.check(deps.api, None)?;
            let borrowed_asset = borrowed_asset.check(deps.api, None)?;

            to_json_binary(&money_market.current_ltv(
                deps,
                contract_addr,
                user,
                borrowed_asset,
                collateral_asset,
            )?)?
        }
        MoneyMarketRawQuery::MaxLTV {
            user,
            collateral_asset,
            borrowed_asset,
            contract_addr,
        } => {
            let user = deps.api.addr_validate(&user)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            let collateral_asset = collateral_asset.check(deps.api, None)?;
            let borrowed_asset = borrowed_asset.check(deps.api, None)?;

            to_json_binary(&money_market.max_ltv(
                deps,
                contract_addr,
                user,
                borrowed_asset,
                collateral_asset,
            )?)?
        }
        MoneyMarketRawQuery::Price { quote, base } => {
            let quote = quote.check(deps.api, None)?;
            let base = base.check(deps.api, None)?;

            to_json_binary(&money_market.price(deps, base, quote)?)?
        }
    })
}
