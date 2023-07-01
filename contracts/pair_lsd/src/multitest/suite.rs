use anyhow::Result as AnyResult;

use cosmwasm_std::{coin, to_binary, Addr, Coin, Decimal, Uint128};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use cw20_base::msg::InstantiateMsg as Cw20BaseInstantiateMsg;
use cw_multi_test::{App, AppResponse, BankSudo, ContractWrapper, Executor, SudoMsg};

use wyndex::asset::{Asset, AssetInfo};
use wyndex::factory::{
    DefaultStakeConfig, ExecuteMsg as FactoryExecuteMsg, InstantiateMsg as FactoryInstantiateMsg,
    PairConfig, PairType, QueryMsg as FactoryQueryMsg,
};
use wyndex::fee_config::FeeConfig;
use wyndex::pair::{
    Cw20HookMsg, ExecuteMsg as PairExecuteMsg, PairInfo, QueryMsg, SimulationResponse,
    SpotPricePredictionResponse, SpotPriceResponse, StablePoolParams, StablePoolUpdateParams,
};

use super::mock_hub;

const SECONDS_PER_DAY: u64 = 60 * 60 * 24;

fn store_mock_hub(app: &mut App) -> u64 {
    let contract = Box::new(ContractWrapper::new_with_empty(
        mock_hub::execute,
        mock_hub::instantiate,
        mock_hub::query,
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

fn store_pair(app: &mut App) -> u64 {
    let contract = Box::new(
        ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply_empty(crate::contract::reply),
    );

    app.store_code(contract)
}

fn store_xyk_pair(app: &mut App) -> u64 {
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

fn store_staking(app: &mut App) -> u64 {
    let contract = Box::new(ContractWrapper::new(
        wyndex_stake::contract::execute,
        wyndex_stake::contract::instantiate,
        wyndex_stake::contract::query,
    ));

    app.store_code(contract)
}

#[derive(Debug)]
pub struct SuiteBuilder {
    funds: Vec<(Addr, Vec<Coin>)>,
    max_referral_commission: Decimal,
    stake_config: DefaultStakeConfig,
    total_fee_bps: u16,
    protocol_fee_bps: u16,
    initial_target_rate: Decimal,
}

#[allow(dead_code)]
impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            funds: vec![],
            max_referral_commission: Decimal::one(),
            total_fee_bps: 0,
            protocol_fee_bps: 0,
            stake_config: DefaultStakeConfig {
                staking_code_id: 0, // will be set in build()
                tokens_per_power: Uint128::new(1000),
                min_bond: Uint128::new(1000),
                unbonding_periods: vec![
                    SECONDS_PER_DAY * 7,
                    SECONDS_PER_DAY * 14,
                    SECONDS_PER_DAY * 21,
                ],
                max_distributions: 6,
                converter: None,
            },
            initial_target_rate: Decimal::one(),
        }
    }

    pub fn with_fees(mut self, total_fee_bps: u16, protocol_fee_bps: u16) -> Self {
        self.total_fee_bps = total_fee_bps;
        self.protocol_fee_bps = protocol_fee_bps;
        self
    }

    pub fn with_funds(mut self, addr: &str, funds: &[Coin]) -> Self {
        self.funds.push((Addr::unchecked(addr), funds.into()));
        self
    }

    pub fn with_max_referral_commission(mut self, max: Decimal) -> Self {
        self.max_referral_commission = max;
        self
    }

    pub fn with_initial_target_rate(mut self, rate: Decimal) -> Self {
        self.initial_target_rate = rate;
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");

        let cw20_code_id = store_cw20(&mut app);
        let stable_pair_code_id = store_pair(&mut app);
        let xyk_pair_code_id = store_xyk_pair(&mut app);
        let factory_code_id = store_factory(&mut app);
        let staking_code_id = store_staking(&mut app);
        let factory = app
            .instantiate_contract(
                factory_code_id,
                owner.clone(),
                &FactoryInstantiateMsg {
                    pair_configs: vec![
                        PairConfig {
                            code_id: stable_pair_code_id,
                            pair_type: PairType::Lsd {},
                            fee_config: FeeConfig {
                                total_fee_bps: self.total_fee_bps,
                                protocol_fee_bps: self.protocol_fee_bps,
                            },
                            is_disabled: false,
                        },
                        PairConfig {
                            code_id: xyk_pair_code_id,
                            pair_type: PairType::Xyk {},
                            fee_config: FeeConfig {
                                total_fee_bps: self.total_fee_bps,
                                protocol_fee_bps: self.protocol_fee_bps,
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

        let code_id = store_mock_hub(&mut app);
        let mock_hub = app
            .instantiate_contract(
                code_id,
                owner.clone(),
                &self.initial_target_rate,
                &[],
                "Mock Hub",
                None,
            )
            .unwrap();

        Suite {
            owner: owner.to_string(),
            app,
            factory,
            cw20_code_id,
            mock_hub,
        }
    }
}

pub struct Suite {
    pub owner: String,
    pub app: App,
    pub factory: Addr,
    pub mock_hub: Addr,
    cw20_code_id: u64,
}

#[allow(dead_code)]
impl Suite {
    // update block's time to simulate passage of time
    pub fn wait(&mut self, seconds: u64) {
        self.app
            .update_block(|block| block.time = block.time.plus_seconds(seconds));
    }

    pub fn create_pair(
        &mut self,
        sender: &str,
        pair_type: PairType,
        init_params: Option<StablePoolParams>,
        tokens: &[AssetInfo],
    ) -> AnyResult<Addr> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.factory.clone(),
            &FactoryExecuteMsg::CreatePair {
                pair_type,
                asset_infos: tokens.to_vec(),
                init_params: init_params.map(|p| to_binary(&p).unwrap()),
                staking_config: Default::default(),
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

    pub fn provide_liquidity(
        &mut self,
        owner: &str,
        pair: &Addr,
        assets: &[Asset],
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

    /// Create LP for provided assets and provides some liquidity to them.
    /// Requirement: if using native token provide coins to sent as last argument
    pub fn create_pair_and_provide_liquidity(
        &mut self,
        pair_type: PairType,
        init_params: Option<StablePoolParams>,
        first_asset: (AssetInfo, u128),
        second_asset: (AssetInfo, u128),
        native_tokens: Vec<Coin>,
    ) -> AnyResult<Addr> {
        let owner = self.owner.clone();
        let whale = "whale";

        let pair = self.create_pair(
            &owner,
            pair_type,
            init_params,
            &[first_asset.0.clone(), second_asset.0.clone()],
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
            &[
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

    pub fn mint(&mut self, owner: &str, asset: Asset, recipient: &str) -> AnyResult<AppResponse> {
        match asset.info {
            AssetInfo::Token(token) => self.mint_cw20(
                owner,
                &Addr::unchecked(token),
                asset.amount.u128(),
                recipient,
            ),
            AssetInfo::Native(denom) => self.app.sudo(SudoMsg::Bank(BankSudo::Mint {
                to_address: recipient.to_string(),
                amount: vec![coin(asset.amount.u128(), denom)],
            })),
        }
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

    pub fn change_target_value(&mut self, target_value: Decimal) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked("sender"),
            self.mock_hub.clone(),
            &target_value,
            &[],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn swap<'a>(
        &mut self,
        pair: &Addr,
        sender: &str,
        offer_asset: Asset,
        ask_asset_info: impl Into<Option<AssetInfo>>,
        belief_price: impl Into<Option<Decimal>>,
        max_spread: impl Into<Option<Decimal>>,
        to: impl Into<Option<&'a str>>,
    ) -> AnyResult<AppResponse> {
        match &offer_asset.info {
            AssetInfo::Token(token) => self.app.execute_contract(
                Addr::unchecked(sender),
                Addr::unchecked(token),
                &Cw20ExecuteMsg::Send {
                    contract: pair.to_string(),
                    amount: offer_asset.amount,
                    msg: to_binary(&Cw20HookMsg::Swap {
                        ask_asset_info: ask_asset_info.into(),
                        referral_address: None,
                        referral_commission: None,
                        belief_price: belief_price.into(),
                        max_spread: max_spread.into(),
                        to: to.into().map(|s| s.to_owned()),
                    })?,
                },
                &[],
            ),
            AssetInfo::Native(denom) => {
                let funds = &[coin(offer_asset.amount.u128(), denom)];
                self.app.execute_contract(
                    Addr::unchecked(sender),
                    pair.clone(),
                    &PairExecuteMsg::Swap {
                        offer_asset,
                        ask_asset_info: ask_asset_info.into(),
                        referral_address: None,
                        referral_commission: None,
                        belief_price: belief_price.into(),
                        max_spread: max_spread.into(),
                        to: to.into().map(|s| s.to_owned()),
                    },
                    funds,
                )
            }
        }
    }

    pub fn withdraw_liquidity(
        &mut self,
        sender: &str,
        pair: &Addr,
        liquidity_token: &Addr,
        amount: u128,
        assets: Vec<Asset>,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            liquidity_token.clone(),
            &Cw20ExecuteMsg::Send {
                contract: pair.to_string(),
                amount: amount.into(),
                msg: to_binary(&Cw20HookMsg::WithdrawLiquidity { assets })?,
            },
            &[],
        )
    }

    pub fn update_config(&mut self, params: StablePoolUpdateParams) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked("sender"),
            self.mock_hub.clone(),
            &PairExecuteMsg::UpdateConfig {
                params: to_binary(&params)?,
            },
            &[],
        )
    }

    pub fn query_simulation(
        &self,
        pair: &Addr,
        offer_asset: Asset,
        ask_asset_info: impl Into<Option<AssetInfo>>,
    ) -> AnyResult<SimulationResponse> {
        let res: SimulationResponse = self.app.wrap().query_wasm_smart(
            pair.clone(),
            &QueryMsg::Simulation {
                offer_asset,
                ask_asset_info: ask_asset_info.into(),
                referral: false,
                referral_commission: None,
            },
        )?;
        Ok(res)
    }

    pub fn query_pair(&self, pair: &Addr) -> AnyResult<PairInfo> {
        let res: PairInfo = self
            .app
            .wrap()
            .query_wasm_smart(pair.clone(), &QueryMsg::Pair {})?;
        Ok(res)
    }

    pub fn query_spot_price(
        &self,
        pair: &Addr,
        offer: &AssetInfo,
        ask: &AssetInfo,
    ) -> AnyResult<Decimal> {
        let res: SpotPriceResponse = self.app.wrap().query_wasm_smart(
            pair.clone(),
            &QueryMsg::SpotPrice {
                offer: offer.clone(),
                ask: ask.clone(),
            },
        )?;
        Ok(res.price)
    }

    pub fn query_predict_spot_price(
        &self,
        pair: &Addr,
        offer: &AssetInfo,
        ask: &AssetInfo,
        max_trade: Uint128,
        target_price: Decimal,
        iterations: u8,
    ) -> AnyResult<Option<Uint128>> {
        let res: SpotPricePredictionResponse = self.app.wrap().query_wasm_smart(
            pair.clone(),
            &QueryMsg::SpotPricePrediction {
                offer: offer.clone(),
                ask: ask.clone(),
                max_trade,
                target_price,
                iterations,
            },
        )?;
        Ok(res.trade)
    }

    pub fn query_balance(&self, sender: &str, denom: &str) -> AnyResult<u128> {
        let amount = self
            .app
            .wrap()
            .query_balance(Addr::unchecked(sender), denom)?
            .amount;
        Ok(amount.into())
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
}
