use abstract_core::objects::{account::AccountTrace, AccountId};
use abstract_money_market_standard::msg::{MoneyMarketFees, MoneyMarketInstantiateMsg};
use abstract_sdk::AccountVerification;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::{MoneyMarketAdapter, MoneyMarketResult},
    state::MONEYMARKET_FEES,
};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    adapter: MoneyMarketAdapter,
    msg: MoneyMarketInstantiateMsg,
) -> MoneyMarketResult {
    let recipient = adapter
        .account_registry(deps.as_ref())?
        .proxy_address(&AccountId::new(msg.recipient_account, AccountTrace::Local)?)?;
    let money_market_fees = MoneyMarketFees::new(msg.swap_fee, recipient)?;
    MONEYMARKET_FEES.save(deps.storage, &money_market_fees)?;
    Ok(Response::default())
}
