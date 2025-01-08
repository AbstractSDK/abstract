//! adapted from https://github.com/cosmorama/wynddex/blob/main/tests/src/suite.rs to suite our needs
use std::cell::RefMut;

use anyhow::Result as AnyResult;
use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{coin, to_json_binary, Decimal, Uint128};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_orch::{mock::MockAppBech32, prelude::*};

use cw_controllers::{Claim, ClaimsResponse};
use cw_orch::mock::cw_multi_test::{AppResponse, BankSudo, Executor, SudoMsg};
use wyndex::{
    asset::{Asset, AssetInfo, AssetInfoValidated},
    factory::{
        DefaultStakeConfig, DistributionFlow, ExecuteMsg as FactoryExecuteMsg,
        InstantiateMsg as FactoryInstantiateMsg, PairConfig, PairType, PartialStakeConfig,
        QueryMsg as FactoryQueryMsg,
    },
    fee_config::FeeConfig,
    pair::{ExecuteMsg as PairExecuteMsg, PairInfo},
    stake::UnbondingPeriod,
};
use wyndex_multi_hop::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, SimulateSwapOperationsResponse, SwapOperation,
};
use wyndex_stake::msg::{
    AllStakedResponse, AnnualizedReward, AnnualizedRewardsResponse, BondingInfoResponse,
    BondingPeriodInfo, ExecuteMsg as StakeExecuteMsg, QueryMsg as StakeQueryMsg,
    RewardsPowerResponse, StakedResponse, TotalStakedResponse,
};

use crate::{MULTI_HOP, POOL_FACTORY, WYNDEX_OWNER};
pub const SEVEN_DAYS: u64 = 604800;

fn store_multi_hop(module: &mut MockAppBech32) -> u64 {
    let contract = Box::new(ContractWrapper::new_with_empty(
        wyndex_multi_hop::contract::execute,
        wyndex_multi_hop::contract::instantiate,
        wyndex_multi_hop::contract::query,
    ));

    module.store_code(contract)
}

fn store_factory(module: &mut MockAppBech32) -> u64 {
    let contract = Box::new(
        ContractWrapper::new_with_empty(
            wyndex_factory::contract::execute,
            wyndex_factory::contract::instantiate,
            wyndex_factory::contract::query,
        )
        .with_reply_empty(wyndex_factory::contract::reply),
    );

    module.store_code(contract)
}

fn store_pair(module: &mut MockAppBech32) -> u64 {
    let contract = Box::new(
        ContractWrapper::new_with_empty(
            wyndex_pair::contract::execute,
            wyndex_pair::contract::instantiate,
            wyndex_pair::contract::query,
        )
        .with_reply_empty(wyndex_pair::contract::reply),
    );

    module.store_code(contract)
}

fn store_staking(module: &mut MockAppBech32) -> u64 {
    let contract = Box::new(ContractWrapper::new(
        wyndex_stake::contract::execute,
        wyndex_stake::contract::instantiate,
        wyndex_stake::contract::query,
    ));

    module.store_code(contract)
}

fn store_cw20(module: &mut MockAppBech32) -> u64 {
    let contract = Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));

    module.store_code(contract)
}

#[derive(Debug)]
pub struct SuiteBuilder {
    max_referral_commission: Decimal,
    stake_config: DefaultStakeConfig,
    trading_starts: Option<u64>,
}

impl Default for SuiteBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            max_referral_commission: Decimal::one(),
            stake_config: DefaultStakeConfig {
                staking_code_id: 0, // will be set in build()
                tokens_per_power: Uint128::new(1000),
                min_bond: Uint128::new(1000),
                unbonding_periods: vec![60 * 60 * 24 * 7, 60 * 60 * 24 * 14, 60 * 60 * 24 * 21],
                max_distributions: 6,
                converter: None,
            },
            trading_starts: None,
        }
    }

    pub fn with_stake_config(mut self, stake_config: DefaultStakeConfig) -> Self {
        self.stake_config = stake_config;
        self
    }

    pub fn with_trading_starts(mut self, trading_starts: u64) -> Self {
        self.trading_starts = Some(trading_starts);
        self
    }

    pub fn with_max_referral_commission(mut self, max: Decimal) -> Self {
        self.max_referral_commission = max;
        self
    }

    #[track_caller]
    pub fn build(self, mock_chain: &MockBech32) -> Suite {
        let owner = mock_chain.addr_make(WYNDEX_OWNER);
        let mut app = mock_chain.app.borrow_mut();

        let cw20_code_id = store_cw20(&mut app);
        let pair_code_id = store_pair(&mut app);
        let staking_code_id = store_staking(&mut app);
        let factory_code_id = store_factory(&mut app);
        let factory = app
            .instantiate_contract(
                factory_code_id,
                owner.clone(),
                &FactoryInstantiateMsg {
                    pair_configs: vec![
                        PairConfig {
                            code_id: pair_code_id,
                            pair_type: PairType::Xyk {},
                            fee_config: FeeConfig {
                                total_fee_bps: 0,
                                protocol_fee_bps: 0,
                            },
                            is_disabled: false,
                        },
                        PairConfig {
                            code_id: pair_code_id,
                            pair_type: PairType::Stable {},
                            fee_config: FeeConfig {
                                total_fee_bps: 0,
                                protocol_fee_bps: 0,
                            },
                            is_disabled: false,
                        },
                    ],
                    token_code_id: cw20_code_id,
                    fee_address: None,
                    owner: owner.to_string(),
                    max_referral_commission: self.max_referral_commission,
                    default_stake_config: DefaultStakeConfig {
                        staking_code_id,
                        ..self.stake_config
                    },
                    trading_starts: self.trading_starts,
                },
                &[],
                "Wyndex Factory",
                None,
            )
            .unwrap();

        let multi_hop_code_id = store_multi_hop(&mut app);
        let multi_hop = app
            .instantiate_contract(
                multi_hop_code_id,
                owner.clone(),
                &InstantiateMsg {
                    wyndex_factory: factory.to_string(),
                },
                &[],
                "Wyndex Multi Hop",
                None,
            )
            .unwrap();

        drop(app);
        Suite {
            mock: mock_chain.clone(),
            owner,
            factory,
            multi_hop,
        }
    }
}

pub struct Suite {
    pub owner: Addr,
    pub factory: Addr,
    mock: MockBech32,
    pub multi_hop: Addr,
}

impl PartialEq for Suite {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner
            && self.factory == other.factory
            && self.multi_hop == other.multi_hop
    }
}

impl Suite {
    pub fn load_from(mock: &MockBech32) -> Self {
        let owner = mock.addr_make(WYNDEX_OWNER);
        let factory = mock.state.get_address(POOL_FACTORY).unwrap();
        let multi_hop = mock.state.get_address(MULTI_HOP).unwrap();

        Self {
            mock: mock.clone(),
            owner,
            factory,
            multi_hop,
        }
    }

    pub fn app(&self) -> RefMut<MockAppBech32> {
        self.mock.app.borrow_mut()
    }
    pub fn advance_time(&mut self, seconds: u64) {
        self.app()
            .update_block(|block| block.time = block.time.plus_seconds(seconds));
    }

    fn unbonding_period_or_default(&self, unbonding_period: impl Into<Option<u64>>) -> u64 {
        // Use default SEVEN_DAYS unbonding period if none provided
        if let Some(up) = unbonding_period.into() {
            up
        } else {
            SEVEN_DAYS
        }
    }

    pub fn create_pair(
        &mut self,
        sender: &Addr,
        pair_type: PairType,
        tokens: [AssetInfo; 2],
        staking_config: Option<PartialStakeConfig>,
        total_fee_bps: Option<u16>,
    ) -> AnyResult<Addr> {
        self.app().execute_contract(
            Addr::unchecked(sender),
            self.factory.clone(),
            &FactoryExecuteMsg::CreatePair {
                pair_type,
                asset_infos: tokens.to_vec(),
                init_params: None,
                staking_config: staking_config.unwrap_or_default(),
                total_fee_bps,
            },
            &[],
        )?;

        let factory = self.factory.clone();
        let res: PairInfo = self.app().wrap().query_wasm_smart(
            Addr::unchecked(factory),
            &FactoryQueryMsg::Pair {
                asset_infos: tokens.to_vec(),
            },
        )?;
        Ok(res.contract_addr)
    }

    pub fn create_pair_and_distributions(
        &mut self,
        sender: &Addr,
        pair_type: PairType,
        asset_infos: Vec<AssetInfo>,
        staking_config: Option<PartialStakeConfig>,
        distribution_flows: Vec<DistributionFlow>,
    ) -> AnyResult<AppResponse> {
        self.app().execute_contract(
            Addr::unchecked(sender),
            self.factory.clone(),
            &FactoryExecuteMsg::CreatePairAndDistributionFlows {
                pair_type,
                asset_infos,
                init_params: None,
                staking_config: staking_config.unwrap_or_default(),
                distribution_flows,
                total_fee_bps: None,
            },
            &[],
        )
    }

    pub fn provide_liquidity(
        &mut self,
        owner: &Addr,
        pair: &Addr,
        assets: [Asset; 2],
        send_funds: &[Coin],
    ) -> AnyResult<AppResponse> {
        self.app().execute_contract(
            owner.clone(),
            pair.clone(),
            &PairExecuteMsg::ProvideLiquidity {
                assets: assets.to_vec(),
                slippage_tolerance: None,
                receiver: None,
            },
            send_funds,
        )
    }

    fn increase_allowance(
        &mut self,
        owner: &Addr,
        contract: &Addr,
        spender: &Addr,
        amount: u128,
    ) -> AnyResult<AppResponse> {
        self.app().execute_contract(
            owner.clone(),
            contract.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: spender.to_string(),
                amount: amount.into(),
                expires: None,
            },
            &[],
        )
    }

    /// Create LP for provided assets and provides some liquidity to them.
    /// Requirement: if using native token provide coins to sent as last argument
    pub fn create_pair_and_provide_liquidity(
        &mut self,
        pair_type: PairType,
        first_asset: (AssetInfo, u128),
        second_asset: (AssetInfo, u128),
        native_tokens: Vec<Coin>,
    ) -> AnyResult<Addr> {
        let owner = self.owner.clone();
        let whale = self.app().api().addr_make("whale");

        let pair = self.create_pair(
            &owner,
            pair_type,
            [first_asset.0.clone(), second_asset.0.clone()],
            None,
            None,
        )?;

        match first_asset.0.clone() {
            AssetInfo::Token(addr) => {
                // Mint some initial balances for whale user
                self.mint_cw20(&owner, &Addr::unchecked(&addr), first_asset.1, &whale)
                    .unwrap();
                // Increases allowances for given LP contracts in order to provide liquidity to pool
                self.increase_allowance(&whale, &Addr::unchecked(addr), &pair, first_asset.1)
                    .unwrap();
            }
            AssetInfo::Native(denom) => {
                self.app()
                    .sudo(SudoMsg::Bank(BankSudo::Mint {
                        to_address: whale.to_string(),
                        amount: vec![coin(first_asset.1, denom)],
                    }))
                    .unwrap();
            }
        };
        match second_asset.0.clone() {
            AssetInfo::Token(addr) => {
                // Mint some initial balances for whale user
                self.mint_cw20(&owner, &Addr::unchecked(&addr), second_asset.1, &whale)
                    .unwrap();
                // Increases allowances for given LP contracts in order to provide liquidity to pool
                self.increase_allowance(&whale, &Addr::unchecked(addr), &pair, second_asset.1)
                    .unwrap();
            }
            AssetInfo::Native(denom) => {
                self.app()
                    .sudo(SudoMsg::Bank(BankSudo::Mint {
                        to_address: whale.to_string(),
                        amount: vec![coin(second_asset.1, denom)],
                    }))
                    .unwrap();
            }
        };

        self.provide_liquidity(
            &whale,
            &pair,
            [
                Asset {
                    info: first_asset.0,
                    amount: first_asset.1.into(),
                },
                Asset {
                    info: second_asset.0,
                    amount: second_asset.1.into(),
                },
            ],
            &native_tokens, // for native token you need to transfer tokens manually
        )
        .unwrap();

        Ok(pair)
    }

    /// Create a distribution flow through the factory contract
    pub fn create_distribution_flow(
        &mut self,
        sender: &Addr,
        asset_infos: Vec<AssetInfo>,
        asset: AssetInfo,
        rewards: Vec<(UnbondingPeriod, Decimal)>,
    ) -> AnyResult<AppResponse> {
        self.app().execute_contract(
            Addr::unchecked(sender),
            self.factory.clone(),
            &FactoryExecuteMsg::CreateDistributionFlow {
                asset_infos,
                asset,
                rewards,
            },
            &[],
        )
    }

    pub fn distribute_funds(
        &mut self,
        staking_contract: Addr,
        sender: &Addr,
        funds: &[Coin],
    ) -> AnyResult<AppResponse> {
        self.app().execute_contract(
            Addr::unchecked(sender),
            staking_contract,
            &StakeExecuteMsg::DistributeRewards { sender: None },
            funds,
        )
    }

    pub fn mint_cw20(
        &mut self,
        owner: &Addr,
        token: &Addr,
        amount: u128,
        recipient: &Addr,
    ) -> AnyResult<AppResponse> {
        self.app().execute_contract(
            owner.clone(),
            token.clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: recipient.to_string(),
                amount: amount.into(),
            },
            &[],
        )
    }

    pub fn send_cw20(
        &mut self,
        owner: &Addr,
        token: &Addr,
        amount: u128,
        contract: &Addr,
        msg: impl Serialize,
    ) -> AnyResult<AppResponse> {
        self.app().execute_contract(
            owner.clone(),
            token.clone(),
            &Cw20ExecuteMsg::Send {
                contract: contract.to_string(),
                amount: amount.into(),
                msg: to_json_binary(&msg)?,
            },
            &[],
        )
    }

    pub fn swap_operations(
        &mut self,
        sender: &Addr,
        amount: Coin,
        operations: Vec<SwapOperation>,
    ) -> AnyResult<AppResponse> {
        self.swap_operations_ref(sender, amount, operations, None, None)
    }

    pub fn swap_operations_ref(
        &mut self,
        sender: &Addr,
        amount: Coin,
        operations: Vec<SwapOperation>,
        referral_address: impl Into<Option<String>>,
        referral_commission: impl Into<Option<Decimal>>,
    ) -> AnyResult<AppResponse> {
        self.app().execute_contract(
            Addr::unchecked(sender),
            self.multi_hop.clone(),
            &ExecuteMsg::ExecuteSwapOperations {
                operations,
                minimum_receive: None,
                receiver: None,
                max_spread: None,
                referral_address: referral_address.into(),
                referral_commission: referral_commission.into(),
            },
            &[amount],
        )
    }

    pub fn swap_operations_cw20(
        &mut self,
        sender: &Addr,
        token_in: &Addr,
        amount: u128,
        operations: Vec<SwapOperation>,
    ) -> AnyResult<AppResponse> {
        self.swap_operations_cw20_ref(sender, token_in, amount, operations, None, None)
    }

    pub fn swap_operations_cw20_ref(
        &mut self,
        sender: &Addr,
        token_in: &Addr,
        amount: u128,
        operations: Vec<SwapOperation>,
        referral_address: impl Into<Option<String>>,
        referral_commission: impl Into<Option<Decimal>>,
    ) -> AnyResult<AppResponse> {
        self.app().execute_contract(
            Addr::unchecked(sender),
            token_in.clone(),
            &Cw20ExecuteMsg::Send {
                contract: self.multi_hop.to_string(),
                amount: amount.into(),
                msg: to_json_binary(&ExecuteMsg::ExecuteSwapOperations {
                    operations,
                    minimum_receive: None,
                    receiver: None,
                    max_spread: None,
                    referral_address: referral_address.into(),
                    referral_commission: referral_commission.into(),
                })
                .unwrap(),
            },
            &[],
        )
    }

    pub fn assert_minimum_receive(
        &mut self,
        receiver: &Addr,
        asset_info: AssetInfo,
        minimum_receive: impl Into<Uint128>,
    ) -> AnyResult<AppResponse> {
        self.app().execute_contract(
            Addr::unchecked(receiver),
            self.multi_hop.clone(),
            &ExecuteMsg::AssertMinimumReceive {
                asset_info,
                prev_balance: Uint128::zero(),
                minimum_receive: minimum_receive.into(),
                receiver: receiver.into(),
            },
            &[],
        )
    }

    pub fn query_balance(&self, sender: &Addr, denom: &str) -> AnyResult<u128> {
        let amount = self
            .app()
            .wrap()
            .query_balance(Addr::unchecked(sender), denom)?
            .amount;
        Ok(amount.into())
    }

    pub fn query_cw20_balance(&self, sender: &Addr, address: &Addr) -> AnyResult<u128> {
        let balance: BalanceResponse = self.app().wrap().query_wasm_smart(
            address,
            &Cw20QueryMsg::Balance {
                address: sender.to_string(),
            },
        )?;
        Ok(balance.balance.into())
    }

    pub fn query_simulate_swap_operations(
        &self,
        offer_amount: impl Into<Uint128>,
        operations: Vec<SwapOperation>,
    ) -> AnyResult<u128> {
        let amount: SimulateSwapOperationsResponse = self.app().wrap().query_wasm_smart(
            self.multi_hop.clone(),
            &QueryMsg::SimulateSwapOperations {
                offer_amount: offer_amount.into(),
                operations,
                referral: false,
                referral_commission: None,
            },
        )?;
        Ok(amount.amount.into())
    }

    pub fn query_simulate_swap_operations_ref(
        &self,
        offer_amount: impl Into<Uint128>,
        operations: Vec<SwapOperation>,
        referral_commission: impl Into<Option<Decimal>>,
    ) -> AnyResult<u128> {
        let amount: SimulateSwapOperationsResponse = self.app().wrap().query_wasm_smart(
            self.multi_hop.clone(),
            &QueryMsg::SimulateSwapOperations {
                offer_amount: offer_amount.into(),
                operations,
                referral: true,
                referral_commission: referral_commission.into(),
            },
        )?;
        Ok(amount.amount.into())
    }

    /// Queries the info of the given pair from the factory
    pub fn query_pair(&self, asset_infos: Vec<AssetInfo>) -> AnyResult<PairInfo> {
        Ok(self
            .app()
            .wrap()
            .query_wasm_smart(self.factory.clone(), &FactoryQueryMsg::Pair { asset_infos })?)
    }

    // returns address' balance on staking contract
    pub fn query_balance_staking_contract(&self, asset_infos: Vec<AssetInfo>) -> AnyResult<u128> {
        let pair_info = self.query_pair(asset_infos)?;
        let balance: BalanceResponse = self.app().wrap().query_wasm_smart(
            pair_info.liquidity_token.clone(),
            &Cw20QueryMsg::Balance {
                address: pair_info.staking_addr.to_string(),
            },
        )?;
        Ok(balance.balance.u128())
    }

    pub fn query_all_staked(
        &self,
        asset_infos: Vec<AssetInfo>,
        address: &Addr,
    ) -> AnyResult<AllStakedResponse> {
        let pair_info = self.query_pair(asset_infos)?;
        let staked: AllStakedResponse = self.app().wrap().query_wasm_smart(
            pair_info.staking_addr,
            &StakeQueryMsg::AllStaked {
                address: address.to_string(),
            },
        )?;
        Ok(staked)
    }

    pub fn query_staked(
        &self,
        asset_infos: Vec<AssetInfo>,
        address: &Addr,
        unbonding_period: impl Into<Option<u64>>,
    ) -> AnyResult<u128> {
        let pair_info = self.query_pair(asset_infos)?;
        let staked: StakedResponse = self.app().wrap().query_wasm_smart(
            pair_info.staking_addr,
            &StakeQueryMsg::Staked {
                address: address.to_string(),
                unbonding_period: self.unbonding_period_or_default(unbonding_period),
            },
        )?;
        Ok(staked.stake.u128())
    }

    pub fn query_staked_periods(
        &self,
        asset_infos: Vec<AssetInfo>,
    ) -> AnyResult<Vec<BondingPeriodInfo>> {
        let pair_info = self.query_pair(asset_infos)?;
        let info: BondingInfoResponse = self
            .app()
            .wrap()
            .query_wasm_smart(pair_info.staking_addr, &StakeQueryMsg::BondingInfo {})?;
        Ok(info.bonding)
    }

    pub fn query_total_staked(&self, asset_infos: Vec<AssetInfo>) -> AnyResult<u128> {
        let pair_info = self.query_pair(asset_infos)?;
        let total_staked: TotalStakedResponse = self
            .app()
            .wrap()
            .query_wasm_smart(pair_info.staking_addr, &StakeQueryMsg::TotalStaked {})?;
        Ok(total_staked.total_staked.u128())
    }

    pub fn query_claims(
        &self,
        asset_infos: Vec<AssetInfo>,
        address: &Addr,
    ) -> AnyResult<Vec<Claim>> {
        let pair_info = self.query_pair(asset_infos)?;
        let claims: ClaimsResponse = self.app().wrap().query_wasm_smart(
            pair_info.staking_addr,
            &StakeQueryMsg::Claims {
                address: address.to_string(),
            },
        )?;
        Ok(claims.claims)
    }

    // TODO: fix
    pub fn query_annualized_rewards(
        &self,
        asset_infos: Vec<AssetInfo>,
    ) -> AnyResult<Vec<(UnbondingPeriod, Vec<AnnualizedReward>)>> {
        let pair_info = self.query_pair(asset_infos)?;
        let apr: AnnualizedRewardsResponse = self
            .app()
            .wrap()
            .query_wasm_smart(pair_info.staking_addr, &StakeQueryMsg::AnnualizedRewards {})?;
        Ok(apr.rewards)
    }

    pub fn query_rewards_power(
        &self,
        asset_infos: Vec<AssetInfo>,
        address: &Addr,
    ) -> AnyResult<Vec<(AssetInfoValidated, u128)>> {
        let pair_info = self.query_pair(asset_infos)?;
        let rewards: RewardsPowerResponse = self.app().wrap().query_wasm_smart(
            pair_info.staking_addr,
            &StakeQueryMsg::RewardsPower {
                address: address.to_string(),
            },
        )?;

        Ok(rewards
            .rewards
            .into_iter()
            .map(|(a, p)| (a, p.u128()))
            .filter(|(_, p)| *p > 0)
            .collect())
    }

    pub fn query_total_rewards_power(
        &self,
        asset_infos: Vec<AssetInfo>,
    ) -> AnyResult<Vec<(AssetInfoValidated, u128)>> {
        let pair_info = self.query_pair(asset_infos)?;
        let rewards: RewardsPowerResponse = self
            .app()
            .wrap()
            .query_wasm_smart(pair_info.staking_addr, &StakeQueryMsg::TotalRewardsPower {})?;

        Ok(rewards
            .rewards
            .into_iter()
            .map(|(a, p)| (a, p.u128()))
            .filter(|(_, p)| *p > 0)
            .collect())
    }
}
