use anyhow::Result as AnyResult;

use cosmwasm_std::{coin, Addr, Coin, Decimal, Uint128};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use cw20_base::msg::InstantiateMsg as Cw20BaseInstantiateMsg;
use cw_multi_test::{App, AppResponse, BankSudo, ContractWrapper, Executor, SudoMsg};

use wyndex::asset::{Asset, AssetInfo};
use wyndex::factory::{
    DefaultStakeConfig, ExecuteMsg as FactoryExecuteMsg, InstantiateMsg as FactoryInstantiateMsg,
    PairConfig, PairType, PartialStakeConfig, QueryMsg as FactoryQueryMsg,
};
use wyndex::fee_config::FeeConfig;
use wyndex::pair::{ExecuteMsg as PairExecuteMsg, PairInfo};

use crate::msg::AssetWithLimit;
use crate::msg::BalancesResponse as TraderBalancesResponse;
use crate::msg::InstantiateMsg as TraderInstantiateMsg;
use crate::msg::QueryMsg as TraderQueryMsg;

fn contract_trader(app: &mut App) -> u64 {
    let contract = Box::new(ContractWrapper::new_with_empty(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ));

    app.store_code(contract)
}

fn store_factory(app: &mut App) -> u64 {
    let contract = Box::new(
        ContractWrapper::new_with_empty(
            wyndex_factory::contract::execute,
            wyndex_factory::contract::instantiate,
            wyndex_factory::contract::query,
        )
        .with_reply_empty(wyndex_factory::contract::reply),
    );

    app.store_code(contract)
}

fn store_staking(app: &mut App) -> u64 {
    let contract = Box::new(ContractWrapper::new_with_empty(
        wyndex_stake::contract::execute,
        wyndex_stake::contract::instantiate,
        wyndex_stake::contract::query,
    ));

    app.store_code(contract)
}

fn store_pair(app: &mut App) -> u64 {
    let contract = Box::new(
        ContractWrapper::new_with_empty(
            wyndex_pair::contract::execute,
            wyndex_pair::contract::instantiate,
            wyndex_pair::contract::query,
        )
        .with_reply_empty(wyndex_pair::contract::reply),
    );

    app.store_code(contract)
}

fn store_cw20(app: &mut App) -> u64 {
    let contract = Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));

    app.store_code(contract)
}

#[derive(Debug)]
pub struct SuiteBuilder {
    funds: Vec<(Addr, Vec<Coin>)>,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self { funds: vec![] }
    }

    pub fn with_funds(mut self, addr: &str, funds: &[Coin]) -> Self {
        self.funds.push((Addr::unchecked(addr), funds.into()));
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");
        let trader: Addr = Addr::unchecked("trader");

        let cw20_code_id = store_cw20(&mut app);
        let pair_code_id = store_pair(&mut app);
        let factory_code_id = store_factory(&mut app);
        let staking_code_id = store_staking(&mut app);

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
                    max_referral_commission: Decimal::one(),
                    default_stake_config: DefaultStakeConfig {
                        staking_code_id,
                        tokens_per_power: Uint128::new(1000),
                        min_bond: Uint128::new(1000),
                        unbonding_periods: vec![1],
                        max_distributions: 6,
                        converter: None,
                    },
                    trading_starts: None,
                },
                &[],
                "Wyndex Factory",
                None,
            )
            .unwrap();

        let funds = self.funds;
        app.init_modules(|router, _, storage| -> AnyResult<()> {
            for (addr, coin) in funds {
                router.bank.init_balance(storage, &addr, coin)?;
            }
            Ok(())
        })
        .unwrap();

        let trader_id = contract_trader(&mut app);

        Suite {
            owner: owner.to_string(),
            trader: trader.to_string(),
            app,
            factory,
            cw20_code_id,
            trader_id,
        }
    }
}

pub struct Suite {
    pub owner: String,
    pub trader: String,
    app: App,
    factory: Addr,
    cw20_code_id: u64,
    pub trader_id: u64,
}

impl Suite {
    pub fn setup_trader(&mut self, sender: Addr, desired_token_info: AssetInfo) -> AnyResult<Addr> {
        let trader_contract = self
            .app
            .instantiate_contract(
                self.trader_id,
                sender,
                &TraderInstantiateMsg {
                    owner: self.owner.to_string(),
                    nominated_trader: self.trader.to_string(),
                    beneficiary: Addr::unchecked("beneficiary").to_string(),
                    token_contract: desired_token_info,
                    dex_factory_contract: self.factory.to_string(),
                    max_spread: Some(Decimal::percent(50)),
                },
                &[],
                "trader",
                None,
            )
            .unwrap();
        Ok(trader_contract)
    }

    pub fn update_routes(
        &mut self,
        sender: &str,
        trader_contract: Addr,
        add: Option<Vec<(AssetInfo, AssetInfo)>>,
        remove: Option<Vec<AssetInfo>>,
    ) -> AnyResult<AppResponse> {
        let res = self.app.execute_contract(
            Addr::unchecked(sender),
            trader_contract,
            &crate::msg::ExecuteMsg::UpdateRoutes { add, remove },
            &[],
        )?;
        Ok(res)
    }

    pub fn spend(
        &mut self,
        sender: &str,
        trader_contract: Addr,
        recipient: String,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        let res = self.app.execute_contract(
            Addr::unchecked(sender),
            trader_contract,
            &crate::msg::ExecuteMsg::Transfer { recipient, amount },
            &[],
        )?;
        Ok(res)
    }

    pub fn query_trader_balances(
        &self,
        address: &Addr,
        balances_to_query: Vec<AssetInfo>,
    ) -> AnyResult<TraderBalancesResponse> {
        let balance: TraderBalancesResponse = self.app.wrap().query_wasm_smart(
            address,
            &TraderQueryMsg::Balances {
                assets: balances_to_query,
            },
        )?;
        Ok(balance)
    }

    pub fn send_native_tokens(
        &mut self,
        sender: &str,
        trader_contract: Addr,
        amount: &[Coin],
    ) -> AnyResult<AppResponse> {
        let res = self
            .app
            .send_tokens(Addr::unchecked(sender), trader_contract, amount)?;
        Ok(res)
    }

    pub fn trade_collected_assets(
        &mut self,
        sender: &str,
        trader_contract: Addr,
        assets: Vec<AssetWithLimit>,
    ) -> AnyResult<AppResponse> {
        let res = self.app.execute_contract(
            Addr::unchecked(sender),
            trader_contract,
            &crate::msg::ExecuteMsg::Collect { assets },
            &[],
        )?;
        Ok(res)
    }

    fn create_pair(
        &mut self,
        sender: &str,
        pair_type: PairType,
        tokens: [AssetInfo; 2],
    ) -> AnyResult<Addr> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.factory.clone(),
            &FactoryExecuteMsg::CreatePair {
                pair_type,
                asset_infos: tokens.to_vec(),
                init_params: None,
                staking_config: PartialStakeConfig::default(),
                total_fee_bps: None,
            },
            &[],
        )?;

        let factory = self.factory.clone();
        let res: PairInfo = self.app.wrap().query_wasm_smart(
            Addr::unchecked(factory),
            &FactoryQueryMsg::Pair {
                asset_infos: tokens.to_vec(),
            },
        )?;
        Ok(res.contract_addr)
    }

    fn provide_liquidity(
        &mut self,
        owner: &str,
        pair: &Addr,
        assets: [Asset; 2],
        send_funds: &[Coin],
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(owner),
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
        let whale = "whale";

        let pair = self.create_pair(
            &owner,
            pair_type,
            [first_asset.0.clone(), second_asset.0.clone()],
        )?;

        match first_asset.0.clone() {
            AssetInfo::Token(addr) => {
                // Mint some initial balances for whale user
                self.mint_cw20(&owner, &Addr::unchecked(&addr), first_asset.1, whale)
                    .unwrap();
                // Increases allowances for given LP contracts in order to provide liquidity to pool
                self.increase_allowance(
                    whale,
                    &Addr::unchecked(addr),
                    pair.as_str(),
                    first_asset.1,
                )
                .unwrap();
            }
            AssetInfo::Native(denom) => {
                self.app
                    .sudo(SudoMsg::Bank(BankSudo::Mint {
                        to_address: whale.to_owned(),
                        amount: vec![coin(first_asset.1, denom)],
                    }))
                    .unwrap();
            }
        };
        match second_asset.0.clone() {
            AssetInfo::Token(addr) => {
                // Mint some initial balances for whale user
                self.mint_cw20(&owner, &Addr::unchecked(&addr), second_asset.1, whale)
                    .unwrap();
                // Increases allowances for given LP contracts in order to provide liquidity to pool
                self.increase_allowance(
                    whale,
                    &Addr::unchecked(addr),
                    pair.as_str(),
                    second_asset.1,
                )
                .unwrap();
            }
            AssetInfo::Native(denom) => {
                self.app
                    .sudo(SudoMsg::Bank(BankSudo::Mint {
                        to_address: whale.to_owned(),
                        amount: vec![coin(second_asset.1, denom)],
                    }))
                    .unwrap();
            }
        };

        self.provide_liquidity(
            whale,
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

    pub fn instantiate_token(&mut self, owner: &str, token: &str) -> Addr {
        self.app
            .instantiate_contract(
                self.cw20_code_id,
                Addr::unchecked(owner),
                &Cw20BaseInstantiateMsg {
                    name: token.to_owned(),
                    symbol: token.to_owned(),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: owner.to_string(),
                        cap: None,
                    }),
                    marketing: None,
                },
                &[],
                token,
                None,
            )
            .unwrap()
    }

    pub fn mint_cw20(
        &mut self,
        owner: &str,
        token: &Addr,
        amount: u128,
        recipient: &str,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(owner),
            token.clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: recipient.to_owned(),
                amount: amount.into(),
            },
            &[],
        )
    }

    pub fn query_cw20_balance(&self, sender: &str, address: &Addr) -> AnyResult<u128> {
        let balance: BalanceResponse = self.app.wrap().query_wasm_smart(
            address,
            &Cw20QueryMsg::Balance {
                address: sender.to_owned(),
            },
        )?;
        Ok(balance.balance.into())
    }

    pub fn query_routes(&self, address: &Addr) -> AnyResult<Vec<(String, String)>> {
        Ok(self
            .app
            .wrap()
            .query_wasm_smart(address, &TraderQueryMsg::Routes {})?)
    }
}
