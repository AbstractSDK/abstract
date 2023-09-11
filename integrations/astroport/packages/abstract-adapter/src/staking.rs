use std::collections::HashMap;
use std::collections::HashSet;

use crate::ASTROPORT;
use crate::AVAILABLE_CHAINS;
use abstract_sdk::core::objects::LpToken;
use abstract_staking_adapter_traits::msg::StakeRequest;
use abstract_staking_adapter_traits::msg::StakedQuery;
use abstract_staking_adapter_traits::msg::StakingInfo;
use abstract_staking_adapter_traits::msg::UnstakeRequest;
use abstract_staking_adapter_traits::Identify;
use cosmwasm_std::Addr;

#[derive(Clone, Debug)]
pub struct Astroport {
    tokens: Vec<AstroportTokenContext>,
}

#[derive(Clone, Debug)]
pub struct AstroportTokenContext {
    #[allow(unused)]
    lp_token: LpToken,
    #[allow(unused)]
    lp_token_address: Addr,
    #[allow(unused)]
    generator_contract_address: Addr,
}

impl Default for Astroport {
    fn default() -> Self {
        Self {
            tokens: Default::default(),
        }
    }
}

// Data that's retrieved from ANS
// - LP token address, based on provided LP token
// - Generator address = staking_address
impl Identify for Astroport {
    fn name(&self) -> &'static str {
        ASTROPORT
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
}

#[cfg(feature = "full_integration")]
use ::{
    abstract_sdk::{
        core::objects::{AnsEntryConvertor, AssetEntry},
        feature_objects::{AnsHost, VersionControlContract},
        AbstractSdkResult, Resolve,
    },
    abstract_staking_adapter_traits::msg::{
        RewardTokensResponse, StakeResponse, StakingInfoResponse, UnbondingResponse,
    },
    abstract_staking_adapter_traits::{CwStakingCommand, CwStakingError},
    astroport::generator::{
        Config, Cw20HookMsg, ExecuteMsg as GeneratorExecuteMsg, QueryMsg as GeneratorQueryMsg,
        RewardInfoResponse,
    },
    cosmwasm_std::{
        to_binary, wasm_execute, CosmosMsg, Deps, Env, QuerierWrapper, StdError, Uint128,
    },
    cw20::Cw20ExecuteMsg,
    cw_asset::AssetInfo,
};

#[cfg(feature = "full_integration")]
impl CwStakingCommand for Astroport {
    fn fetch_data(
        &mut self,
        deps: Deps,
        _env: Env,
        _info: Option<cosmwasm_std::MessageInfo>,
        ans_host: &AnsHost,
        _version_control_contract: &VersionControlContract,
        lp_tokens: impl IntoIterator<Item = AssetEntry>,
    ) -> AbstractSdkResult<()> {
        let lp_tokens = lp_tokens.into_iter();

        self.tokens = lp_tokens
            .map(|lp_token| {
                let generator_contract_address =
                    self.staking_contract_address(deps, ans_host, &lp_token)?;

                let AssetInfo::Cw20(token_addr) = lp_token.resolve(&deps.querier, ans_host)? else {
                    return Err(
                        StdError::generic_err("expected CW20 as LP token for staking.").into(),
                    );
                };

                let lp_token_address = token_addr;
                let lp_token = AnsEntryConvertor::new(lp_token).lp_token()?;

                Ok(AstroportTokenContext {
                    lp_token,
                    lp_token_address,
                    generator_contract_address,
                })
            })
            .collect::<AbstractSdkResult<_>>()?;
        Ok(())
    }

    fn stake(
        &self,
        _deps: Deps,
        stake_request: Vec<StakeRequest>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let msg = to_binary(&Cw20HookMsg::Deposit {})?;

        let stake_msgs = stake_request
            .into_iter()
            .zip(self.tokens.iter())
            .map(|(stake, token)| {
                let msg: CosmosMsg = wasm_execute(
                    token.lp_token_address.to_string(),
                    &Cw20ExecuteMsg::Send {
                        contract: token.generator_contract_address.to_string(),
                        amount: stake.asset.amount,
                        msg: msg.clone(),
                    },
                    vec![],
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
        unstake_request: Vec<UnstakeRequest>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let unstake_msgs = unstake_request
            .into_iter()
            .zip(self.tokens.iter())
            .map(|(unstake, token)| {
                let msg: CosmosMsg = wasm_execute(
                    token.generator_contract_address.to_string(),
                    &GeneratorExecuteMsg::Withdraw {
                        lp_token: token.lp_token_address.to_string(),
                        amount: unstake.asset.amount,
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
        let mut claims: HashMap<&str, Vec<String>> = HashMap::new();
        for token in &self.tokens {
            claims
                .entry(token.generator_contract_address.as_str())
                .and_modify(|tokens| tokens.push(token.lp_token_address.to_string()))
                .or_insert(vec![token.lp_token_address.to_string()]);
        }
        let claim_msgs = claims
            .into_iter()
            .map(|(generator_addr, lp_tokens)| {
                let msg: CosmosMsg = wasm_execute(
                    generator_addr.to_owned(),
                    &GeneratorExecuteMsg::ClaimRewards { lp_tokens },
                    vec![],
                )?
                .into();
                Ok(msg)
            })
            .collect::<Result<_, CwStakingError>>()?;
        Ok(claim_msgs)
    }

    fn query_info(&self, querier: &QuerierWrapper) -> Result<StakingInfoResponse, CwStakingError> {
        let generator_addrs: HashSet<&Addr> = self
            .tokens
            .iter()
            .map(|t| &t.generator_contract_address)
            .collect();

        let mut infos = Vec::with_capacity(generator_addrs.len());
        for g_addr in generator_addrs {
            let Config { astro_token, .. } = querier
                .query_wasm_smart::<Config>(g_addr.clone(), &GeneratorQueryMsg::Config {})
                .map_err(|e| {
                    StdError::generic_err(format!(
                        "Failed to query staking info for {} with generator: {}, {:?}",
                        self.name(),
                        g_addr.clone(),
                        e
                    ))
                })?;

            let astro_token = match astro_token {
                astroport::asset::AssetInfo::Token { contract_addr } => {
                    AssetInfo::cw20(contract_addr)
                }
                astroport::asset::AssetInfo::NativeToken { denom } => AssetInfo::native(denom),
            };

            infos.push(StakingInfo {
                staking_target: g_addr.clone().into(),
                staking_token: astro_token,
                unbonding_periods: None,
                max_claims: None,
            });
        }

        Ok(StakingInfoResponse { infos })
    }

    fn query_staked(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
        _stakes: Vec<StakedQuery>,
    ) -> Result<StakeResponse, CwStakingError> {
        let amounts = self
            .tokens
            .iter()
            .map(|t| {
                let stake_balance: Uint128 = querier
                    .query_wasm_smart(
                        t.generator_contract_address.clone(),
                        &GeneratorQueryMsg::Deposit {
                            lp_token: t.lp_token_address.to_string(),
                            user: staker.to_string(),
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
                Ok(stake_balance)
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
    ) -> Result<abstract_staking_adapter_traits::msg::RewardTokensResponse, CwStakingError> {
        let tokens =
            self.tokens
                .iter()
                .try_fold(vec![], |mut acc, t| -> Result<_, CwStakingError> {
                    let reward_info: RewardInfoResponse = querier
                        .query_wasm_smart(
                            t.generator_contract_address.clone(),
                            &GeneratorQueryMsg::RewardInfo {
                                lp_token: t.lp_token_address.to_string(),
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

                    let token = match reward_info.base_reward_token {
                        astroport::asset::AssetInfo::Token { contract_addr } => {
                            AssetInfo::cw20(contract_addr)
                        }
                        astroport::asset::AssetInfo::NativeToken { denom } => {
                            AssetInfo::native(denom)
                        }
                    };

                    acc.push(token);

                    if let Some(reward_token) = reward_info.proxy_reward_token {
                        acc.push(AssetInfo::cw20(reward_token));
                    }
                    Ok(acc)
                })?;

        Ok(RewardTokensResponse { tokens })
    }
}
