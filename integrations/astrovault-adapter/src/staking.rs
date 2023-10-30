use crate::ASTROVAULT;
use crate::AVAILABLE_CHAINS;
use abstract_staking_standard::Identify;
use cosmwasm_std::Addr;

#[derive(Clone, Debug, Default)]
pub struct Astrovault {
    pub local_proxy_addr: Option<Addr>,
    pub version_control_contract: Option<VersionControlContract>,
    pub tokens: Vec<AstrovaultTokenContext>,
}

#[derive(Clone, Debug)]
pub struct AstrovaultTokenContext {
    pub lp_token_address: Addr,
}

// Data that's retrieved from ANS
// - LP token address, based on provided LP token
// - Generator address = staking_address
impl Identify for Astrovault {
    fn name(&self) -> &'static str {
        ASTROVAULT
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
}

#[cfg(feature = "full_integration")]
use ::{
    abstract_sdk::{
        core::objects::{AnsAsset, AssetEntry},
        feature_objects::{AnsHost, VersionControlContract},
        features::AbstractRegistryAccess,
        AbstractSdkResult, AccountVerification, Resolve,
    },
    abstract_staking_standard::msg::{
        RewardTokensResponse, StakeResponse, StakingInfo, StakingInfoResponse, UnbondingResponse,
    },
    abstract_staking_standard::{CwStakingCommand, CwStakingError},
    astrovault::lp_staking::{
        handle_msg::ExecuteMsg as LpExecuteMsg,
        query_msg::{LpConfigResponse, QueryMsg as LpQueryMsg, RewardSourceResponse},
    },
    cosmwasm_std::{
        to_binary, wasm_execute, CosmosMsg, Deps, Env, QuerierWrapper, StdError, Uint128,
    },
    cw20::Cw20ExecuteMsg,
    cw_asset::AssetInfo,
    std::collections::HashSet,
};

#[cfg(feature = "full_integration")]
impl CwStakingCommand for Astrovault {
    fn fetch_data(
        &mut self,
        deps: Deps,
        _env: Env,
        info: Option<cosmwasm_std::MessageInfo>,
        ans_host: &AnsHost,
        _version_control_contract: VersionControlContract,
        lp_tokens: Vec<AssetEntry>,
    ) -> AbstractSdkResult<()> {
        let base = info
            .map(|i| self.account_registry(deps).assert_manager(&i.sender))
            .transpose()?;
        self.local_proxy_addr = base.map(|b| b.proxy);
        self.tokens = lp_tokens
            .into_iter()
            .map(|entry| {
                let AssetInfo::Cw20(token_addr) = entry.resolve(&deps.querier, ans_host)? else {
                    return Err(
                        StdError::generic_err("expected CW20 as LP token for staking.").into(),
                    );
                };

                let lp_token_address = token_addr;
                // let lp_token = AnsEntryConvertor::new(entry.clone()).lp_token()?;

                Ok(AstrovaultTokenContext {
                    // lp_token,
                    lp_token_address,
                })
            })
            .collect::<AbstractSdkResult<_>>()?;
        Ok(())
    }

    fn stake(
        &self,
        _deps: Deps,
        stake_request: Vec<AnsAsset>,
        _unbonding_period: Option<cw_utils::Duration>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let msg = to_binary(
            &astrovault::lp_staking::handle_msg::LPStakingReceiveMsg::Deposit {
                sender: None,
                not_claim_rewards: None,
                notify: None,
            },
        )?;

        let stake_msgs = stake_request
            .into_iter()
            .zip(self.tokens.iter())
            .map(|(stake, token)| {
                let msg: CosmosMsg = wasm_execute(
                    token.lp_token_address.to_string(),
                    &Cw20ExecuteMsg::Send {
                        contract: token.lp_token_address.to_string(),
                        amount: stake.amount,
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
        unstake_request: Vec<AnsAsset>,
        _unbonding_period: Option<cw_utils::Duration>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let unstake_msgs = unstake_request
            .into_iter()
            .zip(self.tokens.iter())
            .map(|(unstake, token)| {
                let msg: CosmosMsg = wasm_execute(
                    token.lp_token_address.to_string(),
                    &LpExecuteMsg::Withdrawal {
                        amount: Some(unstake.amount),
                        direct_pool_withdrawal: None,
                        to: None,
                        not_claim_rewards: None,
                        withdrawal_unlocked: None,
                        notify: None,
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
            .map(|context| {
                let msg: CosmosMsg = wasm_execute(
                    context.lp_token_address.to_string(),
                    &LpExecuteMsg::Withdrawal {
                        amount: Some(Uint128::zero()),
                        direct_pool_withdrawal: None,
                        to: None,
                        not_claim_rewards: None,
                        withdrawal_unlocked: None,
                        notify: None,
                    },
                    vec![],
                )?
                .into();
                Ok(msg)
            })
            .collect::<Result<_, CwStakingError>>()?;
        Ok(claim_msgs)
    }

    fn query_info(&self, querier: &QuerierWrapper) -> Result<StakingInfoResponse, CwStakingError> {
        let generator_addrs: HashSet<&Addr> =
            self.tokens.iter().map(|t| &t.lp_token_address).collect();

        let mut infos = Vec::with_capacity(generator_addrs.len());
        for g_addr in generator_addrs {
            let LpConfigResponse { inc_token, .. } = querier
                .query_wasm_smart::<LpConfigResponse>(g_addr.clone(), &LpQueryMsg::Config {})
                .map_err(|e| {
                    StdError::generic_err(format!(
                        "Failed to query staking info for {} with lp_staking: {}, {:?}",
                        self.name(),
                        g_addr.clone(),
                        e
                    ))
                })?;

            // TODO: they store it as CanonicalAddress
            // Is it safe to do this cast?
            let contract_addr = Addr::unchecked(inc_token);
            let astro_token = AssetInfo::cw20(contract_addr);

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
        _stakes: Vec<AssetEntry>,
        _unbonding_period: Option<cw_utils::Duration>,
    ) -> Result<StakeResponse, CwStakingError> {
        let amounts = self
            .tokens
            .iter()
            .map(|t| {
                let stake_balance: Uint128 = querier
                    .query_wasm_smart(
                        t.lp_token_address.clone(),
                        &LpQueryMsg::Balance {
                            address: staker.to_string(),
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
    ) -> Result<abstract_staking_standard::msg::RewardTokensResponse, CwStakingError> {
        let tokens = self
            .tokens
            .iter()
            .map(|t| {
                let rewards_info: Vec<RewardSourceResponse> = querier
                    .query_wasm_smart(
                        t.lp_token_address.clone(),
                        &LpQueryMsg::RewardSources {
                            reward_source: None,
                        },
                    )
                    .map_err(|e| {
                        StdError::generic_err(format!(
                            "Failed to query reward info on {} for lp token. Error: {:?}",
                            self.name(),
                            e
                        ))
                    })?;

                let tokens = rewards_info
                    .into_iter()
                    .map(|rew_source| match rew_source.info.reward_asset {
                        astrovault::assets::asset::AssetInfo::Token { contract_addr } => {
                            AssetInfo::cw20(Addr::unchecked(contract_addr))
                        }
                        astrovault::assets::asset::AssetInfo::NativeToken { denom } => {
                            AssetInfo::native(denom)
                        }
                    })
                    .collect();

                Ok(tokens)
            })
            .collect::<Result<_, CwStakingError>>()?;

        Ok(RewardTokensResponse { tokens })
    }
}

#[cfg(feature = "full_integration")]
impl AbstractRegistryAccess for Astrovault {
    fn abstract_registry(
        &self,
        _: cosmwasm_std::Deps<'_>,
    ) -> std::result::Result<VersionControlContract, abstract_sdk::AbstractSdkError> {
        self.version_control_contract
            .clone()
            .ok_or(abstract_sdk::AbstractSdkError::generic_err(
                "version_control address is not set",
            ))
        // We need to get to the version control somehow (possible from Ans Host ?)
    }
}
