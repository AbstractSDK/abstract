use std::collections::HashMap;

use anyhow::Result as AnyResult;
use cosmwasm_std::testing::MockApi;
use cosmwasm_std::{coin, to_binary, Addr, Coin, Decimal, Empty, StdResult, Uint128};
use cw20::{BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
use derivative::Derivative;
use itertools::Itertools;

use wyndex::asset::{
    native_asset_info, token_asset_info, Asset, AssetInfo, AssetInfoValidated, AssetValidated,
};
use wyndex::factory::{DefaultStakeConfig, PairConfig, PairType, PartialStakeConfig};
use wyndex::fee_config::FeeConfig;
use wyndex::pair::{
    CumulativePricesResponse, Cw20HookMsg, ExecuteMsg, PairInfo, QueryMsg,
    ReverseSimulationResponse, SimulationResponse, StablePoolParams,
};
use wyndex::querier::NATIVE_TOKEN_PRECISION;
use wyndex_pair_lsd::contract::{execute, instantiate, query, reply};

const INIT_BALANCE: u128 = 1_000_000_000_000;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TestCoin {
    Cw20(String),
    Cw20Precise(String, u8),
    Native(String),
}

impl TestCoin {
    pub fn denom(&self) -> Option<String> {
        match self {
            TestCoin::Native(denom) => Some(denom.clone()),
            _ => None,
        }
    }

    pub fn cw20_init_data(&self) -> Option<(String, u8)> {
        match self {
            TestCoin::Cw20(name) => Some((name.clone(), 6u8)),
            TestCoin::Cw20Precise(name, precision) => Some((name.clone(), *precision)),
            _ => None,
        }
    }

    pub fn native(denom: &str) -> Self {
        Self::Native(denom.to_string())
    }

    pub fn cw20(name: &str) -> Self {
        Self::Cw20(name.to_string())
    }

    pub fn cw20precise(name: &str, precision: u8) -> Self {
        Self::Cw20Precise(name.to_string(), precision)
    }
}

pub fn init_native_coins(test_coins: &[TestCoin]) -> Vec<Coin> {
    test_coins
        .iter()
        .filter_map(|test_coin| match test_coin {
            TestCoin::Native(name) => {
                let init_balance = INIT_BALANCE * 10u128.pow(NATIVE_TOKEN_PRECISION as u32);
                Some(coin(init_balance, name))
            }
            _ => None,
        })
        .collect()
}

fn token_contract() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new_with_empty(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ))
}

fn pair_contract() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_reply_empty(reply))
}

fn factory_contract() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new_with_empty(
            wyndex_factory::contract::execute,
            wyndex_factory::contract::instantiate,
            wyndex_factory::contract::query,
        )
        .with_reply_empty(wyndex_factory::contract::reply),
    )
}

fn stake_contract() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new_with_empty(
        wyndex_stake::contract::execute,
        wyndex_stake::contract::instantiate,
        wyndex_stake::contract::query,
    ))
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Helper {
    #[derivative(Debug = "ignore")]
    pub app: App,
    pub owner: Addr,
    pub assets: HashMap<TestCoin, AssetInfoValidated>,
    pub factory: Addr,
    pub pair_addr: Addr,
    pub lp_token: Addr,
    pub amp: u64,
}

impl Helper {
    pub fn new(
        owner: &Addr,
        test_coins: Vec<TestCoin>,
        amp: u64,
        swap_fee: Option<u16>,
    ) -> AnyResult<Self> {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, owner, init_native_coins(&test_coins))
                .unwrap()
        });

        let mut asset_infos_vec: Vec<_> = test_coins
            .clone()
            .into_iter()
            .filter_map(|coin| Some((coin.clone(), native_asset_info(&coin.denom()?))))
            .collect();

        let token_code_id = app.store_code(token_contract());

        test_coins.into_iter().for_each(|coin| {
            if let Some((name, decimals)) = coin.cw20_init_data() {
                let token_addr = Self::init_token(&mut app, token_code_id, name, decimals, owner);
                asset_infos_vec.push((coin, token_asset_info(token_addr.as_str())))
            }
        });

        let pair_code_id = app.store_code(pair_contract());
        let factory_code_id = app.store_code(factory_contract());
        let staking_code_id = app.store_code(stake_contract());

        let init_msg = wyndex::factory::InstantiateMsg {
            fee_address: None,
            pair_configs: vec![PairConfig {
                code_id: pair_code_id,
                fee_config: FeeConfig {
                    protocol_fee_bps: 5000,
                    total_fee_bps: swap_fee.unwrap_or(5u16),
                },
                pair_type: PairType::Lsd {},
                is_disabled: false,
            }],
            token_code_id,
            owner: owner.to_string(),
            max_referral_commission: Decimal::one(),
            default_stake_config: DefaultStakeConfig {
                staking_code_id,
                tokens_per_power: Uint128::new(1000),
                min_bond: Uint128::new(1000),
                unbonding_periods: vec![60 * 60 * 24 * 7],
                max_distributions: 6,
                converter: None,
            },
            trading_starts: None,
        };

        let factory = app.instantiate_contract(
            factory_code_id,
            owner.clone(),
            &init_msg,
            &[],
            "FACTORY",
            None,
        )?;

        let asset_infos = asset_infos_vec
            .clone()
            .into_iter()
            .map(|(_, asset_info)| asset_info)
            .collect_vec();
        let init_pair_msg = wyndex::factory::ExecuteMsg::CreatePair {
            pair_type: PairType::Lsd {},
            asset_infos: asset_infos.clone(),
            init_params: Some(
                to_binary(&StablePoolParams {
                    amp,
                    owner: None,
                    lsd: None,
                })
                .unwrap(),
            ),
            staking_config: PartialStakeConfig::default(),
            total_fee_bps: None,
        };

        app.execute_contract(owner.clone(), factory.clone(), &init_pair_msg, &[])?;

        let resp: PairInfo = app
            .wrap()
            .query_wasm_smart(&factory, &wyndex::factory::QueryMsg::Pair { asset_infos })?;

        Ok(Self {
            app,
            owner: owner.clone(),
            assets: asset_infos_vec
                .into_iter()
                .map(|(c, a)| (c, a.validate(&MockApi::default()).unwrap()))
                .collect(),
            factory,
            pair_addr: resp.contract_addr,
            lp_token: resp.liquidity_token,
            amp,
        })
    }

    pub fn provide_liquidity(
        &mut self,
        sender: &Addr,
        assets: &[AssetValidated],
    ) -> AnyResult<AppResponse> {
        let funds =
            assets.mock_coins_sent(&mut self.app, sender, &self.pair_addr, SendType::Allowance);

        let msg = ExecuteMsg::ProvideLiquidity {
            assets: assets.iter().cloned().map(Into::into).collect(),
            slippage_tolerance: None,
            receiver: None,
        };

        self.app
            .execute_contract(sender.clone(), self.pair_addr.clone(), &msg, &funds)
    }

    pub fn withdraw_liquidity(
        &mut self,
        sender: &Addr,
        amount: u128,
        assets: Vec<Asset>,
    ) -> AnyResult<AppResponse> {
        let msg = Cw20ExecuteMsg::Send {
            contract: self.pair_addr.to_string(),
            amount: Uint128::from(amount),
            msg: to_binary(&Cw20HookMsg::WithdrawLiquidity { assets }).unwrap(),
        };

        self.app
            .execute_contract(sender.clone(), self.lp_token.clone(), &msg, &[])
    }

    pub fn swap(
        &mut self,
        sender: &Addr,
        offer_asset: &AssetValidated,
        ask_asset_info: Option<AssetInfoValidated>,
    ) -> AnyResult<AppResponse> {
        match &offer_asset.info {
            AssetInfoValidated::Token(contract_addr) => {
                let msg = Cw20ExecuteMsg::Send {
                    contract: self.pair_addr.to_string(),
                    amount: offer_asset.amount,
                    msg: to_binary(&Cw20HookMsg::Swap {
                        ask_asset_info: ask_asset_info.map(Into::into),
                        belief_price: None,
                        max_spread: None,
                        to: None,
                        referral_address: None,
                        referral_commission: None,
                    })
                    .unwrap(),
                };

                self.app
                    .execute_contract(sender.clone(), contract_addr.clone(), &msg, &[])
            }
            AssetInfoValidated::Native { .. } => {
                let funds = offer_asset.mock_coin_sent(
                    &mut self.app,
                    sender,
                    &self.pair_addr,
                    SendType::None,
                );

                let msg = ExecuteMsg::Swap {
                    offer_asset: offer_asset.clone().into(),
                    ask_asset_info: ask_asset_info.map(Into::into),
                    belief_price: None,
                    max_spread: None,
                    to: None,
                    referral_address: None,
                    referral_commission: None,
                };

                self.app
                    .execute_contract(sender.clone(), self.pair_addr.clone(), &msg, &funds)
            }
        }
    }

    pub fn simulate_swap(
        &self,
        offer_asset: impl Into<Asset>,
        ask_asset_info: Option<impl Into<AssetInfo>>,
    ) -> StdResult<SimulationResponse> {
        let offer_asset = offer_asset.into();
        let ask_asset_info = ask_asset_info.map(Into::into);
        self.app.wrap().query_wasm_smart(
            &self.pair_addr,
            &QueryMsg::Simulation {
                offer_asset,
                ask_asset_info: ask_asset_info.map(Into::into),
                referral: false,
                referral_commission: None,
            },
        )
    }

    pub fn simulate_reverse_swap(
        &self,
        ask_asset: impl Into<Asset>,
        offer_asset_info: Option<impl Into<AssetInfo>>,
    ) -> StdResult<ReverseSimulationResponse> {
        let ask_asset = ask_asset.into();
        let offer_asset_info = offer_asset_info.map(Into::into);
        self.app.wrap().query_wasm_smart(
            &self.pair_addr,
            &QueryMsg::ReverseSimulation {
                ask_asset,
                offer_asset_info,
                referral: false,
                referral_commission: None,
            },
        )
    }

    pub fn query_prices(&self) -> StdResult<CumulativePricesResponse> {
        self.app
            .wrap()
            .query_wasm_smart(&self.pair_addr, &QueryMsg::CumulativePrices {})
    }

    fn init_token(
        app: &mut App,
        token_code: u64,
        name: String,
        decimals: u8,
        owner: &Addr,
    ) -> Addr {
        let init_balance = INIT_BALANCE * 10u128.pow(decimals as u32);
        app.instantiate_contract(
            token_code,
            owner.clone(),
            &cw20_base::msg::InstantiateMsg {
                symbol: name.to_string(),
                name,
                decimals,
                initial_balances: vec![Cw20Coin {
                    address: owner.to_string(),
                    amount: Uint128::from(init_balance),
                }],
                mint: None,
                marketing: None,
            },
            &[],
            "{name}_token",
            None,
        )
        .unwrap()
    }

    pub fn token_balance(&self, token_addr: impl Into<String>, user: impl Into<String>) -> u128 {
        let resp: BalanceResponse = self
            .app
            .wrap()
            .query_wasm_smart(
                token_addr,
                &Cw20QueryMsg::Balance {
                    address: user.into(),
                },
            )
            .unwrap();

        resp.balance.u128()
    }

    pub fn coin_balance(&self, coin: &TestCoin, user: &Addr) -> u128 {
        match &self.assets[coin] {
            AssetInfoValidated::Token(contract_addr) => self.token_balance(contract_addr, user),
            AssetInfoValidated::Native(denom) => self
                .app
                .wrap()
                .query_balance(user, denom)
                .unwrap()
                .amount
                .u128(),
        }
    }

    pub fn give_me_money(&mut self, assets: &[AssetValidated], recipient: &Addr) {
        let funds =
            assets.mock_coins_sent(&mut self.app, &self.owner, recipient, SendType::Transfer);

        if !funds.is_empty() {
            self.app
                .send_tokens(self.owner.clone(), recipient.clone(), &funds)
                .unwrap();
        }
    }
}

#[derive(Clone, Copy)]
pub enum SendType {
    Allowance,
    Transfer,
    None,
}

pub trait AssetExt {
    fn mock_coin_sent(
        &self,
        app: &mut App,
        user: &Addr,
        spender: &Addr,
        typ: SendType,
    ) -> Vec<Coin>;
}

impl AssetExt for AssetValidated {
    fn mock_coin_sent(
        &self,
        app: &mut App,
        user: &Addr,
        spender: &Addr,
        typ: SendType,
    ) -> Vec<Coin> {
        let mut funds = vec![];
        match &self.info {
            AssetInfoValidated::Token(contract_addr) if !self.amount.is_zero() => {
                let msg = match typ {
                    SendType::Allowance => Cw20ExecuteMsg::IncreaseAllowance {
                        spender: spender.to_string(),
                        amount: self.amount,
                        expires: None,
                    },
                    SendType::Transfer => Cw20ExecuteMsg::Transfer {
                        recipient: spender.to_string(),
                        amount: self.amount,
                    },
                    _ => unimplemented!(),
                };
                app.execute_contract(user.clone(), contract_addr.clone(), &msg, &[])
                    .unwrap();
            }
            AssetInfoValidated::Native(denom) if !self.amount.is_zero() => {
                funds = vec![coin(self.amount.u128(), denom)];
            }
            _ => {}
        }

        funds
    }
}

pub trait AssetsExt {
    fn mock_coins_sent(
        &self,
        app: &mut App,
        user: &Addr,
        spender: &Addr,
        typ: SendType,
    ) -> Vec<Coin>;
}

impl AssetsExt for &[AssetValidated] {
    fn mock_coins_sent(
        &self,
        app: &mut App,
        user: &Addr,
        spender: &Addr,
        typ: SendType,
    ) -> Vec<Coin> {
        let mut funds = vec![];
        for asset in self.iter() {
            funds.extend(asset.mock_coin_sent(app, user, spender, typ));
        }
        funds
    }
}

pub trait AppExtension {
    fn next_block(&mut self, time: u64);
}

impl AppExtension for App {
    fn next_block(&mut self, time: u64) {
        self.update_block(|block| {
            block.time = block.time.plus_seconds(time);
            block.height += 1
        });
    }
}
