use crate::AVAILABLE_CHAINS;
pub use crate::WYNDEX;
use abstract_sdk::core::objects::LpToken;
use abstract_staking_adapter_traits::Identify;
use cosmwasm_std::{Addr, Env};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WynDex {
    lp_token: LpToken,
    lp_token_address: Addr,
    staking_contract_address: Addr,
    ans_host: Addr,
    env: Option<Env>,
}

impl Default for WynDex {
    fn default() -> Self {
        Self {
            lp_token: Default::default(),
            lp_token_address: Addr::unchecked(""),
            staking_contract_address: Addr::unchecked(""),
            ans_host: Addr::unchecked(""),
            env: None,
        }
    }
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
use ::{
    abstract_sdk::{
        core::objects::{AnsEntryConvertor, AssetEntry},
        feature_objects::AnsHost,
        AbstractSdkError, Resolve,
    },
    abstract_staking_adapter_traits::msg::{
        Claim, RewardTokensResponse, StakeResponse, StakingInfoResponse, UnbondingResponse,
    },
    abstract_staking_adapter_traits::CwStakingCommand,
    abstract_staking_adapter_traits::CwStakingError,
    cosmwasm_std::{to_binary, CosmosMsg, Deps, QuerierWrapper, StdError, Uint128, WasmMsg},
    cw20::Cw20ExecuteMsg,
    cw_asset::{AssetInfo, AssetInfoBase},
    cw_utils::Duration,
    wyndex::stake::ReceiveMsg,
    wyndex_stake::msg::DistributionDataResponse,
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
        env: Env,
        _info: Option<cosmwasm_std::MessageInfo>,
        ans_host: &AnsHost,
        _abstract_registry: Addr,
        lp_token: AssetEntry,
    ) -> std::result::Result<(), AbstractSdkError> {
        self.staking_contract_address = self.staking_contract_address(deps, ans_host, &lp_token)?;

        let AssetInfoBase::Cw20(token_addr) = lp_token.resolve(&deps.querier, ans_host)? else {
                return Err(StdError::generic_err("expected CW20 as LP token for staking.").into());
            };
        self.lp_token_address = token_addr;

        self.lp_token = AnsEntryConvertor::new(lp_token).lp_token()?;
        self.env = Some(env);
        Ok(())
    }

    fn stake(
        &self,
        _deps: Deps,
        amount: Uint128,
        unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let unbonding_period = unwrap_unbond(self, unbonding_period)?;
        let msg = to_binary(&ReceiveMsg::Delegate {
            unbonding_period,
            delegate_as: None,
        })?;
        Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.lp_token_address.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: self.staking_contract_address.to_string(),
                amount,
                msg,
            })?,
            funds: vec![],
        })])
    }

    fn unstake(
        &self,
        _deps: Deps,
        amount: Uint128,
        unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let unbonding_period = unwrap_unbond(self, unbonding_period)?;
        let msg = StakeCw20ExecuteMsg::Unbond {
            tokens: amount,
            unbonding_period,
        };
        Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.staking_contract_address.to_string(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })])
    }

    fn claim(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let msg = StakeCw20ExecuteMsg::Claim {};

        Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.staking_contract_address.to_string(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })])
    }

    fn claim_rewards(&self, _deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError> {
        let msg = StakeCw20ExecuteMsg::WithdrawRewards {
            owner: None,
            receiver: None,
        };
        Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.staking_contract_address.to_string(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })])
    }

    fn query_info(&self, querier: &QuerierWrapper) -> StakingResult<StakingInfoResponse> {
        let bonding_info_resp: BondingInfoResponse = querier.query_wasm_smart(
            self.staking_contract_address.clone(),
            &wyndex_stake::msg::QueryMsg::BondingInfo {},
        )?;

        Ok(StakingInfoResponse {
            staking_target: self.staking_contract_address.clone().into(),
            staking_token: AssetInfo::Cw20(self.lp_token_address.clone()),
            unbonding_periods: Some(
                bonding_info_resp
                    .bonding
                    .into_iter()
                    .map(|bond_period| Duration::Time(bond_period.unbonding_period))
                    .collect(),
            ),
            max_claims: None,
        })
    }

    fn query_staked(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
        unbonding_period: Option<Duration>,
    ) -> StakingResult<StakeResponse> {
        let unbonding_period = unwrap_unbond(self, unbonding_period)
            .map_err(|e| StdError::generic_err(e.to_string()))?;

        // Raw query because the smart-query returns staked + currently unbonding tokens, which is not what we want.
        // we want the actual staked token balance.
        let stake_balance_res: Result<Option<BondingInfo>, _> = STAKE.query(
            querier,
            self.staking_contract_address.clone(),
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
        Ok(StakeResponse { amount })
    }

    fn query_unbonding(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
    ) -> StakingResult<UnbondingResponse> {
        let claims: cw_controllers::ClaimsResponse = querier.query_wasm_smart(
            self.staking_contract_address.clone(),
            &wyndex_stake::msg::QueryMsg::Claims {
                address: staker.into_string(),
            },
        )?;
        let claims = claims
            .claims
            .iter()
            .map(|claim| Claim {
                amount: claim.amount,
                claimable_at: claim.release_at,
            })
            .collect();
        Ok(UnbondingResponse { claims })
    }

    fn query_rewards(&self, querier: &QuerierWrapper) -> StakingResult<RewardTokensResponse> {
        let resp: DistributionDataResponse = querier.query_wasm_smart(
            self.staking_contract_address.clone(),
            &wyndex_stake::msg::QueryMsg::DistributionData {},
        )?;
        let reward_tokens = resp
            .distributions
            .into_iter()
            .map(|(asset, _)| {
                let token = match asset {
                    wyndex::asset::AssetInfoValidated::Native(denom) => AssetInfo::Native(denom),
                    wyndex::asset::AssetInfoValidated::Token(token) => AssetInfo::Cw20(token),
                };
                Result::<_, CwStakingError>::Ok(token)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RewardTokensResponse {
            tokens: reward_tokens,
        })
    }
}

#[cfg(feature = "full_integration")]
fn unwrap_unbond(dex: &WynDex, unbonding_period: Option<Duration>) -> Result<u64, CwStakingError> {
    let Some(Duration::Time(unbonding_period)) = unbonding_period else {
        if unbonding_period.is_none() {
            return Err(CwStakingError::UnbondingPeriodNotSet(dex.name().to_owned()));
        } else {
            return Err(CwStakingError::UnbondingPeriodNotSupported("height".to_owned(), dex.name().to_owned()));
        }
    };
    Ok(unbonding_period)
}
