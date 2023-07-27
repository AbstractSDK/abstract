use crate::AVAILABLE_CHAINS;
use crate::OSMOSIS;
use abstract_staking_adapter_traits::Identify;
use cosmwasm_std::Addr;

#[derive(Default)]
pub struct Osmosis {
    pub abstract_registry: Option<Addr>,
    pub local_proxy_addr: Option<Addr>,
    pub pool_id: Option<u64>,
    pub lp_token: Option<String>,
}

impl Identify for Osmosis {
    fn name(&self) -> &'static str {
        OSMOSIS
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
}

#[cfg(feature = "full_integration")]
pub mod fns {
    use abstract_core::VERSION_CONTROL;
    use abstract_sdk::features::{AbstractNameService, AbstractRegistryAccess};
    use abstract_sdk::{AbstractSdkError, AccountVerification};
    use abstract_staking_adapter_traits::msg::{
        Claim, RewardTokensResponse, StakeResponse, StakingInfoResponse, UnbondingResponse,
    };
    use cw_utils::Expiration;
    use osmosis_std::types::osmosis::lockup::{
        LockupQuerier, ModuleLockedAmountRequest, ModuleLockedAmountResponse, MsgBeginUnlockingAll,
    };
    use std::cmp::min;
    use std::str::FromStr;

    use abstract_core::objects::ans_host::AnsHost;
    use abstract_core::objects::{AnsEntryConvertor, AssetEntry};
    use abstract_core::objects::{ContractEntry, PoolReference, UncheckedContractEntry};
    use osmosis_std::types::osmosis::poolmanager::v1beta1::PoolmanagerQuerier;

    use abstract_sdk::AbstractSdkResult;
    use abstract_staking_adapter_traits::{CwStakingCommand, CwStakingError};
    use cosmwasm_std::{
        to_binary, Coin, CosmosMsg, Deps, MessageInfo, QuerierWrapper, StdError, StdResult, Uint128,
    };
    use cw_asset::AssetInfoBase;

    use super::*;
    // const FORTEEN_DAYS: i64 = 60 * 60 * 24 * 14;
    use cosmwasm_std::Env;
    use osmosis_std::{
        shim::Duration,
        types::osmosis::gamm::v1beta1::Pool,
        types::{osmosis::lockup::MsgBeginUnlocking, osmosis::lockup::MsgLockTokens},
    };

    fn to_osmo_duration(dur: Option<cw_utils::Duration>) -> Result<Option<Duration>, StdError> {
        if let Some(dur) = dur {
            match dur {
                cw_utils::Duration::Time(sec) => Ok(Some(Duration {
                    seconds: sec.try_into().unwrap(),
                    nanos: 0,
                })),
                _ => Err(StdError::generic_err("Wrong duration, only time accepted")).unwrap(),
            }
        } else {
            Ok(None)
        }
    }

    impl Osmosis {
        /// Take the staking asset and query the pool id via the ans host
        pub fn query_pool_id_via_ans(
            &self,
            querier: &QuerierWrapper,
            ans_host: &AnsHost,
            staking_asset: AssetEntry,
        ) -> AbstractSdkResult<u64> {
            let dex_pair =
                AnsEntryConvertor::new(AnsEntryConvertor::new(staking_asset).lp_token()?)
                    .dex_asset_pairing()?;

            let mut pool_ref = ans_host.query_asset_pairing(querier, &dex_pair)?;
            // Currently takes the first pool found, but should be changed to take the best pool
            let found: &PoolReference = pool_ref.first().ok_or(StdError::generic_err(format!(
                "No pool found for asset pairing {:?}",
                dex_pair
            )))?;

            Ok(found.pool_address.expect_id()?)
        }

        pub fn query_pool_data(&self, querier: &QuerierWrapper) -> StdResult<Pool> {
            let querier = PoolmanagerQuerier::new(querier);

            let res = querier.pool(self.pool_id.unwrap())?;
            let pool = Pool::try_from(res.pool.unwrap()).unwrap();

            Ok(pool)
        }
    }

    impl AbstractRegistryAccess for Osmosis {
        fn abstract_registry(
            &self,
            _: cosmwasm_std::Deps<'_>,
        ) -> std::result::Result<cosmwasm_std::Addr, abstract_sdk::AbstractSdkError> {
            self.abstract_registry
                .clone()
                .ok_or(AbstractSdkError::generic_err(
                    "version_control address is not set",
                ))
            // We need to get to the version control somehow (possible from Ans Host ?)
        }
    }

    /// Osmosis app-chain dex implementation
    impl CwStakingCommand for Osmosis {
        fn fetch_data(
            &mut self,
            deps: cosmwasm_std::Deps,
            _env: Env,
            info: Option<MessageInfo>,
            ans_host: &AnsHost,
            abstract_registry: Addr,
            staking_asset: AssetEntry,
        ) -> abstract_sdk::AbstractSdkResult<()> {
            self.abstract_registry = Some(abstract_registry);
            let account_registry = self.account_registry(deps);

            let base = info
                .map(|i| account_registry.assert_manager(&i.sender))
                .transpose()?;
            self.local_proxy_addr = base.map(|b| b.proxy);

            let pool_id = self.query_pool_id_via_ans(&deps.querier, ans_host, staking_asset)?;

            self.pool_id = Some(pool_id);
            self.lp_token = Some(format!("gamm/pool/{}", self.pool_id.unwrap()));

            Ok(())
        }

        fn stake(
            &self,
            _deps: Deps,
            amount: Uint128,
            unbonding_period: Option<cw_utils::Duration>,
        ) -> Result<Vec<cosmwasm_std::CosmosMsg>, CwStakingError> {
            let lock_tokens_msg = MsgLockTokens {
                owner: self.local_proxy_addr.as_ref().unwrap().to_string(),
                duration: to_osmo_duration(unbonding_period)?,
                coins: vec![Coin {
                    amount,
                    denom: self.lp_token.clone().unwrap(),
                }
                .into()],
            };

            Ok(vec![lock_tokens_msg.into()])
        }

        fn unstake(
            &self,
            deps: Deps,
            amount: Uint128,
            _unbonding_period: Option<cw_utils::Duration>,
        ) -> Result<Vec<CosmosMsg>, CwStakingError> {
            let msg = MsgBeginUnlocking {
                owner: self.local_proxy_addr.as_ref().unwrap().to_string(),
                id: self.pool_id.unwrap(),
                coins: vec![Coin {
                    denom: self.lp_token.clone().unwrap(),
                    amount,
                }
                .into()],
            };
            Ok(vec![msg.into()])
        }

        fn claim(&self, deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError> {
            // Withdraw all
            let msg = MsgBeginUnlockingAll {
                owner: self.local_proxy_addr.as_ref().unwrap().to_string(),
            };
            Ok(vec![msg.into()])
        }

        // TODO, not sure this is needed in that case
        fn claim_rewards(
            &self,
            _deps: Deps,
        ) -> Result<Vec<cosmwasm_std::CosmosMsg>, CwStakingError> {
            Err(CwStakingError::NotImplemented("osmosis".to_owned()))
        }

        // For osmosis, we don't have a staking token or a staking contract, everything happens at the sdk level
        // TODO
        fn query_info(
            &self,
            _querier: &cosmwasm_std::QuerierWrapper,
        ) -> Result<StakingInfoResponse, CwStakingError> {
            let res = StakingInfoResponse {
                staking_token: AssetInfoBase::Native(self.lp_token.clone().unwrap()),
                staking_target: self.pool_id.clone().unwrap().into(),
                unbonding_periods: Some(vec![]),
                max_claims: None,
            };

            Ok(res)
        }

        fn query_staked(
            &self,
            querier: &QuerierWrapper,
            staker: Addr,
            _unbonding_period: Option<cw_utils::Duration>,
        ) -> Result<StakeResponse, CwStakingError> {
            Err(CwStakingError::NotImplemented("osmosis".to_owned()))
            // TODO: whitelist for contracts
            // let lockup_request = LockupQuerier::new(querier);
            // let locked_up = lockup_request.account_locked_coins(staker.to_string())?;

            // let amount = locked_up
            //     .coins
            //     .into_iter()
            //     .filter(|coin| coin.denom == self.lp_token.clone().unwrap())
            //     .map(|lock| Uint128::from_str(&lock.amount).unwrap())
            //     .sum();

            // Ok(StakeResponse { amount })
        }

        fn query_unbonding(
            &self,
            querier: &QuerierWrapper,
            staker: Addr,
        ) -> Result<UnbondingResponse, CwStakingError> {
            let lockup_request = LockupQuerier::new(querier);
            let unlocking = lockup_request
                .account_unlocking_coins(staker.to_string())?
                .coins
                .into_iter()
                .filter(|coin| coin.denom == self.lp_token.clone().unwrap())
                .map(|lock| Claim {
                    amount: Uint128::from_str(&lock.amount).unwrap(),
                    claimable_at: Expiration::Never {},
                })
                .collect();

            Ok(UnbondingResponse { claims: unlocking })
        }

        fn query_rewards(
            &self,
            _querier: &QuerierWrapper,
        ) -> Result<RewardTokensResponse, CwStakingError> {
            Err(CwStakingError::NotImplemented("osmosis".to_owned()))
        }
    }
}
