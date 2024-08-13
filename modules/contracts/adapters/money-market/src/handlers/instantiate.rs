use abstract_adapter::sdk::AccountVerification;
use abstract_adapter::std::objects::{account::AccountTrace, fee::UsageFee, AccountId};
use abstract_money_market_standard::msg::MoneyMarketInstantiateMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::{MoneyMarketAdapter, MoneyMarketResult},
    state::MONEY_MARKET_FEES,
};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    module: MoneyMarketAdapter,
    msg: MoneyMarketInstantiateMsg,
) -> MoneyMarketResult {
    let recipient = module
        .account_registry(deps.as_ref())?
        .proxy_address(&AccountId::new(msg.recipient_account, AccountTrace::Local)?)?;
    let money_market_fees = UsageFee::new(msg.fee, recipient)?;
    MONEY_MARKET_FEES.save(deps.storage, &money_market_fees)?;
    Ok(Response::default())
}
