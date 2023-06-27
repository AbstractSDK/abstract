use abstract_sdk::core::objects::LpToken;
use abstract_staking_adapter_traits::Identify;
use cosmwasm_std::Addr;

pub const KUJIRA: &str = "kujira";
#[cfg(feature = "local")]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::core::registry::LOCAL_CHAIN;
#[cfg(not(feature = "local"))]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::core::registry::KUJIRA;

// TODO: use optional values here?
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "kujira"), allow(unused))]
pub struct Kujira {
    lp_token: LpToken,
    lp_token_denom: String,
    staking_contract_address: Addr,
}

impl Default for Kujira {
    fn default() -> Self {
        Self {
            lp_token: Default::default(),
            lp_token_denom: "".to_string(),
            staking_contract_address: Addr::unchecked(""),
        }
    }
}

impl Identify for Kujira {
    fn name(&self) -> &'static str {
        KUJIRA
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
}

#[cfg(feature = "kujira")]
use ::{
    abstract_sdk::{
        core::objects::{AnsEntryConvertor, AssetEntry},
        feature_objects::AnsHost,
        AbstractSdkResult,
    },
    abstract_staking_adapter_traits::msg::{
        RewardTokensResponse, StakeResponse, StakingInfoResponse, UnbondingResponse,
    },
    abstract_staking_adapter_traits::{CwStakingCommand, CwStakingError},
    cosmwasm_std::{wasm_execute, Coin, CosmosMsg, Deps, Env, QuerierWrapper, StdError, Uint128},
    cw_asset::AssetInfo,
    cw_utils::Duration,
    kujira::bow::staking as BowStaking,
};

#[cfg(feature = "kujira")]
impl CwStakingCommand for Kujira {
    fn fetch_data(
        &mut self,
        deps: Deps,
        _env: Env,
        ans_host: &AnsHost,
        lp_token: AssetEntry,
    ) -> AbstractSdkResult<()> {
        self.staking_contract_address = self.staking_contract_address(deps, ans_host, &lp_token)?;

        self.lp_token = AnsEntryConvertor::new(lp_token).lp_token()?;
        Ok(())
    }

    fn stake(
        &self,
        _deps: Deps,
        amount: Uint128,
        _unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let msg = BowStaking::ExecuteMsg::Stake { addr: None };
        Ok(vec![wasm_execute(
            self.staking_contract_address.clone(),
            &msg,
            vec![Coin {
                amount,
                denom: self.lp_token_denom.clone(),
            }],
        )?
        .into()])
    }

    fn unstake(
        &self,
        _deps: Deps,
        amount: Uint128,
        _unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let msg = BowStaking::ExecuteMsg::Withdraw {
            amount: Coin {
                denom: self.lp_token_denom.clone(),
                amount,
            },
        };
        Ok(vec![wasm_execute(
            self.staking_contract_address.clone(),
            &msg,
            vec![],
        )?
        .into()])
    }

    fn claim(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError> {
        Ok(vec![])
    }

    fn claim_rewards(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let msg = BowStaking::ExecuteMsg::Claim {
            denom: self.lp_token_denom.clone().into(),
        };
        Ok(vec![wasm_execute(
            self.staking_contract_address.clone(),
            &msg,
            vec![],
        )?
        .into()])
    }

    fn query_info(&self, _querier: &QuerierWrapper) -> Result<StakingInfoResponse, CwStakingError> {
        let lp_token = AssetInfo::Native(self.lp_token_denom.clone());

        Ok(StakingInfoResponse {
            staking_contract_address: self.staking_contract_address.clone(),
            staking_token: lp_token,
            unbonding_periods: None,
            max_claims: None,
        })
    }

    fn query_staked(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
        _unbonding_period: Option<Duration>,
    ) -> Result<StakeResponse, CwStakingError> {
        let stake_response: BowStaking::StakeResponse = querier
            .query_wasm_smart(
                self.staking_contract_address.clone(),
                &BowStaking::QueryMsg::Stake {
                    denom: self.lp_token_denom.clone().into(),
                    addr: staker.clone(),
                },
            )
            .map_err(|e| {
                StdError::generic_err(format!(
                    "Failed to query staked balance on {} for {}. Error: {:?}",
                    self.name(),
                    staker,
                    e
                ))
            })?;
        Ok(StakeResponse {
            amount: stake_response.amount,
        })
    }

    fn query_unbonding(
        &self,
        _querier: &QuerierWrapper,
        _staker: Addr,
    ) -> Result<UnbondingResponse, CwStakingError> {
        Ok(UnbondingResponse { claims: vec![] })
    }

    fn query_rewards(
        &self,
        querier: &QuerierWrapper,
    ) -> Result<abstract_staking_adapter_traits::msg::RewardTokensResponse, CwStakingError> {
        let reward_info: BowStaking::IncentivesResponse = querier
            .query_wasm_smart(
                self.staking_contract_address.clone(),
                &BowStaking::QueryMsg::Incentives {
                    denom: self.lp_token_denom.clone().into(),
                    start_after: None,
                    limit: None,
                },
            )
            .map_err(|e| {
                StdError::generic_err(format!(
                    "Failed to query reward info on {} for lp token {}. Error: {:?}",
                    self.name(),
                    self.lp_token,
                    e
                ))
            })?;

        let reward_tokens = reward_info
            .incentives
            .into_iter()
            .map(|asset| {
                let token = AssetInfo::Native(asset.denom.to_string());
                Result::<_, CwStakingError>::Ok(token)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RewardTokensResponse {
            tokens: reward_tokens,
        })
    }
}
