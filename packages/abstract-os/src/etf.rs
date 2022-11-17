//! # Liquidity Interface Add-On
//!
//! `abstract_os::etf` is an app which allows users to deposit into or withdraw from a [`crate::proxy`] contract.
//!
//! ## Description  
//! This contract uses the proxy's value calculation configuration to get the value of the assets held in the proxy and the relative value of the deposit asset.
//! It then mints LP tokens that are claimable for an equal portion of the proxy assets at a later date.  
//!
//! ---
//! **WARNING:** This mint/burn mechanism can be mis-used by flash-loan attacks if the assets contained are of low-liquidity compared to the etf's size.
//!
//! ## Creation
//! The etf contract can be added on an OS by calling [`ExecuteMsg::CreateModule`](crate::manager::ExecuteMsg::CreateModule) on the manager of the os.
//! ```ignore
//! let etf_init_msg = InstantiateMsg{
//!                deposit_asset: "juno".to_string(),
//!                base: BaseInstantiateMsg{ans_host_address: "juno1...".to_string()},
//!                fee: Decimal::percent(10),
//!                provider_addr: "juno1...".to_string(),
//!                token_code_id: 3,
//!                etf_lp_token_name: Some("demo_etf".to_string()),
//!                etf_lp_token_symbol: Some("DEMO".to_string()),
//!        };
//! let create_module_msg = ExecuteMsg::CreateModule {
//!                 module: Module {
//!                     info: ModuleInfo {
//!                         name: ETF.into(),
//!                         version: None,
//!                     },
//!                     kind: crate::core::modules::ModuleKind::External,
//!                 },
//!                 init_msg: Some(to_binary(&etf_init_msg).unwrap()),
//!        };
//! // Call create_module_msg on manager
//! ```
//!
//! ## Migration
//! Migrating this contract is done by calling `ExecuteMsg::Upgrade` on [`crate::manager`] with `crate::ETF` as module.

pub mod state {
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use crate::objects::fee::Fee;
    use cosmwasm_std::Addr;
    use cw_storage_plus::Item;

    #[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
    /// State stores LP token address
    /// BaseState is initialized in contract
    pub struct State {
        pub liquidity_token_addr: Addr,
        pub provider_addr: Addr,
    }

    pub const STATE: Item<State> = Item::new("\u{0}{5}state");
    pub const FEE: Item<Fee> = Item::new("\u{0}{3}fee");
}

use cosmwasm_std::Decimal;
use cw_asset::AssetUnchecked;
/// Init msg
#[cosmwasm_schema::cw_serde]
pub struct EtfInstantiateMsg {
    /// Code-id used to create the LP token
    pub token_code_id: u64,
    /// Fee charged on withdrawal
    pub fee: Decimal,
    /// Address of the service provider which receives the fee.
    pub provider_addr: String,
    /// Name of the etf token
    pub token_name: Option<String>,
    /// Symbol of the etf token
    pub token_symbol: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub enum EtfExecuteMsg {
    /// Provide liquidity to the attached proxy using a native token.
    ProvideLiquidity { asset: AssetUnchecked },
    /// Set the withdraw fee
    SetFee { fee: Decimal },
}

#[cosmwasm_schema::cw_serde]
pub enum EtfQueryMsg {
    // Add dapp-specific queries here
    /// Returns [`StateResponse`]
    State {},
}

#[cosmwasm_schema::cw_serde]
pub enum DepositHookMsg {
    WithdrawLiquidity {},
    ProvideLiquidity {},
}

#[cosmwasm_schema::cw_serde]
pub struct StateResponse {
    pub liquidity_token: String,
    pub fee: Decimal,
}
