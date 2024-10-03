use abstract_sdk::feature_objects::VersionControlContract;
use abstract_staking_standard::Identify;
use cosmwasm_std::Addr;

use crate::{ASTROVAULT, AVAILABLE_CHAINS};

#[derive(Clone, Debug, Default)]
pub struct Astrovault {
    pub addr_as_sender: Option<Addr>,
    pub version_control_contract: Option<VersionControlContract>,
    pub tokens: Vec<AstrovaultTokenContext>,
}

#[derive(Clone, Debug)]
pub struct AstrovaultTokenContext {
    pub lp_token_address: Addr,
    pub staking_contract_address: Addr,
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
use {
    crate::mini_astrovault,
    crate::mini_astrovault::RewardSourceResponse,
    abstract_sdk::{
        feature_objects::AnsHost,
        features::AbstractRegistryAccess,
        std::objects::{AnsAsset, AssetEntry},
        Resolve,
    },
    abstract_staking_standard::msg::{
        Claim, RewardTokensResponse, StakeResponse, StakingInfo, StakingInfoResponse,
        UnbondingResponse,
    },
    abstract_staking_standard::{CwStakingCommand, CwStakingError},
    cosmwasm_std::{
        to_json_binary, wasm_execute, CosmosMsg, Deps, Env, QuerierWrapper, StdError, Timestamp,
        Uint128,
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
        addr_as_sender: Option<Addr>,
        ans_host: &AnsHost,
        version_control_contract: VersionControlContract,
        lp_tokens: Vec<AssetEntry>,
    ) -> Result<(), CwStakingError> {
        self.version_control_contract = Some(version_control_contract);
        self.addr_as_sender = addr_as_sender;
        self.tokens = lp_tokens
            .into_iter()
            .map(|entry| {
                let staking_contract_address =
                    self.staking_contract_address(deps, ans_host, &entry)?;
                let AssetInfo::Cw20(token_addr) = entry.resolve(&deps.querier, ans_host)? else {
                    return Err(
                        StdError::generic_err("expected CW20 as LP token for staking.").into(),
                    );
                };

                let lp_token_address = token_addr;

                Ok(AstrovaultTokenContext {
                    lp_token_address,
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
        let msg = to_json_binary(&mini_astrovault::AstrovaultStakingReceiveMsg::Deposit {
            sender: None,
            not_claim_rewards: Some(true),
            notify: None,
        })?;

        let stake_msgs = stake_request
            .into_iter()
            .zip(self.tokens.iter())
            .map(|(stake, token)| {
                let msg: CosmosMsg = wasm_execute(
                    token.lp_token_address.to_string(),
                    &Cw20ExecuteMsg::Send {
                        contract: token.staking_contract_address.to_string(),
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
                    token.staking_contract_address.to_string(),
                    &mini_astrovault::AstrovaultStakingExecuteMsg::Withdrawal {
                        amount: Some(unstake.amount),
                        direct_pool_withdrawal: None,
                        to: None,
                        not_claim_rewards: Some(false),
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
        let claim_msgs = self
            .tokens
            .iter()
            .map(|context| {
                let msg: CosmosMsg = wasm_execute(
                    context.staking_contract_address.to_string(),
                    &mini_astrovault::AstrovaultStakingExecuteMsg::WithdrawalFromLockup {
                        to: None,
                        direct_pool_withdrawal: None,
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

    fn claim_rewards(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let withdraw_msgs = self
            .tokens
            .iter()
            .map(|context| {
                let msg: CosmosMsg = wasm_execute(
                    context.staking_contract_address.to_string(),
                    &mini_astrovault::AstrovaultStakingExecuteMsg::Withdrawal {
                        amount: Some(Uint128::zero()),
                        direct_pool_withdrawal: None,
                        to: None,
                        not_claim_rewards: Some(false),
                        withdrawal_unlocked: None,
                        notify: None,
                    },
                    vec![],
                )?
                .into();
                Ok(msg)
            })
            .collect::<Result<_, CwStakingError>>()?;
        Ok(withdraw_msgs)
    }

    fn query_info(&self, querier: &QuerierWrapper) -> Result<StakingInfoResponse, CwStakingError> {
        let staking_addrs: HashSet<&Addr> = self
            .tokens
            .iter()
            .map(|t| &t.staking_contract_address)
            .collect();

        let mut infos = Vec::with_capacity(staking_addrs.len());
        for staking_addr in staking_addrs {
            let mini_astrovault::LpConfigResponse { inc_token, .. } = querier
                .query_wasm_smart(
                    staking_addr.clone(),
                    &mini_astrovault::AstrovaultStakingQueryMsg::Config {},
                )
                .map_err(|e| {
                    StdError::generic_err(format!(
                        "Failed to query staking info for {} with lp_staking: {}, {:?}",
                        self.name(),
                        staking_addr.clone(),
                        e
                    ))
                })?;

            // TODO: they store it as CanonicalAddress
            // Is it safe to do this cast?
            let contract_addr = Addr::unchecked(inc_token);
            let astro_token = AssetInfo::cw20(contract_addr);

            infos.push(StakingInfo {
                staking_target: staking_addr.clone().into(),
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
                let stake_balance: mini_astrovault::LpBalanceResponse = querier
                    .query_wasm_smart(
                        t.staking_contract_address.clone(),
                        &mini_astrovault::AstrovaultStakingQueryMsg::Balance {
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
                Ok(stake_balance.locked)
            })
            .collect::<Result<_, CwStakingError>>()?;

        Ok(StakeResponse { amounts })
    }

    fn query_unbonding(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
    ) -> Result<UnbondingResponse, CwStakingError> {
        let claims: Vec<Vec<Claim>> = self
            .tokens
            .iter()
            .map(|t| {
                let stake_balance: mini_astrovault::LpBalanceResponse = querier
                    .query_wasm_smart(
                        t.staking_contract_address.clone(),
                        &mini_astrovault::AstrovaultStakingQueryMsg::Balance {
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
                let claim = stake_balance
                    .pending_lockup_withdrawals
                    .into_iter()
                    .map(|withdrawal| Claim {
                        claimable_at: cw_utils::Expiration::AtTime(Timestamp::from_seconds(
                            withdrawal.withdrawal_timestamp,
                        )),
                        amount: withdrawal.to_withdrawal_amount,
                    })
                    .collect();
                Ok(claim)
            })
            .collect::<Result<_, CwStakingError>>()?;

        Ok(UnbondingResponse { claims })
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
                        t.staking_contract_address.clone(),
                        &mini_astrovault::AstrovaultStakingQueryMsg::RewardSources {
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
                        mini_astrovault::AstrovaultAssetInfo::Token { contract_addr } => {
                            AssetInfo::cw20(Addr::unchecked(contract_addr))
                        }
                        mini_astrovault::AstrovaultAssetInfo::NativeToken { denom } => {
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
        _: &cosmwasm_std::Env,
    ) -> std::result::Result<VersionControlContract, abstract_sdk::AbstractSdkError> {
        self.version_control_contract
            .clone()
            .ok_or(abstract_sdk::AbstractSdkError::generic_err(
                "version_control address is not set",
            ))
        // We need to get to the version control somehow (possible from Ans Host ?)
    }
}

#[cfg(feature = "full_integration")]
impl abstract_sdk::features::ModuleIdentification for Astrovault {
    fn module_id(&self) -> abstract_sdk::std::objects::module::ModuleId<'static> {
        abstract_staking_standard::CW_STAKING_ADAPTER_ID
    }
}
