use crate::fee::{Fee, VaultFee};
use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, CosmosMsg, Decimal, StdResult, Uint128, WasmMsg,
};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use terraswap::asset::{Asset, AssetInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub anchor_money_market_address: String,
    pub aust_address: String,
    pub profit_check_address: String,
    pub treasury_addr: String,
    pub asset_info: AssetInfo,
    pub token_code_id: u64,
    pub treasury_fee: Decimal,
    pub flash_loan_fee: Decimal,
    pub commission_fee: Decimal,
    pub stable_cap: Uint128,
    pub vault_lp_token_name: Option<String>,
    pub vault_lp_token_symbol: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Receive hook for the liquidity token
    Receive(Cw20ReceiveMsg),
    /// Provide liquidity to the vault
    ProvideLiquidity { asset: Asset },
    /// Set minimum amount of stables held liquid (not deposited into anchor)
    SetStableCap { stable_cap: Uint128 },
    /// Sets the withdraw fee and flash loan fee
    SetFee {
        flash_loan_fee: Option<Fee>,
        treasury_fee: Option<Fee>,
        commission_fee: Option<Fee>,
    },
    /// Forwards the profit amount to the vault, is called by profit-check
    SendTreasuryCommission { profit: Uint128 },
    /// Set the admin of the contract
    SetAdmin { admin: String },
    /// Add provided contract to the whitelisted contracts
    AddToWhitelist { contract_addr: String },
    /// Remove provided contract from the whitelisted contracts
    RemoveFromWhitelist { contract_addr: String },
    /// Update the interal State struct
    UpdateState {
        anchor_money_market_address: Option<String>,
        aust_address: Option<String>,
        profit_check_address: Option<String>,
        allow_non_whitelisted: Option<bool>,
    },
    /// Execute a flashloan
    FlashLoan { payload: FlashLoanPayload },
    /// Internal callback message
    Callback(CallbackMsg),
}

/// MigrateMsg allows a privileged contract administrator to run
/// a migration on the contract. In this case it is just migrating
/// from one terra code to the same code, but taking advantage of the
/// migration step to set a new validator.
///
/// Note that the contract doesn't enforce permissions here, this is done
/// by blockchain logic (in the future by blockchain governance)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

// Modified from
// https://github.com/CosmWasm/cosmwasm-plus/blob/v0.2.3/packages/cw20/src/receiver.rs#L15
impl CallbackMsg {
    pub fn to_cosmos_msg<T: Clone + fmt::Debug + PartialEq + JsonSchema>(
        &self,
        contract_addr: &Addr,
    ) -> StdResult<CosmosMsg<T>> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from(contract_addr),
            msg: to_binary(&ExecuteMsg::Callback(self.clone()))?,
            funds: vec![],
        }))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallbackMsg {
    AfterSuccessfulLoanCallback {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolResponse {
    pub assets: [Asset; 2],
    pub total_value_in_ust: Uint128,
    pub total_share: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FlashLoanPayload {
    pub requested_asset: Asset,
    pub callback: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VaultQueryMsg {
    PoolConfig {},
    PoolState {},
    State {},
    Fees {},
    EstimateWithdrawFee { amount: Uint128 },
    VaultValue {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositResponse {
    pub deposit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValueResponse {
    pub total_ust_value: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeeResponse {
    pub fees: VaultFee,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EstimateDepositFeeResponse {
    pub fee: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EstimateWithdrawFeeResponse {
    pub fee: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub anchor_money_market_address: String,
    pub aust_address: String,
    pub profit_check_address: String,
    pub allow_non_whitelisted: bool,
}
