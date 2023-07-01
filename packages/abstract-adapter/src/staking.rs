use crate::AVAILABLE_CHAINS;
use crate::OSMOSIS;
use abstract_staking_adapter_traits::Identify;
use cosmwasm_std::Addr;

#[derive(Default)]
pub struct Osmosis {
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


#[cfg(feature="full_integration")]
pub mod fns {
    use std::str::FromStr;
    use abstract_sdk::AccountVerification;
    use abstract_sdk::features::AbstractRegistryAccess;
    use cw_utils::Expiration;
    use std::cmp::min;
    use abstract_staking_adapter_traits::msg::{RewardTokensResponse, Claim, UnbondingResponse, StakeResponse, StakingInfoResponse};
    use osmosis_std::types::osmosis::lockup::LockupQuerier;
        
    use osmosis_std::types::osmosis::poolmanager::v1beta1::PoolmanagerQuerier;
    use abstract_core::objects::ans_host::AnsHost;
    use abstract_core::objects::AssetEntry;

    use cw_asset::AssetInfoBase;
    use abstract_staking_adapter_traits::{CwStakingError, CwStakingCommand};
    use cosmwasm_std::{Uint128, Deps, StdError, StdResult, CosmosMsg,QuerierWrapper, Coin};

    use super::*;
    // const FORTEEN_DAYS: i64 = 60 * 60 * 24 * 14;
    use cosmwasm_std::Env;
    use osmosis_std::{
        shim::Duration,
        types::{
            osmosis::gamm::v1beta1::{Pool},
        },
        types::{osmosis::lockup::MsgBeginUnlocking, osmosis::lockup::MsgLockTokens},
    };

    fn to_osmo_duration(dur: Option<cw_utils::Duration>) -> Result<Option<Duration>, StdError>{
        if let Some(dur) = dur{
            match dur{
                cw_utils::Duration::Time(sec) => Ok(Some(Duration { seconds: sec.try_into().unwrap(), nanos: 0 })),
                _ => Err(StdError::generic_err("Wrong duration, only time accepted")).unwrap()
            }
        }else{
            Ok(None)
        }
    }

    impl Osmosis{
        pub fn query_pool_data(&self, querier: &QuerierWrapper) -> StdResult<Pool> {
            let querier = PoolmanagerQuerier::new(querier);

            let res = querier.pool(self.pool_id.unwrap())?;
            let pool = Pool::try_from(res.pool.unwrap()).unwrap();

            Ok(pool)
        }
    }

    impl AbstractRegistryAccess for Osmosis{

        fn abstract_registry(&self, _: cosmwasm_std::Deps<'_>) -> std::result::Result<cosmwasm_std::Addr, abstract_sdk::AbstractSdkError> { 

            panic!("Not implementable for now");
            // We need to get to the version control somehow (possible from Ans Host ?)
        }
    }

    /// Osmosis app-chain dex implementation
    impl CwStakingCommand for Osmosis {
        fn fetch_data(
            &mut self,
            deps: cosmwasm_std::Deps,
            _env: Env,
            ans_host: &AnsHost,
            staking_asset: AssetEntry,
        ) -> abstract_sdk::AbstractSdkResult<()> {

            let provider_staking_contract_entry = self.staking_entry(&staking_asset);

            let account_registry = self.account_registry(deps);
            // TODO, this will never work
            // We need a receiver address to make that work
            self.local_proxy_addr = Some(account_registry.assert_manager(&Addr::unchecked("manager address from calling info ?"))?.proxy);

            let pool_addr =
                ans_host.query_contract(&deps.querier, &provider_staking_contract_entry)?;

            self.pool_id = Some(pool_addr.to_string().parse().unwrap());
            self.lp_token = Some(format!("gamm/pool/{}", self.pool_id.unwrap()));
        
            Ok(())
        }

        fn stake(
            &self,
            _deps: Deps,
            amount: Uint128,
            unbonding_period: Option<cw_utils::Duration>,
        ) -> Result<Vec<cosmwasm_std::CosmosMsg>, CwStakingError> {

            let lock_tokens_msg = MsgLockTokens{ 
                owner: self.local_proxy_addr.as_ref().unwrap().to_string(), 
                duration: to_osmo_duration(unbonding_period)?,
                coins: vec![Coin{
                    amount,
                    denom: self.lp_token.clone().unwrap()
                }.into()]
            };

            Ok(vec![lock_tokens_msg.into()])
        }

        // We unstake all the amount from the pools that only have the coin inside it
        // TODO, this is not perfect, don't know how to do that better for now
        fn unstake(
            &self,
            deps: Deps,
            amount: Uint128,
            _unbonding_period: Option<cw_utils::Duration>,
        ) -> Result<Vec<CosmosMsg>, CwStakingError> {

            let lockup_request = LockupQuerier::new(&deps.querier);
            let locked_up = lockup_request.account_locked_past_time_not_unlocking_only(
                self.local_proxy_addr.as_ref().unwrap().to_string(),
                None
            )?;
            let lock_ids: Vec<_> =  locked_up.locks.into_iter()
                .filter(|lock| lock.coins.len() == 1 && lock.coins[0].denom == self.lp_token.clone().unwrap())
                .collect();

            let mut msgs = vec![];
            let mut remaining_amount = amount;
            for period_lock in lock_ids{
                let period_amount = Uint128::from_str(&period_lock.coins[0].amount).unwrap();
                let withdraw_amount = min(remaining_amount, period_amount);
                remaining_amount -= withdraw_amount;
                msgs.push(MsgBeginUnlocking {
                    owner: self.local_proxy_addr.as_ref().unwrap().to_string(),
                    coins: vec![Coin{
                        denom: self.lp_token.clone().unwrap(),
                        amount: withdraw_amount,
                    }.into()], // We withdraw all
                    id: period_lock.id,
                }.into());

                if remaining_amount.is_zero() {
                    break;
                }
            }

            Ok(msgs)
        }

        fn claim(&self, deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError> {
            
            let lockup_request = LockupQuerier::new(&deps.querier);
            let locked_up = lockup_request.account_unlocked_before_time(
                self.local_proxy_addr.as_ref().unwrap().to_string(),
                None
            )?;
            let lock_ids: Vec<u64> =  locked_up.locks.into_iter()
                .filter(|lock| lock.coins.len() == 1 && lock.coins[0].denom == self.lp_token.clone().unwrap())
                .map(|lock| lock.id)
                .collect();

            let msgs: Vec<CosmosMsg> = lock_ids.iter().map(|id| MsgBeginUnlocking {
                owner: self.local_proxy_addr.as_ref().unwrap().to_string(),
                coins: vec![], // We withdraw all
                id: *id,
            }
            .into()).collect();

            Ok(msgs)
        }

        // TODO, not sure this is needed in that case
        fn claim_rewards(&self, _deps: Deps) -> Result<Vec<cosmwasm_std::CosmosMsg>, CwStakingError> {
            Ok(vec![])
        }

        // For osmosis, we don't have a staking token or a staking contract, everything happens at the sdk level
        // TODO
        fn query_info(
            &self,
            _querier: &cosmwasm_std::QuerierWrapper,
        ) -> Result<StakingInfoResponse, CwStakingError> {
            
            let res = StakingInfoResponse {
                staking_token: AssetInfoBase::Native(self.lp_token.clone().unwrap()),
                staking_contract_address: Addr::unchecked(""),
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
            // We query all the locked tokens that correspond to the token in question
            let lockup_request = LockupQuerier::new(querier);
            let locked_up = lockup_request.account_locked_coins(
                staker.to_string(),
            )?.coins
                .into_iter()
                .filter(|coin| coin.denom == self.lp_token.clone().unwrap())
                .map(|lock| Uint128::from_str(&lock.amount).unwrap())
                .sum();

            Ok(StakeResponse {
                amount: locked_up
            })
        }

        fn query_unbonding(
            &self,
            querier: &QuerierWrapper,
            staker: Addr,
        ) -> Result<UnbondingResponse, CwStakingError> {
            let lockup_request = LockupQuerier::new(querier);
            let unlocking = lockup_request.account_unlocking_coins(
                staker.to_string(),
            )?.coins
                .into_iter()
                .filter(|coin| coin.denom == self.lp_token.clone().unwrap())
                .map(|lock| Claim{
                    amount: Uint128::from_str(&lock.amount).unwrap(),
                    claimable_at: Expiration::Never {  }
                })
                .collect();

            Ok(UnbondingResponse { claims: unlocking})
        }

        // TODO, not sure, how the rewards are being given out during the lockup period. Do users even have to claim rewards ?

        fn query_rewards(
            &self,
            _querier: &QuerierWrapper,
        ) -> Result<RewardTokensResponse, CwStakingError> {
            Ok(RewardTokensResponse{
                tokens: vec![]
            })
        }
    }
}
