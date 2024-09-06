//! Minimalistic versions of astrovault types that was created to reduce wasm size
//! Currently execute/query messages not in use

use std::fmt;

use cosmwasm_std::{Decimal, Uint128};

pub enum AstrovaultPoolType {
    Standard,
    Stable { is_xasset: bool },
    Ratio,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
// Ignoring unknown fields
pub struct ConfigResponse {
    pub factory: String,
    // To determine if pool is xAsset
    pub pool_settings: Option<PoolSettings>,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct PoolSettings {
    pub xasset_mode_minter: Option<String>,
}

/// This is enum that includes all of the messages we use in astrovault cw20 hook
/// It's separated for minimizing the size of the wasm
#[cosmwasm_schema::cw_serde]
pub enum AstrovaultCw20HookMsg {
    WithdrawLiquidity {
        to: Option<String>,
    },
    WithdrawalXassetMode {
        to: Option<String>,
        expected_return: Option<Vec<Uint128>>,
    },
    #[serde(rename(serialize = "withdrawal_to_lockup"))]
    WithdrawalToLockupStable {
        withdrawal_lockup_assets_amount: Vec<Uint128>,
        to: Option<String>,
        is_instant_withdrawal: Option<bool>,
        expected_return: Option<Vec<Uint128>>,
    },
    #[serde(rename(serialize = "withdrawal_to_lockup"))]
    WithdrawalToLockupRatio {
        to: Option<String>,
        is_instant_withdrawal: Option<bool>,
        expected_return: Option<Vec<Uint128>>,
    },
}

/// This is enum that includes all of the messages we use in astrovault execution
/// It's separated for minimizing the size of the wasm
#[cosmwasm_schema::cw_serde]
pub enum AstrovaultDexExecuteMsg {
    #[serde(rename(serialize = "swap"))]
    SwapStandard {
        // cw20 hook don't have this field, need to skip this one
        #[serde(skip_serializing_if = "Option::is_none")]
        offer_asset: Option<AstrovaultAsset>,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        expected_return: Option<Uint128>,
        to: Option<String>,
    },
    #[serde(rename(serialize = "swap"))]
    SwapStable {
        swap_to_asset_index: u32,
        to: Option<String>,
        expected_return: Option<Uint128>,
    },
    #[serde(rename(serialize = "swap"))]
    SwapRatio {
        to: Option<String>,
        expected_return: Option<Uint128>,
    },
    ProvideLiquidity {
        assets: [AstrovaultAsset; 2],
        slippage_tolerance: Option<Decimal>,
        receiver: Option<String>,
        direct_staking: Option<cosmwasm_std::Empty>,
    },
    #[serde(rename(serialize = "deposit"))]
    DepositStable {
        assets_amount: Vec<Uint128>,
        receiver: Option<String>,
        direct_staking: Option<cosmwasm_std::Empty>,
    },
    #[serde(rename(serialize = "deposit"))]
    DepositRatio {
        assets_amount: [Uint128; 2],
        receiver: Option<String>,
        direct_staking: Option<cosmwasm_std::Empty>,
        expected_return: Option<Uint128>,
    },
}

/// This is enum that includes all of the messages we use in astrovault queries
/// It's separated for minimizing the size of the wasm
#[cosmwasm_schema::cw_serde]
pub enum AstrovaultDexQueryMsg {
    Config {},
    PoolInfo {},
    Pool {},
    Simulation {
        offer_asset: AstrovaultAsset,
    },
    #[serde(rename(serialize = "swap_simulation"))]
    SwapSimulationStable {
        amount: Uint128,
        swap_from_asset_index: u32,
        swap_to_asset_index: u32,
    },
    #[serde(rename(serialize = "swap_simulation"))]
    SwapSimulationRatio {
        amount: Uint128,
        swap_from_asset_index: u8,
    },
}

#[derive(cosmwasm_schema::serde::Serialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub enum AstrovaultStakingExecuteMsg {
    // User handle: Withdrawals previously deposited assets. Also can be used to claim rewards when the amount is set to "0"
    Withdrawal {
        amount: Option<Uint128>, // if None it withdrawal all balance
        direct_pool_withdrawal: Option<cosmwasm_std::Empty>, // if this has data, will withdrawal the LP tokens from this staking pool directly to the pool (standard or stable) associated and use them to withdrawal the assets from there
        to: Option<cosmwasm_std::Empty>, // flag that specifies the addrees to send the tokens to (rewards and/or lockedup tokens) (only relevant if NOT direct_pool_withdrawal)
        not_claim_rewards: Option<bool>, // flag if true not claim the rewards automatically
        withdrawal_unlocked: Option<cosmwasm_std::Empty>, // flag to also execute the WithdrawalFromLockup function (only relevant for pools that have lockup times)
        notify: Option<cosmwasm_std::Empty>, // flag that specifies a contract to be notified after tokens are sent there
    },
    // User handle: Withdrawals assets that were locked after using the Withdrawal Handle
    // only relevant for pools that have lockup times (config.lockup_duration)
    WithdrawalFromLockup {
        to: Option<cosmwasm_std::Empty>,
        direct_pool_withdrawal: Option<cosmwasm_std::Empty>,
        notify: Option<cosmwasm_std::Empty>, // flag that specifies a contract to be notified after tokens are sent there
    },
}

#[derive(cosmwasm_schema::serde::Serialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub enum AstrovaultStakingQueryMsg {
    Config {},
    Balance { address: String },
    RewardSources { reward_source: Option<String> },
}

#[derive(cosmwasm_schema::serde::Serialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub enum AstrovaultStakingReceiveMsg {
    Deposit {
        sender: Option<String>,
        not_claim_rewards: Option<bool>, // flag if true not claim the rewards automatically
        notify: Option<bool>,
    },
}

/// Response for [`AstrovaultQueryMsg::Pool`]
#[cosmwasm_schema::cw_serde]
pub struct PoolResponse {
    pub assets: Vec<AstrovaultAsset>,
    pub total_share: Uint128,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct RewardSourceResponse {
    pub info: RewardSource,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct RewardSource {
    pub reward_asset: AstrovaultAssetInfo,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct PoolInfo {
    pub asset_infos: Vec<AstrovaultAssetInfo>,
}

#[cosmwasm_schema::cw_serde]
pub enum AstrovaultAssetInfo {
    Token { contract_addr: String },
    NativeToken { denom: String },
}

impl fmt::Display for AstrovaultAssetInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NativeToken { denom } => write!(f, "{}", denom),
            Self::Token { contract_addr } => write!(f, "{}", contract_addr),
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct AstrovaultAsset {
    pub info: AstrovaultAssetInfo,
    pub amount: Uint128,
}

impl fmt::Display for AstrovaultAsset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.amount, self.info)
    }
}

/// SimulationResponse returns swap simulation response
#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct SimulationResponse {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
    pub commission_amount: Uint128,
    pub buybackburn_amount: Uint128,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct SwapCalcResponse {
    pub from_amount: Uint128,
    pub to_amount_without_fee: Uint128,
    pub to_amount_minus_fee: Uint128,
    pub fee_amount: Uint128,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct StablePoolQuerySwapSimulation {
    pub from_assets_amount: Vec<Uint128>,
    pub swap_to_assets_amount: Vec<Uint128>,
    pub mint_to_assets_amount: Vec<Uint128>,
    pub assets_fee_amount: Vec<Uint128>,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct PendingLockupWithdrawal {
    pub to_withdrawal_amount: Uint128,
    pub withdrawal_timestamp: u64,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct AssetStakingRewards {
    pub asset: AstrovaultAssetInfo,
    pub rewards: Uint128,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct LpBalanceResponse {
    pub locked: Uint128,
    pub boosted_amount: Uint128,
    pub pending_lockup_withdrawals: Vec<PendingLockupWithdrawal>,
    pub rewards: Vec<AssetStakingRewards>,
    pub height: u64,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct OwnerSettings {
    pub is_deposit_enabled: bool,
    pub is_withdrawal_enabled: bool,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct LpConfigResponse {
    pub owner: String,
    pub pool: Option<String>,
    pub owner_settings: OwnerSettings,
    pub lockup_duration: Option<u64>,
    pub inc_token: String,
}
