use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;

use cosmwasm_std::{Addr, Decimal, Uint128};
use wyndex::asset::{AssetInfo, AssetInfoValidated, AssetValidated};

use wyndex::stake::{ConverterConfig, FundingInfo, UnbondingPeriod};

#[cw_serde]
pub enum ExecuteMsg {
    /// Rebond will update an amount of bonded tokens from one bond period to the other
    Rebond {
        tokens: Uint128,
        // these must be valid time periods
        bond_from: u64,
        bond_to: u64,
    },
    /// Unbond will start the unbonding process for the given number of tokens.
    /// The sender immediately loses power from these tokens, and can claim them
    /// back to his wallet after `unbonding_period`
    Unbond {
        tokens: Uint128,
        /// As each unbonding period in delegation corresponds to particular voting
        /// multiplier, unbonding_period needs to be passed in unbond as well
        unbonding_period: u64,
    },
    /// Will immediately unbond all tokens for the given addresses.
    /// Can only be called by the `unbonder` account.
    QuickUnbond {
        /// The addresses of the stakers that should be unbonded
        stakers: Vec<String>,
    },
    /// UnbondAll is used to allow instant unbond of tokens in emergency cases.
    /// Can only be called by the `unbonder` account.
    UnbondAll {},
    /// Allows to revert the unbond all flag to false.
    /// Can only be called by the `unbonder` account or the ADMIN.
    StopUnbondAll {},
    /// Claim is used to claim your native tokens that you previously "unbonded"
    /// after the contract-defined waiting period (eg. 1 week)
    Claim {},

    /// Change the admin
    UpdateAdmin { admin: Option<String> },
    /// Create a new distribution flow
    CreateDistributionFlow {
        /// The address of the manager that can change this distribution
        manager: String,

        /// The asset that will be distributed
        asset: AssetInfo,

        /// Rewards multiplier by unbonding period for this distribution
        /// Only periods that are defined in the contract can be used here
        rewards: Vec<(UnbondingPeriod, Decimal)>,
    },

    /// This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),

    /// Distributes rewards sent with this message, and all rewards transferred since last call of this
    /// to members, proportionally to their points. Rewards are not immediately send to members, but
    /// assigned to them for later withdrawal (see: `ExecuteMsg::WithdrawFunds`)
    DistributeRewards {
        /// Original source of rewards, informational. If present overwrites "sender" field on
        /// propagated event.
        sender: Option<String>,
    },
    /// Withdraws rewards which were previously distributed and assigned to sender.
    WithdrawRewards {
        /// Account from which assigned rewards would be withdrawn; `sender` by default. `sender` has
        /// to be eligible for withdrawal from `owner` address to perform this call (`owner` has to
        /// call `DelegateWithdrawal { delegated: sender }` before)
        owner: Option<String>,
        /// Address where to transfer funds. If not present, funds would be sent to `sender`.
        receiver: Option<String>,
    },
    /// Sets given address as allowed for senders funds withdrawal. Funds still can be withdrawn by
    /// sender himself, but this additional account is allowed to perform it as well. There can be only
    /// one account delegated for withdrawal for any owner at any single time.
    DelegateWithdrawal {
        /// Account delegated for withdrawal. To disallow current withdrawal, the best is to set it
        /// to own address.
        delegated: String,
    },
    /// Fund a distribution flow with 1 or more native tokens, updating each provided native token's reward config appropriately.
    /// Funds to be provided are included in `info.funds`
    FundDistribution { funding_info: FundingInfo },

    /// Moves the given amount of LP tokens staked to the given unbonding period from the sender's
    /// account to a different pool (by converting one or more of the pool tokens).
    MigrateStake {
        amount: Uint128,
        unbonding_period: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Claims shows the tokens in process of unbonding for this address
    #[returns(cw_controllers::ClaimsResponse)]
    Claims { address: String },
    /// Show the number of tokens currently staked by this address.
    #[returns(StakedResponse)]
    Staked {
        address: String,
        /// Unbonding period in seconds
        unbonding_period: u64,
    },
    /// Show the number of tokens currently staked by this address for all unbonding periods
    #[returns(AllStakedResponse)]
    AllStaked { address: String },
    /// Show the number of all, not unbonded tokens delegated by all users for all unbonding periods
    #[returns(TotalStakedResponse)]
    TotalStaked {},
    /// Show the number of all tokens being unbonded for all unbonding periods
    #[returns(TotalUnbondingResponse)]
    TotalUnbonding {},
    /// Show the total number of outstanding rewards
    #[returns(RewardsPowerResponse)]
    TotalRewardsPower {},
    /// Show the outstanding rewards for this address
    #[returns(RewardsPowerResponse)]
    RewardsPower { address: String },
    /// Return AdminResponse
    #[returns(cw_controllers::AdminResponse)]
    Admin {},
    #[returns(BondingInfoResponse)]
    BondingInfo {},

    /// Return how many rewards will be received per token in each unbonding period in one year
    #[returns(AnnualizedRewardsResponse)]
    AnnualizedRewards {},
    /// Return how many rewards are assigned for withdrawal from the given address. Returns
    /// `RewardsResponse`.
    #[returns(WithdrawableRewardsResponse)]
    WithdrawableRewards { owner: String },
    /// Return how many rewards were distributed in total by this contract. Returns
    /// `RewardsResponse`.
    #[returns(DistributedRewardsResponse)]
    DistributedRewards {},
    /// Return how many funds were sent to this contract since last `ExecuteMsg::DistributeFunds`,
    /// and await for distribution. Returns `RewardsResponse`.
    #[returns(UndistributedRewardsResponse)]
    UndistributedRewards {},
    /// Return address allowed for withdrawal of the funds assigned to owner. Returns `DelegatedResponse`
    #[returns(DelegatedResponse)]
    Delegated { owner: String },
    /// Returns rewards distribution data
    #[returns(DistributionDataResponse)]
    DistributionData {},
    /// Returns withdraw adjustment data
    #[returns(WithdrawAdjustmentDataResponse)]
    WithdrawAdjustmentData { addr: String, asset: AssetInfo },
    /// Returns the value of unbond all flag
    #[returns(UnbondAllResponse)]
    UnbondAll {},
}

#[cw_serde]
pub struct MigrateMsg {
    /// Address of the account that can call [`ExecuteMsg::QuickUnbond`], [`ExecuteMsg::UnbondAll`]
    /// and [`ExecuteMsg::StopUnbondAll`]
    pub unbonder: Option<String>,
    /// Allows adding a converter to the staking contract after instantiation.
    pub converter: Option<ConverterConfig>,
    /// Allows to directly set unbond all flag during migrations.
    pub unbond_all: bool,
}

#[cw_serde]
pub struct StakedResponse {
    pub stake: Uint128,
    pub total_locked: Uint128,
    pub unbonding_period: u64,
    pub cw20_contract: String,
}

#[cw_serde]
pub struct AllStakedResponse {
    pub stakes: Vec<StakedResponse>,
}

#[cw_serde]
pub struct TotalStakedResponse {
    pub total_staked: Uint128,
}

#[cw_serde]
pub struct TotalUnbondingResponse {
    pub total_unbonding: Uint128,
}

#[cw_serde]
pub struct RewardsPowerResponse {
    /// The rewards power of the address per asset
    /// This does not use `AssetValidated`, because the semantics are different.
    /// The `Uint128` is not an actual asset amount, but the address' rewards power for that asset.
    pub rewards: Vec<(AssetInfoValidated, Uint128)>,
}

#[cw_serde]
pub struct BondingPeriodInfo {
    pub unbonding_period: u64,
    pub total_staked: Uint128,
}

#[cw_serde]
pub struct BondingInfoResponse {
    pub bonding: Vec<BondingPeriodInfo>,
}

#[cw_serde]
pub struct AnnualizedRewardsResponse {
    /// The rewards per token for each unbonding period.
    pub rewards: Vec<(UnbondingPeriod, Vec<AnnualizedReward>)>,
}

#[cw_serde]
pub struct AnnualizedReward {
    pub info: AssetInfoValidated,
    /// The amount of tokens. the semantics of this are equivalent to [`AssetValidated`].
    /// This is a decimal value to reduce rounding when using it for further calculations.
    /// None means contract does not know the value - total_staked or total_power could be 0.
    pub amount: Option<Decimal>,
}

// just for the proper json outputs
#[cw_serde]
pub struct TokenContractResponse(Addr);

#[cw_serde]
pub struct WithdrawableRewardsResponse {
    /// Amount of rewards assigned for withdrawal from the given address.
    pub rewards: Vec<AssetValidated>,
}

#[cw_serde]
pub struct DelegatedResponse {
    pub delegated: Addr,
}

#[cw_serde]
pub struct DistributedRewardsResponse {
    /// Total number of tokens sent to the contract over all time.
    pub distributed: Vec<AssetValidated>,
    /// Total number of tokens available to be withdrawn.
    pub withdrawable: Vec<AssetValidated>,
}

pub type UndistributedRewardsResponse = WithdrawableRewardsResponse;
#[cw_serde]
pub struct DistributionDataResponse {
    pub distributions: Vec<(AssetInfoValidated, crate::state::Distribution)>,
}
pub type WithdrawAdjustmentDataResponse = crate::state::WithdrawAdjustment;

#[cw_serde]
pub struct UnbondAllResponse {
    /// Value of unbond all flag.
    pub unbond_all: bool,
}
