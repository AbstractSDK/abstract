use abstract_adapter::sdk::features::AbstractNameService;
use abstract_money_market_standard::{
    ans_action::MoneyMarketActionResolveWrapper,
    msg::{
        GenerateMessagesResponse, MoneyMarketExecuteMsg, MoneyMarketQueryMsg, PriceResponse,
        UserBorrowResponse, UserCollateralResponse, UserCurrentLTVResponse, UserDepositResponse,
        UserMaxLTVResponse,
    },
    query::MoneyMarketQueryResolveWrapper,
    MoneyMarketError,
};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdError};

use crate::{
    contract::{MoneyMarketAdapter, MoneyMarketResult},
    platform_resolver::{self, is_over_ibc},
    state::MONEY_MARKET_FEES,
};

pub fn query_handler(
    deps: Deps,
    env: Env,
    module: &MoneyMarketAdapter,
    msg: MoneyMarketQueryMsg,
) -> MoneyMarketResult<Binary> {
    let ans = module.name_service(deps);
    let whole_money_market_query =
        MoneyMarketQueryResolveWrapper(platform_resolver::resolve_money_market, msg);
    let msg = ans.query(&whole_money_market_query)?;

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
                let ans = module.name_service(deps);
                let whole_money_market_action = MoneyMarketActionResolveWrapper(
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
                    let (local_money_market_name, is_over_ibc) = is_over_ibc(&env, &money_market)?;
                    // if exchange is on an app-chain, execute the action on the app-chain
                    if is_over_ibc {
                        return Err(MoneyMarketError::IbcMsgQuery);
                    }
                    let money_market =
                        platform_resolver::resolve_money_market(&local_money_market_name)?;
                    let addr_as_sender = deps.api.addr_validate(&addr_as_sender)?;

                    let (messages, _) =
                        crate::adapter::MoneyMarketAdapter::resolve_money_market_action(
                            module,
                            deps,
                            addr_as_sender,
                            action,
                            money_market,
                        )?;
                    to_json_binary(&GenerateMessagesResponse { messages }).map_err(Into::into)
                }
                _ => Err(MoneyMarketError::InvalidGenerateMessage {}),
            }
        }
        MoneyMarketQueryMsg::Fees {} => fees(deps),
        _ => {
            let money_market = msg.money_market()?;

            let (local_money_market_name, is_over_ibc) = is_over_ibc(&env, money_market)?;

            // if money_market is on an app-chain, execute the action on the app-chain
            if is_over_ibc {
                unimplemented!()
            } else {
                // the action can be executed on the local chain
                handle_local_query(deps, env, local_money_market_name, module, msg)
            }
        }
    }
}

pub fn fees(deps: Deps) -> MoneyMarketResult<Binary> {
    let money_market_fees = MONEY_MARKET_FEES.load(deps.storage)?;

    to_json_binary(&money_market_fees).map_err(Into::into)
}

/// Handle an adapter request that can be executed on the local chain
/// We only execute local queries here
fn handle_local_query(
    deps: Deps,
    env: Env,
    money_market: String,
    module: &MoneyMarketAdapter,
    query: MoneyMarketQueryMsg,
) -> MoneyMarketResult<Binary> {
    let mut money_market = platform_resolver::resolve_money_market(&money_market)?;
    let ans_host = module.ans_host(deps)?;
    Ok(match query {
        MoneyMarketQueryMsg::RawUserDeposit {
            user,
            asset,
            contract_addr,
            money_market: _,
        } => {
            let user = deps.api.addr_validate(&user)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            let asset = asset.check(deps.api, None)?;

            money_market.fetch_data(user.clone(), &deps.querier, &ans_host)?;
            to_json_binary(&UserDepositResponse {
                amount: money_market.user_deposit(deps, contract_addr, user, asset)?,
            })?
        }
        MoneyMarketQueryMsg::RawUserCollateral {
            user,
            collateral_asset,
            borrowed_asset,
            contract_addr,
            money_market: _,
        } => {
            let user = deps.api.addr_validate(&user)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            let collateral_asset = collateral_asset.check(deps.api, None)?;
            let borrowed_asset = borrowed_asset.check(deps.api, None)?;

            money_market.fetch_data(user.clone(), &deps.querier, &ans_host)?;
            to_json_binary(&UserCollateralResponse {
                amount: money_market.user_collateral(
                    deps,
                    contract_addr,
                    user,
                    borrowed_asset,
                    collateral_asset,
                )?,
            })?
        }
        MoneyMarketQueryMsg::RawUserBorrow {
            user,
            collateral_asset,
            borrowed_asset,
            contract_addr,
            money_market: _,
        } => {
            let user = deps.api.addr_validate(&user)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            let collateral_asset = collateral_asset.check(deps.api, None)?;
            let borrowed_asset = borrowed_asset.check(deps.api, None)?;

            money_market.fetch_data(user.clone(), &deps.querier, &ans_host)?;
            to_json_binary(&UserBorrowResponse {
                amount: money_market.user_borrow(
                    deps,
                    contract_addr,
                    user,
                    borrowed_asset,
                    collateral_asset,
                )?,
            })?
        }
        MoneyMarketQueryMsg::RawCurrentLTV {
            user,
            collateral_asset,
            borrowed_asset,
            contract_addr,
            money_market: _,
        } => {
            let user = deps.api.addr_validate(&user)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            let collateral_asset = collateral_asset.check(deps.api, None)?;
            let borrowed_asset = borrowed_asset.check(deps.api, None)?;

            money_market.fetch_data(user.clone(), &deps.querier, &ans_host)?;
            to_json_binary(&UserCurrentLTVResponse {
                current_ltv: money_market.current_ltv(
                    deps,
                    contract_addr,
                    user,
                    borrowed_asset,
                    collateral_asset,
                )?,
            })?
        }
        MoneyMarketQueryMsg::RawMaxLTV {
            user,
            collateral_asset,
            borrowed_asset,
            contract_addr,
            money_market: _,
        } => {
            let user = deps.api.addr_validate(&user)?;
            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            let collateral_asset = collateral_asset.check(deps.api, None)?;
            let borrowed_asset = borrowed_asset.check(deps.api, None)?;

            money_market.fetch_data(user.clone(), &deps.querier, &ans_host)?;
            to_json_binary(&UserMaxLTVResponse {
                max_ltv: money_market.max_ltv(
                    deps,
                    contract_addr,
                    user,
                    borrowed_asset,
                    collateral_asset,
                )?,
            })?
        }
        MoneyMarketQueryMsg::RawPrice {
            quote,
            base,
            money_market: _,
        } => {
            let quote = quote.check(deps.api, None)?;
            let base = base.check(deps.api, None)?;

            money_market.fetch_data(env.contract.address.clone(), &deps.querier, &ans_host)?;
            to_json_binary(&PriceResponse {
                price: money_market.price(deps, base, quote)?,
            })?
        }
        _ => {
            return Err(
                StdError::generic_err("Can't treat non-local ans query, unreachable").into(),
            )
        }
    })
}
