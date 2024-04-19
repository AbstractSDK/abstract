use crate::{interface::CwStakingAdapter, CW_STAKING_ADAPTER_ID};
use abstract_adapter::std::{
    adapter,
    objects::{
        module::{ModuleInfo, ModuleVersion},
        AnsAsset, AssetEntry,
    },
};
use abstract_client::{AbstractClient, Environment};
use abstract_interface::{AdapterDeployer, DeployStrategy, VCExecFns};
use abstract_staking_standard::msg::{
    RewardTokensResponse, StakeResponse, StakingAction, StakingExecuteMsg, StakingInfoResponse,
    StakingQueryMsg, StakingTarget,
};
use cosmwasm_std::Uint128;
use cw_asset::AssetInfoUnchecked;
use cw_orch::{environment::MutCwEnv, prelude::*};

// TODO: beta clippy trips here, try again later
#[allow(unused_imports)]
use cw_orch::anyhow;

pub trait MockStaking {
    /// Name of the staking provider
    fn name(&self) -> String;

    /// Stake token
    fn stake_token(&self) -> (String, AssetInfoUnchecked);

    /// Mint lp
    fn mint_lp(&self, addr: &Addr, amount: u128) -> anyhow::Result<()>;

    /// Generate rewards
    fn generate_rewards(&self, addr: &Addr, amount: u128) -> anyhow::Result<()>;

    /// Staking_target
    fn staking_target(&self) -> StakingTarget;

    /// Reward asset
    fn reward_asset(&self) -> AssetInfoUnchecked;
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
        // Ensure staked
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

    pub fn test_unstake(&self) -> anyhow::Result<()> {
        let (ans_stake_token, lp_asset) = self.provider.stake_token();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<CwStakingAdapter<Chain>>()?
            .build()?;
        let proxy_addr = new_account.proxy()?;

        let stake_value = 1_000_000_000u128;

        self.provider.mint_lp(&proxy_addr, stake_value * 2)?;

        // TODO: unbonding period

        // stake 1_000_000_000 * 2
        self.staking_adapter.execute(
            &crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: Some(proxy_addr.to_string()),
                request: StakingExecuteMsg {
                    provider: self.provider.name(),
                    action: StakingAction::Stake {
                        assets: vec![AnsAsset::new(ans_stake_token.clone(), stake_value * 2)],
                        unbonding_period: None,
                    },
                },
            }),
            None,
        )?;

        // Unstake half
        self.staking_adapter.execute(
            &crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: Some(proxy_addr.to_string()),
                request: StakingExecuteMsg {
                    provider: self.provider.name(),
                    action: StakingAction::Unstake {
                        assets: vec![AnsAsset::new(ans_stake_token.clone(), stake_value)],
                        unbonding_period: None,
                    },
                },
            }),
            None,
        )?;

        // Ensure user got his lp back
        let lp_balance = self.query_proxy_balance(&proxy_addr, &lp_asset)?.u128();
        assert_eq!(lp_balance, stake_value);

        // Unstake rest
        self.staking_adapter.execute(
            &crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: Some(proxy_addr.to_string()),
                request: StakingExecuteMsg {
                    provider: self.provider.name(),
                    action: StakingAction::Unstake {
                        assets: vec![AnsAsset::new(ans_stake_token.clone(), stake_value)],
                        unbonding_period: None,
                    },
                },
            }),
            None,
        )?;

        // Ensure unstaked
        let stake_response: StakeResponse =
            self.staking_adapter
                .query(&crate::msg::QueryMsg::Module(StakingQueryMsg::Staked {
                    provider: self.provider.name(),
                    stakes: vec![ans_stake_token.into()],
                    staker_address: proxy_addr.to_string(),
                    unbonding_period: None,
                }))?;

        assert_eq!(stake_response.amounts, vec![Uint128::zero()]);

        // Ensure user got all of his lp back
        let lp_balance = self.query_proxy_balance(&proxy_addr, &lp_asset)?.u128();
        assert_eq!(lp_balance, stake_value * 2);

        Ok(())
    }

    pub fn test_claim(&self) -> anyhow::Result<()> {
        let (ans_stake_token, _) = self.provider.stake_token();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<CwStakingAdapter<Chain>>()?
            .build()?;
        let proxy_addr = new_account.proxy()?;

        let stake_value = 1_000_000_000u128;
        let reward_value = 10_000_000u128;
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

        self.provider.generate_rewards(&proxy_addr, reward_value)?;

        self.staking_adapter.execute(
            &crate::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: Some(proxy_addr.to_string()),
                request: StakingExecuteMsg {
                    provider: self.provider.name(),
                    action: StakingAction::ClaimRewards {
                        assets: vec![AssetEntry::new(&ans_stake_token)],
                    },
                },
            }),
            None,
        )?;
        let reward = self
            .query_proxy_balance(&proxy_addr, &self.provider.reward_asset())?
            .u128();
        assert!(reward >= reward_value);

        Ok(())
    }

    pub fn test_staking_info(&self) -> anyhow::Result<()> {
        let (ans_stake_token, asset_info_stake_token) = self.provider.stake_token();

        let info_response: StakingInfoResponse =
            self.staking_adapter
                .query(&crate::msg::QueryMsg::Module(StakingQueryMsg::Info {
                    provider: self.provider.name(),
                    staking_tokens: vec![AssetEntry::new(&ans_stake_token)],
                }))?;
        let info = info_response.infos[0].clone();
        assert_eq!(
            AssetInfoUnchecked::from(info.staking_token),
            asset_info_stake_token
        );
        assert_eq!(info.staking_target, self.provider.staking_target());

        Ok(())
    }

    pub fn test_query_rewards(&self) -> anyhow::Result<()> {
        let (ans_stake_token, _) = self.provider.stake_token();

        let new_account = self
            .abstr_deployment
            .account_builder()
            .install_adapter::<CwStakingAdapter<Chain>>()?
            .build()?;
        let proxy_addr = new_account.proxy()?;

        // In case it's mock incentive need to generate and stake it first
        let stake_value = 1_000_000_000u128;
        self.provider.mint_lp(&proxy_addr, stake_value)?;
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
        self.provider
            .generate_rewards(&proxy_addr, 10_000_000u128)?;

        let rewards_respone: RewardTokensResponse = self.staking_adapter.query(
            &crate::msg::QueryMsg::Module(StakingQueryMsg::RewardTokens {
                provider: self.provider.name(),
                staking_tokens: vec![AssetEntry::new(&ans_stake_token)],
            }),
        )?;
        rewards_respone.tokens.iter().flatten().find(|&asset_info| {
            self.provider.reward_asset() == AssetInfoUnchecked::from(asset_info)
        });

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
