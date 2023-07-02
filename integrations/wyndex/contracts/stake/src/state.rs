use cosmwasm_schema::cw_serde;
use serde::{Deserialize, Serialize};
use wynd_curve_utils::Curve;

use crate::{utils::calc_power, ContractError};
use cosmwasm_std::{Addr, Decimal, Env, OverflowError, StdResult, Storage, Timestamp, Uint128};
use cw_controllers::{Admin, Claims};
use cw_storage_plus::{Item, Map};
use wyndex::asset::AssetInfoValidated;
use wyndex::stake::UnbondingPeriod;

pub const CLAIMS: Claims = Claims::new("claims");

#[cw_serde]
pub struct Config {
    /// address of cw20 contract token to stake
    pub cw20_contract: Addr,
    /// address that instantiated the contract
    pub instantiator: Addr,
    pub tokens_per_power: Uint128,
    pub min_bond: Uint128,
    /// configured unbonding periods in seconds
    pub unbonding_periods: Vec<UnbondingPeriod>,
    /// the maximum number of distributions that can be created
    pub max_distributions: u32,
    /// Address of the account that can call [`ExecuteMsg::QuickUnbond`]
    pub unbonder: Option<Addr>,
    /// Configuration for the [`crate::msg::ExecuteMsg::MigrateStake`] message.
    /// Allows converting staked LP tokens to LP tokens of another pool.
    /// E.g. LP tokens of the USDC-JUNO pool can be converted to LP tokens of the USDC-wyJUNO pool
    pub converter: Option<ConverterConfig>,
}

#[cw_serde]
pub struct ConverterConfig {
    /// Address of the contract that converts the LP tokens
    pub contract: Addr,
    /// Address of the pair contract the converter should convert to
    pub pair_to: Addr,
}

#[cw_serde]
#[derive(Default)]
pub struct BondingInfo {
    /// the amount of staked tokens which are not locked
    stake: Uint128,
    /// Vec of locked_tokens sorted by expiry timestamp
    locked_tokens: Vec<(Timestamp, Uint128)>,
}

impl BondingInfo {
    /// Add an amount of tokens to the stake
    pub fn add_unlocked_tokens(&mut self, amount: Uint128) -> Uint128 {
        let tokens = self.stake.checked_add(amount).unwrap();

        self.stake = tokens;

        tokens
    }

    /// Inserts a new locked_tokens entry in its correct place with a provided expires Timestamp and an amount
    pub fn add_locked_tokens(&mut self, expires: Timestamp, amount: Uint128) {
        // Insert the new locked_tokens entry into its correct place using a binary search and an insert
        match self.locked_tokens.binary_search(&(expires, amount)) {
            Ok(pos) => self.locked_tokens[pos].1 += amount,
            Err(pos) => self.locked_tokens.insert(pos, (expires, amount)),
        }
    }

    /// Free any tokens which are now considered unlocked
    /// Split locked tokens based on which are expired and assign the remaining ones to locked_tokens
    /// For each unlocked one, add this amount to the stake
    pub fn free_unlocked_tokens(&mut self, env: &Env) {
        if self.locked_tokens.is_empty() {
            return;
        }
        let (unlocked, remaining): (Vec<_>, Vec<_>) = self
            .locked_tokens
            .iter()
            .partition(|(time, _)| time <= &env.block.time);
        self.locked_tokens = remaining;

        self.stake += unlocked.into_iter().map(|(_, v)| v).sum::<Uint128>();
    }

    /// Attempt to release an amount of stake. First releasing any already unlocked tokens
    /// and then subtracting the requested amount from stake.
    /// On success, returns total_unlocked() after reducing the stake by this amount.
    pub fn release_stake(&mut self, env: &Env, amount: Uint128) -> Result<Uint128, OverflowError> {
        self.free_unlocked_tokens(env);

        let new_stake = self.stake.checked_sub(amount)?;

        self.stake = new_stake;

        Ok(self.stake)
    }

    /// Releases all locked stake, regardless of its bonding time
    /// On success, returns the unlocked stake (which is also the total stake)
    pub fn force_unlock_all(&mut self) -> Result<Uint128, OverflowError> {
        let locked: Uint128 = self.locked_tokens.iter().map(|(_, amount)| amount).sum();
        self.stake = self.stake.checked_add(locked)?;
        self.locked_tokens = vec![];
        Ok(self.stake)
    }

    /// Return all locked tokens at a given block time that is all
    /// locked_tokens with a Timestamp > the block time passed in env as a param
    pub fn total_locked(&self, env: &Env) -> Uint128 {
        let locked_stake = self
            .locked_tokens
            .iter()
            .filter_map(|(t, v)| if t > &env.block.time { Some(v) } else { None })
            .sum::<Uint128>();
        locked_stake
    }

    /// Return all locked tokens at a given block time that is all
    /// locked_tokens with a Timestamp > the block time passed in env as a param
    pub fn total_unlocked(&self, env: &Env) -> Uint128 {
        let mut unlocked_stake: Uint128 = self.stake;
        unlocked_stake += self
            .locked_tokens
            .iter()
            .filter_map(|(t, v)| if t <= &env.block.time { Some(v) } else { None })
            .sum::<Uint128>();

        unlocked_stake
    }

    /// Return all stake for this BondingInfo, including locked_tokens
    pub fn total_stake(&self) -> Uint128 {
        let total_stake: Uint128 = self
            .stake
            .checked_add(self.locked_tokens.iter().map(|x| x.1).sum())
            .unwrap();
        total_stake
    }
}

pub const REWARD_CURVE: Map<&AssetInfoValidated, Curve> = Map::new("reward_curve");

pub const ADMIN: Admin = Admin::new("admin");
pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Default, Serialize, Deserialize)]
pub struct TokenInfo {
    // how many tokens are fully bonded
    pub staked: Uint128,
    // how many tokens are unbounded and awaiting claim
    pub unbonding: Uint128,
}

impl TokenInfo {
    pub fn total(&self) -> Uint128 {
        self.staked + self.unbonding
    }
}

pub const TOTAL_STAKED: Item<TokenInfo> = Item::new("total_staked");

pub const STAKE: Map<(&Addr, UnbondingPeriod), BondingInfo> = Map::new("stake");

#[derive(Default, Serialize, Deserialize)]
pub struct TotalStake {
    /// Total stake
    pub staked: Uint128,
    /// Total stake minus any stake that is below min_bond by unbonding period.
    /// This is used when calculating the total staking power because we don't
    /// want to count stakes below min_bond into the total.
    pub powered_stake: Uint128,
}
/// Total stake minus any stake that is below min_bond by unbonding period.
/// This is used when calculating the total staking power because we don't
/// want to count stakes below min_bond into the total.
///
/// Using an item here to save some gas.
pub const TOTAL_PER_PERIOD: Item<Vec<(UnbondingPeriod, TotalStake)>> =
    Item::new("total_per_period");

/// Loads the total powered stake of the given period.
/// See [`TOTAL_PER_PERIOD`] for more details.
pub fn load_total_of_period(
    storage: &dyn Storage,
    unbonding_period: UnbondingPeriod,
) -> Result<TotalStake, ContractError> {
    let mut totals = TOTAL_PER_PERIOD.load(storage)?;
    totals
        .binary_search_by_key(&unbonding_period, |(period, _)| *period)
        .map_err(|_| ContractError::NoUnbondingPeriodFound(unbonding_period))
        .map(|idx| totals.swap_remove(idx).1)
}

/**** For distribution logic *****/

/// How much points is the worth of single token in rewards distribution.
/// The scaling is performed to have better precision of fixed point division.
/// This value is not actually the scaling itself, but how much bits value should be shifted
/// (for way more efficient division).
///
/// 32, to have those 32 bits, but it reduces how much tokens may be handled by this contract
/// (it is now 96-bit integer instead of 128). In original ERC2222 it is handled by 256-bit
/// calculations, but I256 is missing and it is required for this.
pub const SHARES_SHIFT: u8 = 32;

#[cw_serde]
pub struct Distribution {
    /// How many shares is single point worth
    pub shares_per_point: Uint128,
    /// Shares which were not fully distributed on previous distributions, and should be redistributed
    pub shares_leftover: u64,
    /// Total rewards distributed by this contract.
    pub distributed_total: Uint128,
    /// Total rewards not yet withdrawn.
    pub withdrawable_total: Uint128,
    /// The manager of this distribution
    pub manager: Addr,
    /// Rewards multiplier by unbonding period for this distribution
    pub reward_multipliers: Vec<(UnbondingPeriod, Decimal)>,
}

impl Distribution {
    /// Returns the rewards multiplier for a given unbonding period
    pub fn rewards_multiplier(
        &self,
        unbonding_period: UnbondingPeriod,
    ) -> Result<Decimal, ContractError> {
        self.reward_multipliers
            .binary_search_by_key(&unbonding_period, |(period, _)| *period)
            .map(|idx| self.reward_multipliers[idx].1) // map to multiplier
            .map_err(|_| ContractError::NoUnbondingPeriodFound(unbonding_period))
    }

    pub fn total_rewards_power_of_period(
        &self,
        storage: &dyn Storage,
        cfg: &Config,
        period: UnbondingPeriod,
    ) -> Result<Uint128, ContractError> {
        let totals = TOTAL_PER_PERIOD.load(storage).unwrap_or_default();
        let total = totals
            .binary_search_by_key(&period, |(period, _)| *period)
            .map(|idx| totals[idx].1.powered_stake) // map to powered stake
            .map_err(|_| ContractError::NoUnbondingPeriodFound(period))?;
        Ok(calc_power(cfg, total, self.rewards_multiplier(period)?))
    }

    /// Returns the total rewards power within this distribution.
    pub fn total_rewards_power(&self, storage: &dyn Storage, cfg: &Config) -> Uint128 {
        let totals = TOTAL_PER_PERIOD.load(storage).unwrap_or_default();
        self.reward_multipliers
            .iter()
            .zip(totals.into_iter())
            .map(
                |(&(unbonding_period, multiplier), (unbonding_period2, total_stake))| {
                    // sanity check
                    debug_assert_eq!(
                        unbonding_period, unbonding_period2,
                        "Unbonding period mismatch"
                    );
                    calc_power(cfg, total_stake.powered_stake, multiplier)
                },
            )
            .sum::<Uint128>()
    }

    pub fn calc_rewards_power(
        &self,
        storage: &dyn Storage,
        cfg: &Config,
        staker: &Addr,
    ) -> StdResult<Uint128> {
        // get rewards for all unbonding periods
        let mut power = Uint128::zero();
        for &(unbonding_period, multiplier) in self.reward_multipliers.iter() {
            let bonding_info = STAKE
                .may_load(storage, (staker, unbonding_period))?
                .unwrap_or_default();
            power += calc_power(cfg, bonding_info.total_stake(), multiplier);
        }
        Ok(power)
    }
}

#[cw_serde]
#[derive(Default)]
pub struct WithdrawAdjustment {
    /// How much points should be added/removed from calculated funds while withdrawal.
    pub shares_correction: i128,
    /// How much funds addresses already withdrawn.
    pub withdrawn_rewards: Uint128,
}

/// Rewards distribution data
pub const DISTRIBUTION: Map<&AssetInfoValidated, Distribution> = Map::new("distribution");
/// Information how to exactly adjust rewards while withdrawal.
/// This is per user, so it applies to all distributions.
pub const WITHDRAW_ADJUSTMENT: Map<(&Addr, &AssetInfoValidated), WithdrawAdjustment> =
    Map::new("withdraw_adjustment");

/// User delegated for funds withdrawal
pub const DELEGATED: Map<&Addr, Addr> = Map::new("delegated");

/// Flag to allow fast unbonding in emergency cases.
pub const UNBOND_ALL: Item<bool> = Item::new("unbond_all");

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::OverflowOperation;

    #[test]
    fn test_bonding_info_add() {
        let mut info = BondingInfo::default();
        let env = mock_env();

        info.stake = info.add_unlocked_tokens(Uint128::new(1000u128));

        assert_eq!(info.total_unlocked(&env), Uint128::new(1000u128));

        info.add_locked_tokens(env.block.time.plus_seconds(1000), Uint128::new(1000u128));
        assert_eq!(
            info.locked_tokens,
            [(env.block.time.plus_seconds(1000), Uint128::new(1000u128))]
        );
        assert_eq!(info.total_locked(&env), Uint128::new(1000u128))
    }
    #[test]
    fn test_bonding_info_add_then_release() {
        let mut info = BondingInfo::default();
        let env = mock_env();

        info.stake = info.add_unlocked_tokens(Uint128::new(1000u128));

        info.add_locked_tokens(env.block.time.plus_seconds(1000), Uint128::new(1000u128));
        // Trying to release both locked and unlocked tokens fails
        let err = info
            .release_stake(&env, Uint128::new(2000u128))
            .unwrap_err();
        assert_eq!(
            err,
            OverflowError {
                operation: OverflowOperation::Sub,
                operand1: "1000".to_string(),
                operand2: "2000".to_string()
            }
        );
        // But releasing the unlocked tokens passes
        info.release_stake(&env, Uint128::new(1000u128)).unwrap();
    }

    #[test]
    fn test_bonding_info_queries() {
        let mut info = BondingInfo::default();
        let env = mock_env();

        info.stake = info.add_unlocked_tokens(Uint128::new(1000u128));
        info.add_locked_tokens(env.block.time.plus_seconds(10), Uint128::new(1000u128));

        info.stake = info.add_unlocked_tokens(Uint128::new(500u128));
        info.add_locked_tokens(env.block.time.plus_seconds(20), Uint128::new(500u128));

        info.stake = info.add_unlocked_tokens(Uint128::new(100u128));
        info.add_locked_tokens(env.block.time.plus_seconds(30), Uint128::new(100u128));

        assert_eq!(info.total_locked(&env), Uint128::new(1600u128));
        assert_eq!(info.total_unlocked(&env), Uint128::new(1600u128));
        assert_eq!(info.total_stake(), Uint128::new(3200u128));
    }

    #[test]
    fn test_free_tokens() {
        let mut info = BondingInfo::default();
        let env = mock_env();

        info.stake = info.add_unlocked_tokens(Uint128::new(1000u128));

        assert_eq!(info.total_unlocked(&env), Uint128::new(1000u128));

        info.add_locked_tokens(env.block.time.minus_seconds(1000), Uint128::new(1000u128));
        assert_eq!(
            info.locked_tokens,
            [(env.block.time.minus_seconds(1000), Uint128::new(1000u128))]
        );

        info.add_locked_tokens(env.block.time.plus_seconds(1000), Uint128::new(1000u128));

        assert_eq!(info.total_unlocked(&env), Uint128::new(2000u128));
        assert_eq!(
            info.release_stake(&env, Uint128::new(1500u128)).unwrap(),
            Uint128::new(500u128)
        );
        assert_eq!(info.total_stake(), Uint128::new(1500));
        assert_eq!(info.total_locked(&env), Uint128::new(1000u128));
    }
}
