use abstract_sdk::std::objects::LpToken;
use abstract_staking_standard::Identify;
use cosmwasm_std::Addr;

use crate::AVAILABLE_CHAINS;

pub const BOW: &str = "bow";

// TODO: use optional values here?
#[derive(Clone, Debug, Default)]
pub struct Bow {
    pub tokens: Vec<KujiraTokenContext>,
}

#[derive(Clone, Debug)]
pub struct KujiraTokenContext {
    pub lp_token: LpToken,
    pub lp_token_denom: String,
    pub staking_contract_address: Addr,
}

impl Identify for Bow {
    fn name(&self) -> &'static str {
        BOW
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
}

#[cfg(feature = "full_integration")]
use ::{
    abstract_sdk::{
        feature_objects::{AnsHost, VersionControlContract},
        std::objects::{AnsAsset, AnsEntryConvertor, AssetEntry},
        Resolve,
    },
    abstract_staking_standard::msg::{
        RewardTokensResponse, StakeResponse, StakingInfoResponse, UnbondingResponse,
    },
    abstract_staking_standard::{msg::StakingInfo, CwStakingCommand, CwStakingError},
    cosmwasm_std::{wasm_execute, Coin, CosmosMsg, Deps, Env, QuerierWrapper, StdError, Uint128},
    cw_asset::{AssetInfo, AssetInfoBase},
    kujira::bow::staking as BowStaking,
};

#[cfg(feature = "full_integration")]
impl CwStakingCommand for Bow {
    fn fetch_data(
        &mut self,
        deps: Deps,
        _env: Env,
        _addr_as_sender: Option<Addr>,
        ans_host: &AnsHost,
        _version_control_contract: VersionControlContract,
        lp_tokens: Vec<AssetEntry>,
    ) -> Result<(), CwStakingError> {
        self.tokens = lp_tokens
            .into_iter()
            .map(|entry| {
                let staking_contract_address =
                    self.staking_contract_address(deps, ans_host, &entry)?;

                let AssetInfoBase::Native(denom) = entry.resolve(&deps.querier, ans_host)? else {
                    return Err(
                        StdError::generic_err("expected denom as LP token for staking.").into(),
                    );
                };
                let lp_token_denom = denom;
                let lp_token = AnsEntryConvertor::new(entry.clone()).lp_token()?;

                Ok(KujiraTokenContext {
                    lp_token,
                    lp_token_denom,
                    staking_contract_address,
                })
            })
            .collect::<Result<_, CwStakingError>>()?;

        Ok(())
    }

    fn stake(
        &self,
        _deps: Deps,
        stake_request: Vec<AnsAsset>,
        _unbonding_period: Option<cw_utils::Duration>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let msg = BowStaking::ExecuteMsg::Stake { addr: None };

        let stake_msgs = stake_request
            .into_iter()
            .zip(self.tokens.iter())
            .map(|(stake, token)| {
                let msg: CosmosMsg = wasm_execute(
                    token.staking_contract_address.clone(),
                    &msg,
                    vec![Coin {
                        amount: stake.amount,
                        denom: token.lp_token_denom.clone(),
                    }],
                )?
                .into();
                Ok(msg)
            })
            .collect::<Result<_, CwStakingError>>()?;

        Ok(stake_msgs)
    }

    fn unstake(
        &self,
        _deps: Deps,
        unstake_request: Vec<AnsAsset>,
        _unbonding_period: Option<cw_utils::Duration>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let unstake_msgs = unstake_request
            .into_iter()
            .zip(self.tokens.iter())
            .map(|(unstake, token)| {
                let msg: CosmosMsg = wasm_execute(
                    token.staking_contract_address.clone(),
                    &BowStaking::ExecuteMsg::Withdraw {
                        amount: Coin {
                            denom: token.lp_token_denom.clone(),
                            amount: unstake.amount,
                        },
                    },
                    vec![],
                )?
                .into();
                Ok(msg)
            })
            .collect::<Result<_, CwStakingError>>()?;
        Ok(unstake_msgs)
    }

    fn claim(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError> {
        Ok(vec![])
    }

    fn claim_rewards(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let claim_msgs = self
            .tokens
            .iter()
            .map(|t| {
                let msg: CosmosMsg = wasm_execute(
                    t.staking_contract_address.clone(),
                    &BowStaking::ExecuteMsg::Claim {
                        denom: t.lp_token_denom.clone().into(),
                    },
                    vec![],
                )?
                .into();
                Ok(msg)
            })
            .collect::<Result<_, CwStakingError>>()?;

        Ok(claim_msgs)
    }

    fn query_info(&self, _querier: &QuerierWrapper) -> Result<StakingInfoResponse, CwStakingError> {
        let infos = self
            .tokens
            .iter()
            .map(|t| {
                let lp_token = AssetInfo::Native(t.lp_token_denom.clone());
                StakingInfo {
                    staking_target: t.staking_contract_address.clone().into(),
                    staking_token: lp_token,
                    unbonding_periods: None,
                    max_claims: None,
                }
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
        let amounts: Vec<Uint128> = self
            .tokens
            .iter()
            .map(|t| {
                let stake_response: BowStaking::StakeResponse = querier
                    .query_wasm_smart(
                        t.staking_contract_address.clone(),
                        &BowStaking::QueryMsg::Stake {
                            denom: t.lp_token_denom.clone().into(),
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
                Ok(stake_response.amount)
            })
            .collect::<Result<_, CwStakingError>>()?;

        Ok(StakeResponse { amounts })
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
    ) -> Result<abstract_staking_standard::msg::RewardTokensResponse, CwStakingError> {
        let tokens = self
            .tokens
            .iter()
            .map(|t| {
                let reward_info: BowStaking::IncentivesResponse = querier
                    .query_wasm_smart(
                        t.staking_contract_address.clone(),
                        &BowStaking::QueryMsg::Incentives {
                            denom: t.lp_token_denom.clone().into(),
                            start_after: None,
                            limit: None,
                        },
                    )
                    .map_err(|e| {
                        StdError::generic_err(format!(
                            "Failed to query reward info on {} for lp token {}. Error: {:?}",
                            self.name(),
                            t.lp_token,
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
                    .collect::<Result<_, _>>()?;
                Ok(reward_tokens)
            })
            .collect::<Result<_, CwStakingError>>()?;

        Ok(RewardTokensResponse { tokens })
    }
}

#[cfg(feature = "full_integration")]
impl abstract_sdk::features::ModuleIdentification for Bow {
    fn module_id(&self) -> abstract_sdk::std::objects::module::ModuleId<'static> {
        abstract_staking_standard::CW_STAKING_ADAPTER_ID
    }
}
