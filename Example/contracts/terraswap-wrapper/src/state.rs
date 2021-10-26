use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Decimal};
use cw_controllers::Admin;
use cw_storage_plus::Item;
use terraswap::asset::Asset;
use white_whale::deposit_info::DepositInfo;
use white_whale::trader::Trader;

/*
    terraswap_pool_addr: address of terraswap pool contract
    lp_token_addr: address of LP token (cw20)
    max_deposit: If the contract owns more liquidity than this value, profits can be withdrawn until this value
    min_profit: Minimum profit before profits can be withdrawn.
    slippage: Max allowed slippage on adding to the LP
*/

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub terraswap_pool_addr: CanonicalAddr,
    pub lp_token_addr: CanonicalAddr,
    pub max_deposit: Asset,
    pub min_profit: Asset,
    pub slippage: Decimal,
}

pub const STATE: Item<State> = Item::new("\u{0}{5}state");
pub const ADMIN: Admin = Admin::new("admin");
pub const TRADER: Trader = Trader::new("trader");
pub const DEPOSIT_INFO: Item<DepositInfo> = Item::new("\u{0}{7}deposit");
