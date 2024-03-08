use abstract_core::objects::{AssetEntry, PoolAddress};
use abstract_moneymarket_standard::{
    ans_action::WholeMoneymarketAction,
    msg::{
        GenerateMessagesResponse, MoneymarketExecuteMsg, MoneymarketFeesResponse,
        MoneymarketQueryMsg,
    },
    MoneymarketError,
};
use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdError};

use crate::{
    contract::{MoneymarketAdapter, MoneymarketResult},
    platform_resolver::{self, is_over_ibc, resolve_moneymarket},
    state::MONEYMARKET_FEES,
};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

pub fn query_handler(
    deps: Deps,
    env: Env,
    adapter: &MoneymarketAdapter,
    msg: MoneymarketQueryMsg,
) -> MoneymarketResult<Binary> {
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
