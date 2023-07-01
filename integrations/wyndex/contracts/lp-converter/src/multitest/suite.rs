// needed contracts:
// - hub
// - pair + stake

use std::collections::HashMap;

use anyhow::Result as AnyResult;

use cosmwasm_std::{
    testing::mock_env, to_binary, Addr, Coin, Decimal, Empty, StdResult, Uint128, Validator,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor, StakingInfo};
use wynd_lsd_hub::msg::{ConfigResponse, TokenInitInfo};
use wyndex::{
    asset::{Asset, AssetInfo, AssetInfoExt},
    factory::{
        DefaultStakeConfig, ExecuteMsg as FactoryExecuteMsg, PairConfig, PairType,
        PartialStakeConfig, QueryMsg as FactoryQueryMsg,
    },
    fee_config::FeeConfig,
    pair::{ExecuteMsg as PairExecuteMsg, PairInfo},
    stake::{ConverterConfig, ReceiveMsg, UnbondingPeriod},
};
use wyndex_stake::msg::{ExecuteMsg as StakeExecuteMsg, StakedResponse};

pub const DAY: u64 = 24 * HOUR;
pub const HOUR: u64 = 60 * 60;

fn contract_factory() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        wyndex_factory::contract::execute,
        wyndex_factory::contract::instantiate,
        wyndex_factory::contract::query,
    )
    .with_reply_empty(wyndex_factory::contract::reply);

    Box::new(contract)
}

fn contract_pair() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        wyndex_pair::contract::execute,
        wyndex_pair::contract::instantiate,
        wyndex_pair::contract::query,
    )
    .with_reply_empty(wyndex_pair::contract::reply);

    Box::new(contract)
}

fn contract_stake() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        wyndex_stake::contract::execute,
        wyndex_stake::contract::instantiate,
        wyndex_stake::contract::query,
    )
    .with_migrate(wyndex_stake::contract::migrate);

    Box::new(contract)
}

fn contract_token() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );

    Box::new(contract)
}

fn contract_lsd_hub() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        wynd_lsd_hub::contract::execute,
        wynd_lsd_hub::contract::instantiate,
        wynd_lsd_hub::contract::query,
    )
    .with_reply_empty(wynd_lsd_hub::contract::reply);

    Box::new(contract)
}

fn contract_converter() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply_empty(crate::contract::reply);

    Box::new(contract)
}

pub(super) fn juno(amount: u128) -> Asset {
    native_asset(amount, "ujuno")
}

pub(super) fn uusd(amount: u128) -> Asset {
    native_asset(amount, "uusd")
}

pub(super) fn native_asset(amount: u128, denom: &str) -> Asset {
    AssetInfo::Native(denom.to_string()).with_balance(amount)
}

#[derive(Debug)]
pub struct SuiteBuilder {
    pub unbonding_periods: Vec<UnbondingPeriod>,
    pub admin: Option<String>,
    pub native_balances: Vec<(Addr, Coin)>,
    pub no_converter: bool,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            unbonding_periods: vec![7 * DAY, 14 * DAY],
            admin: None,
            native_balances: vec![],
            no_converter: false,
        }
    }

    pub fn without_converter(mut self) -> Self {
        self.no_converter = true;
        self
    }

    pub fn with_native_balances(mut self, denom: &str, balances: Vec<(&str, u128)>) -> Self {
        self.native_balances
            .extend(balances.into_iter().map(|(addr, amount)| {
                (
                    Addr::unchecked(addr),
                    Coin {
                        denom: denom.to_owned(),
                        amount: amount.into(),
                    },
                )
            }));
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");
        // provide initial native balances
        app.init_modules(|router, api, storage| {
            // group by address
            let mut balances = HashMap::<Addr, Vec<Coin>>::new();
            for (addr, coin) in self.native_balances {
                let addr_balance = balances.entry(addr).or_default();
                addr_balance.push(coin);
            }

            for (addr, coins) in balances {
                router
                    .bank
                    .init_balance(storage, &addr, coins)
                    .expect("init balance");
            }

            router
                .staking
                .setup(
                    storage,
                    StakingInfo {
                        bonded_denom: "ujuno".to_string(),
                        unbonding_time: 28 * DAY,
                        apr: Decimal::percent(35),
                    },
                )
                .unwrap();

            // register validator for hub
            router
                .staking
                .add_validator(
                    api,
                    storage,
                    &mock_env().block,
                    Validator {
                        address: "testvaloper1".to_string(),
                        commission: Decimal::percent(5),
                        max_commission: Decimal::one(),
                        max_change_rate: Decimal::one(),
                    },
                )
                .unwrap();
        });

        // create factory contract
        let pair_code_id = app.store_code(contract_pair());
        let staking_code_id = app.store_code(contract_stake());
        let token_code_id = app.store_code(contract_token());
        let factory_code_id = app.store_code(contract_factory());
        let factory = app
            .instantiate_contract(
                factory_code_id,
                owner.clone(),
                &wyndex::factory::InstantiateMsg {
                    pair_configs: vec![PairConfig {
                        code_id: pair_code_id,
                        pair_type: PairType::Xyk {},
                        fee_config: FeeConfig {
                            total_fee_bps: 100,
                            protocol_fee_bps: 10,
                        },
                        is_disabled: false,
                    }],
                    token_code_id,
                    fee_address: None,
                    owner: owner.to_string(),
                    max_referral_commission: Decimal::one(),
                    default_stake_config: DefaultStakeConfig {
                        staking_code_id,
                        tokens_per_power: Uint128::new(1000),
                        min_bond: Uint128::new(1000),
                        unbonding_periods: self.unbonding_periods,
                        max_distributions: 6,
                        converter: None,
                    },
                    trading_starts: None,
                },
                &[],
                String::from("ASTRO"),
                None,
            )
            .unwrap();

        // create hub contract
        let lsd_hub_id = app.store_code(contract_lsd_hub());
        let lsd_hub = app
            .instantiate_contract(
                lsd_hub_id,
                owner.clone(),
                &wynd_lsd_hub::msg::InstantiateMsg {
                    treasury: "treasury".to_string(),
                    commission: Decimal::percent(1),
                    validators: vec![("testvaloper1".to_string(), Decimal::percent(100))],
                    owner: owner.to_string(),

                    epoch_period: 23 * HOUR,
                    unbond_period: 28 * DAY,
                    max_concurrent_unbondings: 7,
                    cw20_init: TokenInitInfo {
                        label: "label".to_string(),
                        cw20_code_id: token_code_id,
                        name: "wyJuno".to_string(),
                        symbol: "wyJUNO".to_string(),
                        decimals: 6,
                        initial_balances: vec![],
                        marketing: None,
                    },
                    liquidity_discount: Decimal::percent(4),
                    slashing_safety_margin: 600,
                    tombstone_treshold: Decimal::percent(30u64),
                },
                &[],
                "hub",
                Some(owner.to_string()),
            )
            .unwrap();

        let lsd_token = app
            .wrap()
            .query_wasm_smart::<ConfigResponse>(&lsd_hub, &wynd_lsd_hub::msg::QueryMsg::Config {})
            .unwrap()
            .token_contract;

        // create converter contract
        let converter_code_id = app.store_code(contract_converter());
        let converter = app
            .instantiate_contract(
                converter_code_id,
                owner.clone(),
                &crate::msg::InstantiateMsg {
                    hub: lsd_hub.to_string(),
                },
                &[],
                String::from("ASTRO"),
                None,
            )
            .unwrap();

        // create USD-wyAsset pair
        let lsd_pair_assets = vec![
            AssetInfo::Token(lsd_token.to_string()),
            AssetInfo::Native("uusd".to_owned()),
        ];
        app.execute_contract(
            owner.clone(),
            factory.clone(),
            &FactoryExecuteMsg::CreatePair {
                pair_type: PairType::Xyk {},
                asset_infos: lsd_pair_assets.clone(),
                staking_config: PartialStakeConfig::default(),
                init_params: None,
                total_fee_bps: None,
            },
            &[],
        )
        .unwrap();
        let pair_info = app
            .wrap()
            .query_wasm_smart::<PairInfo>(
                Addr::unchecked(&factory),
                &FactoryQueryMsg::Pair {
                    asset_infos: lsd_pair_assets,
                },
            )
            .unwrap();
        let lsd_pair = pair_info.contract_addr;
        let lsd_staking = pair_info.staking_addr;

        // create USD-Asset pair
        let native_pair_assets = vec![
            AssetInfo::Native("ujuno".to_owned()),
            AssetInfo::Native("uusd".to_owned()),
        ];
        app.execute_contract(
            owner,
            factory.clone(),
            &FactoryExecuteMsg::CreatePair {
                pair_type: PairType::Xyk {},
                asset_infos: native_pair_assets.clone(),
                staking_config: PartialStakeConfig {
                    converter: (!self.no_converter).then_some(ConverterConfig {
                        contract: converter.to_string(),
                        pair_to: lsd_pair.to_string(),
                    }),
                    ..Default::default()
                },
                init_params: None,
                total_fee_bps: None,
            },
            &[],
        )
        .unwrap();
        let pair_info = app
            .wrap()
            .query_wasm_smart::<PairInfo>(
                Addr::unchecked(&factory),
                &FactoryQueryMsg::Pair {
                    asset_infos: native_pair_assets,
                },
            )
            .unwrap();
        let native_pair = pair_info.contract_addr;
        let native_staking = pair_info.staking_addr;

        Suite {
            app,

            staking_code_id,

            converter,
            factory,
            native_pair,
            native_staking,
            lsd_pair,
            lsd_staking,
            lsd_hub,
            lsd_token,
        }
    }
}

pub struct Suite {
    pub app: App,

    staking_code_id: u64,
    pub converter: Addr,
    factory: Addr,
    pub native_pair: Addr,
    native_staking: Addr,
    pub lsd_pair: Addr,
    lsd_staking: Addr,
    lsd_hub: Addr,
    pub lsd_token: Addr,
}

#[derive(Copy, Clone)]
pub enum Pair {
    Native,
    Lsd,
}

impl Pair {
    pub fn addr(self, suite: &Suite) -> Addr {
        match self {
            Pair::Native => suite.native_pair.clone(),
            Pair::Lsd => suite.lsd_pair.clone(),
        }
    }

    pub fn staking_addr(self, suite: &Suite) -> Addr {
        match self {
            Pair::Native => suite.native_staking.clone(),
            Pair::Lsd => suite.lsd_staking.clone(),
        }
    }
}

impl Suite {
    pub fn lsd_asset(&self, amount: u128) -> Asset {
        Asset {
            info: AssetInfo::Token(self.lsd_token.to_string()),
            amount: Uint128::from(amount),
        }
    }

    pub fn bond_juno(&mut self, addr: &str, amount: u128) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(addr),
            self.lsd_hub.clone(),
            &wynd_lsd_hub::msg::ExecuteMsg::Bond {},
            &[Coin {
                denom: "ujuno".to_string(),
                amount: amount.into(),
            }],
        )
    }

    pub fn stake_lp(
        &mut self,
        pair: Pair,
        sender: &str,
        amount: u128,
        unbonding_period: u64,
    ) -> AnyResult<AppResponse> {
        let pair_info = self.query_pair_info(pair)?;
        // send LP tokens to staking contract
        self.app.execute_contract(
            Addr::unchecked(sender),
            pair_info.liquidity_token,
            &Cw20ExecuteMsg::Send {
                contract: pair.staking_addr(self).to_string(),
                amount: amount.into(),
                msg: to_binary(&ReceiveMsg::Delegate {
                    unbonding_period,
                    delegate_as: None,
                })?,
            },
            &[],
        )
    }

    /// Provides some liquidity to the given pair.
    pub fn provide_liquidity(
        &mut self,
        provider: &str,
        first_asset: Asset,
        second_asset: Asset,
    ) -> AnyResult<u128> {
        let pair = self.query_pair(vec![first_asset.info.clone(), second_asset.info.clone()])?;

        let prev_balance = self.query_cw20_balance(provider, pair.liquidity_token.as_str())?;

        let mut native_tokens = vec![];

        match &first_asset.info {
            AssetInfo::Token(addr) => {
                // increases allowances for given LP contracts in order to provide liquidity to pool
                self.increase_allowance(
                    provider,
                    &Addr::unchecked(addr),
                    pair.contract_addr.as_str(),
                    first_asset.amount.u128(),
                )?;
            }
            AssetInfo::Native(denom) => {
                native_tokens.push(Coin {
                    amount: first_asset.amount,
                    denom: denom.to_owned(),
                });
            }
        };
        match &second_asset.info {
            AssetInfo::Token(addr) => {
                // increases allowances for given LP contracts in order to provide liquidity to pool
                self.increase_allowance(
                    provider,
                    &Addr::unchecked(addr),
                    pair.contract_addr.as_str(),
                    second_asset.amount.u128(),
                )?;
            }
            AssetInfo::Native(denom) => {
                native_tokens.push(Coin {
                    amount: second_asset.amount,
                    denom: denom.to_owned(),
                });
            }
        };

        self.app.execute_contract(
            Addr::unchecked(provider),
            pair.contract_addr,
            &PairExecuteMsg::ProvideLiquidity {
                assets: vec![first_asset, second_asset],
                slippage_tolerance: None,
                receiver: None,
            },
            &native_tokens,
        )?;

        let new_balance = self.query_cw20_balance(provider, pair.liquidity_token.as_str())?;

        Ok(new_balance - prev_balance)
    }

    pub fn increase_allowance(
        &mut self,
        owner: &str,
        contract: &Addr,
        spender: &str,
        amount: u128,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(owner),
            contract.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: spender.to_owned(),
                amount: amount.into(),
                expires: None,
            },
            &[],
        )
    }

    /// Migrates the staked LP tokens from the native pair's staking contract to
    /// the lsd pair's staking contract
    pub fn migrate_stake(
        &mut self,
        pair: Pair,
        sender: &str,
        amount: u128,
        unbonding_period: u64,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            pair.staking_addr(self),
            &StakeExecuteMsg::MigrateStake {
                amount: Uint128::from(amount),
                unbonding_period,
            },
            &[],
        )
    }

    pub fn migrate_staking_contract(
        &mut self,
        pair: Pair,
        msg: wyndex_stake::msg::MigrateMsg,
    ) -> AnyResult<AppResponse> {
        self.app.migrate_contract(
            Addr::unchecked("owner"),
            pair.staking_addr(self),
            &msg,
            self.staking_code_id, // same code id
        )
    }

    pub fn query_stake(
        &self,
        pair: Pair,
        addr: &str,
        unbonding_period: u64,
    ) -> AnyResult<StakedResponse> {
        Ok(self.app.wrap().query_wasm_smart(
            pair.staking_addr(self),
            &wyndex_stake::msg::QueryMsg::Staked {
                address: addr.to_string(),
                unbonding_period,
            },
        )?)
    }

    pub fn query_pair(&self, asset_infos: Vec<AssetInfo>) -> AnyResult<PairInfo> {
        Ok(self
            .app
            .wrap()
            .query_wasm_smart(&self.factory, &FactoryQueryMsg::Pair { asset_infos })?)
    }

    pub fn query_pair_info(&self, pair: Pair) -> AnyResult<PairInfo> {
        Ok(self
            .app
            .wrap()
            .query_wasm_smart(pair.addr(self), &wyndex::pair::QueryMsg::Pair {})?)
    }

    pub fn query_cw20_balance(&self, address: &str, cw20: impl Into<String>) -> StdResult<u128> {
        let balance: BalanceResponse = self.app.wrap().query_wasm_smart(
            cw20,
            &Cw20QueryMsg::Balance {
                address: address.to_owned(),
            },
        )?;
        Ok(balance.balance.u128())
    }
}
