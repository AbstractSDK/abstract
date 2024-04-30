use abstract_sdk::std::objects::LpToken;
use abstract_staking_standard::Identify;
use cosmwasm_std::Addr;

use crate::AVAILABLE_CHAINS;
pub use crate::WYNDEX;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct WynDex {
    pub tokens: Vec<WynDexTokenContext>,
    pub ans_host: Option<Addr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WynDexTokenContext {
    pub lp_token: LpToken,
    pub lp_token_address: Addr,
    pub staking_contract_address: Addr,
}

impl Identify for WynDex {
    fn name(&self) -> &'static str {
        WYNDEX
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
}

#[cfg(feature = "full_integration")]
use {
    abstract_sdk::{
        feature_objects::{AnsHost, VersionControlContract},
        std::objects::{AnsAsset, AnsEntryConvertor, AssetEntry},
        Resolve,
    },
    abstract_staking_standard::msg::{
        Claim, RewardTokensResponse, StakeResponse, StakingInfo, StakingInfoResponse,
        UnbondingResponse,
    },
    abstract_staking_standard::CwStakingCommand,
    abstract_staking_standard::CwStakingError,
    cosmwasm_std::{to_json_binary, CosmosMsg, Deps, QuerierWrapper, StdError, Uint128, WasmMsg},
    cw20::Cw20ExecuteMsg,
    cw_asset::{AssetInfo, AssetInfoBase},
    cw_utils::Duration,
    wyndex_stake::msg::DistributionDataResponse,
    wyndex_stake::msg::ReceiveDelegationMsg,
    wyndex_stake::{
        msg::{BondingInfoResponse, ExecuteMsg as StakeCw20ExecuteMsg},
        state::{BondingInfo, STAKE},
    },
};

#[cfg(feature = "full_integration")]
type StakingResult<T> = Result<T, CwStakingError>;

#[cfg(feature = "full_integration")]
impl CwStakingCommand for WynDex {
    fn fetch_data(
        &mut self,
        deps: Deps,
        _env: cosmwasm_std::Env,
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
                let AssetInfoBase::Cw20(lp_token_address) =
                    entry.resolve(&deps.querier, ans_host)?
                else {
                    return Err(
                        StdError::generic_err("expected CW20 as LP token for staking.").into(),
                    );
                };

                let lp_token = AnsEntryConvertor::new(entry.clone()).lp_token()?;
                Ok(WynDexTokenContext {
                    lp_token,
                    lp_token_address,
                    staking_contract_address,
                })
            })
            .collect::<std::result::Result<_, CwStakingError>>()?;

        Ok(())
    }

    fn stake(
        &self,
        _deps: Deps,
        stake_request: Vec<AnsAsset>,
        unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let unbonding_period = unwrap_unbond(self, unbonding_period)?;
        let msg = to_json_binary(&ReceiveDelegationMsg::Delegate {
            unbonding_period,
            delegate_as: None,
        })?;
        let stake_msgs = stake_request
            .into_iter()
            .zip(self.tokens.iter())
            .map(|(stake, token)| {
                Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: token.lp_token_address.to_string(),
                    msg: to_json_binary(&Cw20ExecuteMsg::Send {
                        contract: token.staking_contract_address.to_string(),
                        amount: stake.amount,
                        msg: msg.clone(),
                    })?,
                    funds: vec![],
                }))
            })
            .collect::<Result<_, CwStakingError>>()?;

        Ok(stake_msgs)
    }

    fn unstake(
        &self,
        _deps: Deps,
        unstake_request: Vec<AnsAsset>,
        unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let unbonding_period = unwrap_unbond(self, unbonding_period)?;
        let unstake_msgs = unstake_request
            .into_iter()
            .zip(self.tokens.iter())
            .map(|(unstake, token)| {
                Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: token.staking_contract_address.to_string(),
                    msg: to_json_binary(&StakeCw20ExecuteMsg::Unbond {
                        tokens: unstake.amount,
                        unbonding_period,
                    })?,
                    funds: vec![],
                }))
            })
            .collect::<Result<_, CwStakingError>>()?;

        Ok(unstake_msgs)
    }

    fn claim(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let msg = to_json_binary(&StakeCw20ExecuteMsg::Claim {})?;

        let claim_msgs = self
            .tokens
            .iter()
            .map(|t| {
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: t.staking_contract_address.to_string(),
                    msg: msg.clone(),
                    funds: vec![],
                })
            })
            .collect();
        Ok(claim_msgs)
    }

    fn claim_rewards(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let msg = to_json_binary(&StakeCw20ExecuteMsg::WithdrawRewards {
            owner: None,
            receiver: None,
        })?;

        let claim_msgs = self
            .tokens
            .iter()
            .map(|t| {
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: t.staking_contract_address.to_string(),
                    msg: msg.clone(),
                    funds: vec![],
                })
            })
            .collect();
        Ok(claim_msgs)
    }

    fn query_info(&self, querier: &QuerierWrapper) -> StakingResult<StakingInfoResponse> {
        let infos = self
            .tokens
            .iter()
            .map(|t| {
                let bonding_info_resp: BondingInfoResponse = querier.query_wasm_smart(
                    t.staking_contract_address.clone(),
                    &wyndex_stake::msg::QueryMsg::BondingInfo {},
                )?;
                Ok(StakingInfo {
                    staking_target: t.staking_contract_address.clone().into(),
                    staking_token: AssetInfo::Cw20(t.lp_token_address.clone()),
                    unbonding_periods: Some(
                        bonding_info_resp
                            .bonding
                            .into_iter()
                            .map(|bond_period| Duration::Time(bond_period.unbonding_period))
                            .collect(),
                    ),
                    max_claims: None,
                })
            })
            .collect::<StakingResult<_>>()?;

        Ok(StakingInfoResponse { infos })
    }

    fn query_staked(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
        _stakes: Vec<AssetEntry>,
        unbonding_period: Option<Duration>,
    ) -> StakingResult<StakeResponse> {
        let unbonding_period = unwrap_unbond(self, unbonding_period)
            .map_err(|e| StdError::generic_err(e.to_string()))?;

        let amounts = self
            .tokens
            .iter()
            .map(|token| {
                // Raw query because the smart-query returns staked + currently unbonding tokens, which is not what we want.
                // we want the actual staked token balance.
                let stake_balance_res: Result<Option<BondingInfo>, _> = STAKE.query(
                    querier,
                    token.staking_contract_address.clone(),
                    (&staker, unbonding_period),
                );
                let stake_balance_info = stake_balance_res.map_err(|e| {
                    StdError::generic_err(format!(
                        "Raw query for wynddex stake balance failed. Error: {e:?}"
                    ))
                })?;

                let amount = if let Some(bonding_info) = stake_balance_info {
                    bonding_info.total_stake()
                } else {
                    Uint128::zero()
                };
                Ok(amount)
            })
            .collect::<StakingResult<_>>()?;

        Ok(StakeResponse { amounts })
    }

    fn query_unbonding(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
    ) -> StakingResult<UnbondingResponse> {
        let claims = self
            .tokens
            .iter()
            .map(|token| {
                let claims: cw_controllers::ClaimsResponse = querier.query_wasm_smart(
                    token.staking_contract_address.clone(),
                    &wyndex_stake::msg::QueryMsg::Claims {
                        address: staker.to_string(),
                    },
                )?;
                let claims: Vec<_> = claims
                    .claims
                    .iter()
                    .map(|claim| Claim {
                        amount: claim.amount,
                        claimable_at: claim.release_at,
                    })
                    .collect();
                Ok(claims)
            })
            .collect::<StakingResult<_>>()?;

        Ok(UnbondingResponse { claims })
    }

    fn query_rewards(&self, querier: &QuerierWrapper) -> StakingResult<RewardTokensResponse> {
        let tokens = self
            .tokens
            .iter()
            .map(|t| {
                let resp: DistributionDataResponse = querier.query_wasm_smart(
                    t.staking_contract_address.clone(),
                    &wyndex_stake::msg::QueryMsg::DistributionData {},
                )?;

                let reward_tokens = resp
                    .distributions
                    .into_iter()
                    .map(|(asset, _)| {
                        let token = match asset {
                            wyndex::asset::AssetInfoValidated::Native(denom) => {
                                AssetInfo::Native(denom)
                            }
                            wyndex::asset::AssetInfoValidated::Token(token) => {
                                AssetInfo::Cw20(token)
                            }
                        };
                        Ok(token)
                    })
                    .collect::<StakingResult<_>>()?;
                Ok(reward_tokens)
            })
            .collect::<StakingResult<_>>()?;
        Ok(RewardTokensResponse { tokens })
    }
}

#[cfg(feature = "full_integration")]
fn unwrap_unbond(dex: &WynDex, unbonding_period: Option<Duration>) -> Result<u64, CwStakingError> {
    match unbonding_period {
        // Only time supported for unbonding
        Some(Duration::Time(unbonding_period)) => Ok(unbonding_period),
        Some(Duration::Height(_)) => Err(CwStakingError::UnbondingPeriodNotSupported(
            "height".to_owned(),
            dex.name().to_owned(),
        )),
        None => Err(CwStakingError::UnbondingPeriodNotSet(dex.name().to_owned())),
    }
}
