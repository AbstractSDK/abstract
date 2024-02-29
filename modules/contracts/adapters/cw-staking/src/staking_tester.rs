use crate::{interface::CwStakingAdapter, msg::InstantiateMsg, CW_STAKING_ADAPTER_ID};
use abstract_client::{AbstractClient, ClientResolve, Environment};
use abstract_core::{
    adapter,
    objects::{
        module::{ModuleInfo, ModuleVersion},
        pool_id::PoolAddressBase,
        AnsAsset, AssetEntry, LpToken, PoolMetadata,
    },
};
use abstract_interface::{AdapterDeployer, DeployStrategy, ExecuteMsgFns, VCExecFns};
use abstract_staking_standard::msg::{
    StakeResponse, StakingAction, StakingExecuteMsg, StakingQueryMsg,
};
use cosmwasm_std::{coins, Decimal, Uint128};
use cw_asset::AssetInfoUnchecked;
use cw_orch::{anyhow, environment::MutCwEnv, prelude::*};

pub trait MockStaking {
    /// Name of the staking provider
    fn name(&self) -> String;

    /// Stake token
    fn stake_token(&self) -> (String, AssetInfoUnchecked);

    /// Mint lp
    fn mint_lp(&self, addr: &Addr, amount: u128) -> anyhow::Result<()>;
}

pub struct StakingTester<Chain: MutCwEnv, StakingProvider: MockStaking> {
    pub abstr_deployment: AbstractClient<Chain>,
    pub staking_adapter: CwStakingAdapter<Chain>,
    pub provider: StakingProvider,
}

impl<Chain: MutCwEnv, StakingProvider: MockStaking> StakingTester<Chain, StakingProvider> {
    pub fn new(
        abstr_deployment: AbstractClient<Chain>,
        provider: StakingProvider,
    ) -> anyhow::Result<Self> {
        // Re-register cw-staking, to make sure it's latest
        let _ = abstr_deployment
            .version_control()
            .remove_module(ModuleInfo::from_id(
                CW_STAKING_ADAPTER_ID,
                ModuleVersion::Version(crate::contract::CONTRACT_VERSION.to_owned()),
            )?);
        let staking_adapter =
            CwStakingAdapter::new(CW_STAKING_ADAPTER_ID, abstr_deployment.environment());
        staking_adapter.deploy(
            crate::contract::CONTRACT_VERSION.parse()?,
            Empty {},
            DeployStrategy::Force,
        )?;

        Ok(Self {
            abstr_deployment,
            staking_adapter,
            provider,
        })
    }

    pub fn test_stake(&self) -> anyhow::Result<()> {
        let (ans_stake_token, _) = self.provider.stake_token();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<CwStakingAdapter<Chain>>()?
            .build()?;
        let proxy_addr = new_account.proxy()?;

        let stake_value = 1_000_000_000u128;

        self.provider.mint_lp(&proxy_addr, stake_value)?;

        // TODO: unbonding period

        // stake 1_000_000_000
        self.staking_adapter.execute(
            &crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: Some(proxy_addr.to_string()),
                request: StakingExecuteMsg {
                    provider: self.provider.name(),
                    action: StakingAction::Stake {
                        assets: vec![AnsAsset::new(ans_stake_token.clone(), stake_value)],
                        unbonding_period: None,
                    },
                },
            }),
            None,
        )?;

        // Assert staked
        let stake_response: StakeResponse =
            self.staking_adapter
                .query(&crate::msg::QueryMsg::Module(StakingQueryMsg::Staked {
                    provider: self.provider.name(),
                    stakes: vec![ans_stake_token.into()],
                    staker_address: proxy_addr.to_string(),
                    unbonding_period: None,
                }))?;

        assert_eq!(stake_response.amounts, vec![Uint128::new(stake_value)]);
        Ok(())
    }

    fn add_proxy_balance(
        &self,
        proxy_addr: &Addr,
        asset: &AssetInfoUnchecked,
        amount: u128,
    ) -> anyhow::Result<()> {
        let mut chain = self.abstr_deployment.environment();

        match asset {
            cw_asset::AssetInfoBase::Native(denom) => {
                chain.add_balance(proxy_addr, coins(amount, denom))?;
            }
            cw_asset::AssetInfoBase::Cw20(addr) => {
                chain.execute(
                    &cw20::Cw20ExecuteMsg::Mint {
                        recipient: proxy_addr.to_string(),
                        amount: amount.into(),
                    },
                    &[],
                    &Addr::unchecked(addr),
                )?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn query_proxy_balance(
        &self,
        proxy_addr: &Addr,
        asset: &AssetInfoUnchecked,
    ) -> anyhow::Result<Uint128> {
        let chain = self.abstr_deployment.environment();

        let balance = match asset {
            cw_asset::AssetInfoBase::Native(denom) => {
                chain
                    .bank_querier()
                    .balance(proxy_addr, Some(denom.to_owned()))
                    .unwrap()
                    .pop()
                    .unwrap()
                    .amount
            }
            cw_asset::AssetInfoBase::Cw20(addr) => {
                let balance: cw20::BalanceResponse = chain
                    .query(
                        &cw20::Cw20QueryMsg::Balance {
                            address: proxy_addr.to_string(),
                        },
                        &Addr::unchecked(addr),
                    )
                    .unwrap();
                balance.balance
            }
            _ => unreachable!(),
        };

        Ok(balance)
    }
}
