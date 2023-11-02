use crate::AVAILABLE_CHAINS;
use crate::OSMOSIS;
use abstract_sdk::core::objects::version_control::VersionControlContract;
use abstract_staking_standard::Identify;
use cosmwasm_std::Addr;

#[derive(Default)]
pub struct Osmosis {
    pub version_control_contract: Option<VersionControlContract>,
    pub local_proxy_addr: Option<Addr>,
    pub tokens: Vec<OsmosisTokenContext>,
}

pub struct OsmosisTokenContext {
    pub pool_id: u64,
    pub lp_token: String,
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
    use abstract_sdk::features::AbstractRegistryAccess;
    use abstract_sdk::{AbstractSdkError, AccountVerification};
    use abstract_staking_standard::msg::{
        Claim, RewardTokensResponse, StakeResponse, StakingInfo, StakingInfoResponse,
        UnbondingResponse,
    };
    use cw_utils::Expiration;
    use osmosis_std::types::osmosis::lockup::{LockupQuerier, MsgBeginUnlockingAll};
    use std::str::FromStr;

    use abstract_sdk::core::objects::ans_host::AnsHost;
    use abstract_sdk::core::objects::{AnsAsset, AnsEntryConvertor, AssetEntry, PoolReference};
    use osmosis_std::types::osmosis::poolmanager::v1beta1::PoolmanagerQuerier;

    use abstract_sdk::AbstractSdkResult;
    use abstract_staking_standard::{CwStakingCommand, CwStakingError};
    use cosmwasm_std::{
        Coin, CosmosMsg, Deps, MessageInfo, QuerierWrapper, StdError, StdResult, Uint128,
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
        pub fn query_pool_tokens_via_ans(
            &self,
            querier: &QuerierWrapper,
            ans_host: &AnsHost,
            staking_assets: Vec<AssetEntry>,
        ) -> AbstractSdkResult<Vec<OsmosisTokenContext>> {
            staking_assets
                .into_iter()
                .map(|s_asset| {
                    let dex_pair =
                        AnsEntryConvertor::new(AnsEntryConvertor::new(s_asset.clone()).lp_token()?)
                            .dex_asset_pairing()?;

                    let pool_ref = ans_host.query_asset_pairing(querier, &dex_pair)?;
                    // Currently takes the first pool found, but should be changed to take the best pool
                    let found: &PoolReference = pool_ref.first().ok_or(StdError::generic_err(
                        format!("No pool found for asset pairing {:?}", dex_pair),
                    ))?;

                    let pool_id = found.pool_address.expect_id()?;
                    let lp_token = format!("gamm/pool/{pool_id}");
                    Ok(OsmosisTokenContext { pool_id, lp_token })
                })
                .collect()
        }
    }

    impl OsmosisTokenContext {
        pub fn query_pool_data(&self, querier: &QuerierWrapper) -> StdResult<Pool> {
            let querier = PoolmanagerQuerier::new(querier);

            let res = querier.pool(self.pool_id)?;
            let pool = Pool::try_from(res.pool.unwrap()).unwrap();

            Ok(pool)
        }
    }

    impl AbstractRegistryAccess for Osmosis {
        fn abstract_registry(
            &self,
            _: cosmwasm_std::Deps<'_>,
        ) -> std::result::Result<VersionControlContract, abstract_sdk::AbstractSdkError> {
            self.version_control_contract
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
            version_control_contract: &VersionControlContract,
            staking_assets: Vec<AssetEntry>,
        ) -> abstract_sdk::AbstractSdkResult<()> {
            self.version_control_contract = Some(version_control_contract.clone());
            let account_registry = self.account_registry(deps);

            let base = info
                .map(|i| account_registry.assert_manager(&i.sender))
                .transpose()?;
            self.local_proxy_addr = base.map(|b| b.proxy);

            self.tokens =
                self.query_pool_tokens_via_ans(&deps.querier, ans_host, staking_assets)?;

            Ok(())
        }

        fn stake(
            &self,
            _deps: Deps,
            stake_request: Vec<AnsAsset>,
            unbonding_period: Option<cw_utils::Duration>,
        ) -> Result<Vec<cosmwasm_std::CosmosMsg>, CwStakingError> {
            let lock_coins: Vec<_> = stake_request
                .into_iter()
                .zip(self.tokens.iter())
                .map(|(stake, token)| {
                    Coin {
                        amount: stake.amount,
                        denom: token.lp_token.clone(),
                    }
                    .into()
                })
                .collect();
            let lock_tokens_msg = MsgLockTokens {
                owner: self.local_proxy_addr.as_ref().unwrap().to_string(),
                duration: to_osmo_duration(unbonding_period)?,
                coins: lock_coins,
            };

            Ok(vec![lock_tokens_msg.into()])
        }

        fn unstake(
            &self,
            _deps: Deps,
            unstake_request: Vec<AnsAsset>,
            _unbonding_period: Option<cw_utils::Duration>,
        ) -> Result<Vec<CosmosMsg>, CwStakingError> {
            let unstake_msgs: Vec<_> = unstake_request
                .into_iter()
                .zip(self.tokens.iter())
                .map(|(unstake, token)| {
                    MsgBeginUnlocking {
                        owner: self.local_proxy_addr.as_ref().unwrap().to_string(),
                        id: token.pool_id,
                        coins: vec![Coin {
                            denom: token.lp_token.clone(),
                            amount: unstake.amount,
                        }
                        .into()],
                    }
                    .into()
                })
                .collect();

            Ok(unstake_msgs)
        }

        fn claim(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError> {
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
            Err(CwStakingError::NotImplemented(
                "osmosis does not support claiming rewards".to_owned(),
            ))
        }

        // For osmosis, we don't have a staking token or a staking contract, everything happens at the sdk level
        // TODO
        fn query_info(
            &self,
            _querier: &cosmwasm_std::QuerierWrapper,
        ) -> Result<StakingInfoResponse, CwStakingError> {
            let infos = self
                .tokens
                .iter()
                .map(|t| StakingInfo {
                    staking_token: AssetInfoBase::Native(t.lp_token.clone()),
                    staking_target: t.pool_id.into(),
                    unbonding_periods: Some(vec![]),
                    max_claims: None,
                })
                .collect();

            Ok(StakingInfoResponse { infos })
        }

        fn query_staked(
            &self,
            querier: &QuerierWrapper,
            staker: Addr,
            _stakes: Vec<AssetEntry>,
            _unbonding_period: Option<cw_utils::Duration>,
        ) -> Result<StakeResponse, CwStakingError> {
            let lockup_request = LockupQuerier::new(querier);
            let locked_up = lockup_request.account_locked_coins(staker.to_string())?;

            let amounts = self
                .tokens
                .iter()
                .map(|token| {
                    locked_up
                        .coins
                        .iter()
                        .find(|&c| c.denom == token.lp_token)
                        .map(|c| Uint128::from_str(&c.amount).unwrap())
                        .unwrap_or_default()
                })
                .collect();

            Ok(StakeResponse { amounts })
        }

        fn query_unbonding(
            &self,
            querier: &QuerierWrapper,
            staker: Addr,
        ) -> Result<UnbondingResponse, CwStakingError> {
            let lockup_request = LockupQuerier::new(querier);
            let unlock_coins = lockup_request
                .account_unlocking_coins(staker.to_string())?
                .coins;
            let claims = self
                .tokens
                .iter()
                .map(|token| {
                    unlock_coins
                        .iter()
                        .find(|&c| c.denom == token.lp_token)
                        .map(|c| {
                            vec![Claim {
                                amount: Uint128::from_str(&c.amount).unwrap(),
                                claimable_at: Expiration::Never {},
                            }]
                        })
                        .unwrap_or_default()
                })
                .collect();

            Ok(UnbondingResponse { claims })
        }

        fn query_rewards(
            &self,
            _querier: &QuerierWrapper,
        ) -> Result<RewardTokensResponse, CwStakingError> {
            Err(CwStakingError::NotImplemented("osmosis".to_owned()))
        }
    }
}
