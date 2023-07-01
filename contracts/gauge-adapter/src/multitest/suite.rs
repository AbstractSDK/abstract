use anyhow::Result as AnyResult;

use cosmwasm_std::{coin, to_binary, Addr, Coin, CosmosMsg, Decimal, Uint128};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use cw20_base::msg::InstantiateMsg as Cw20BaseInstantiateMsg;
use cw_multi_test::{App, AppResponse, BankSudo, ContractWrapper, Executor, SudoMsg};

use cw_placeholder::msg::InstantiateMsg as PlaceholderContractInstantiateMsg;
use wyndex::asset::{Asset, AssetInfo, AssetValidated};
use wyndex::factory::{
    DefaultStakeConfig, ExecuteMsg as FactoryExecuteMsg, InstantiateMsg as FactoryInstantiateMsg,
    PairConfig, PairType, PartialStakeConfig, QueryMsg as FactoryQueryMsg,
};
use wyndex::fee_config::FeeConfig;
use wyndex::pair::{ExecuteMsg as PairExecuteMsg, PairInfo, QueryMsg as PairQueryMsg};
use wyndex::stake::{ReceiveMsg, UnbondingPeriod};
use wyndex_stake::msg::{
    ExecuteMsg as StakingExecuteMsg, QueryMsg as StakingQueryMsg, WithdrawableRewardsResponse,
};

use crate::msg::{
    AdapterQueryMsg, AllOptionsResponse, CheckOptionResponse, MigrateMsg, SampleGaugeMsgsResponse,
};

fn store_gauge_adapter(app: &mut App) -> u64 {
    let contract = Box::new(
        ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_migrate_empty(crate::contract::migrate),
    );

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

fn store_placeholder_code(app: &mut App) -> u64 {
    let placeholder_contract = Box::new(ContractWrapper::new_with_empty(
        cw_placeholder::contract::execute,
        cw_placeholder::contract::instantiate,
        cw_placeholder::contract::query,
    ));

    app.store_code(placeholder_contract)
}

#[derive(Debug)]
pub struct SuiteBuilder {
    funds: Vec<(Addr, Vec<Coin>)>,
    stake_config: DefaultStakeConfig,
    reward: Asset,
    via_placeholder: bool,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            funds: vec![],
            stake_config: DefaultStakeConfig {
                staking_code_id: 0, // will be set in build()
                tokens_per_power: Uint128::new(1000),
                min_bond: Uint128::new(1000),
                unbonding_periods: vec![],
                max_distributions: 6,
                converter: None,
            },
            reward: Asset {
                amount: Uint128::zero(),
                info: AssetInfo::Native("juno".to_string()),
            },
            via_placeholder: false,
        }
    }

    pub fn with_funds(mut self, addr: &str, funds: &[Coin]) -> Self {
        self.funds.push((Addr::unchecked(addr), funds.into()));
        self
    }

    pub fn with_stake_config(mut self, stake_config: DefaultStakeConfig) -> Self {
        self.stake_config = stake_config;
        self
    }

    pub fn via_placeholder(mut self) -> Self {
        self.via_placeholder = true;
        self
    }

    pub fn with_native_reward(mut self, amount: u128, denom: &str) -> Self {
        self.reward = Asset {
            amount: amount.into(),
            info: AssetInfo::Native(denom.to_string()),
        };
        self
    }

    pub fn with_cw20_reward(mut self, amount: u128) -> Self {
        self.reward = Asset {
            amount: amount.into(),
            info: AssetInfo::Token(String::new()), // will be filled in when we [`build`]
        };
        self
    }

    #[track_caller]
    pub fn build(mut self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");

        let cw20_code_id = store_cw20(&mut app);
        let pair_code_id = store_pair(&mut app);
        let factory_code_id = store_factory(&mut app);
        let gauge_adapter_code_id = store_gauge_adapter(&mut app);
        let staking_code_id = store_staking(&mut app);
        let place_holder_id = store_placeholder_code(&mut app);

        let epoch_length = 86_400;

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
                        ..self.stake_config
                    },
                    trading_starts: None,
                },
                &[],
                "Wyndex Factory",
                None,
            )
            .unwrap();

        // special handling for cw20 reward
        if let AssetInfo::Token(_) = self.reward.info {
            let cw20 = app
                .instantiate_contract(
                    cw20_code_id,
                    owner.clone(),
                    &Cw20BaseInstantiateMsg {
                        name: "Test Token".to_string(),
                        symbol: "TEST".to_string(),
                        decimals: 6,
                        initial_balances: vec![],
                        mint: Some(MinterResponse {
                            minter: owner.to_string(),
                            cap: None,
                        }),
                        marketing: None,
                    },
                    &[],
                    "Test Token",
                    None,
                )
                .unwrap();

            self.reward.info = AssetInfo::Token(cw20.to_string());
        }

        let adapter_init_msg = crate::msg::InstantiateMsg {
            factory: factory.to_string(),
            owner: owner.to_string(),
            rewards_asset: self.reward.clone(),
            epoch_length,
        };
        let adapter_label = "Gauge Adapter";

        let gauge_adapter = if !self.via_placeholder {
            app.instantiate_contract(
                gauge_adapter_code_id,
                owner.clone(),
                &adapter_init_msg,
                &[],
                adapter_label,
                Some(owner.to_string()),
            )
            .unwrap()
        } else {
            // start with placeholder
            let contract_addr = app
                .instantiate_contract(
                    place_holder_id,
                    owner.clone(),
                    &PlaceholderContractInstantiateMsg {},
                    &[],
                    adapter_label,
                    Some(owner.to_string()),
                )
                .unwrap();
            // now migrate to real one
            app.migrate_contract(
                owner.clone(),
                contract_addr.clone(),
                &MigrateMsg::Init(adapter_init_msg),
                gauge_adapter_code_id,
            )
            .unwrap();
            contract_addr
        };

        app.init_modules(|router, _, storage| -> AnyResult<()> {
            for (addr, coin) in self.funds {
                router.bank.init_balance(storage, &addr, coin)?;
            }
            Ok(())
        })
        .unwrap();

        Suite {
            owner: owner.to_string(),
            app,
            factory,
            gauge_adapter,
            cw20_code_id,
            reward: self.reward,
            epoch_length,
        }
    }
}

pub struct Suite {
    pub owner: String,
    pub app: App,
    pub factory: Addr,
    pub gauge_adapter: Addr,
    cw20_code_id: u64,
    pub reward: Asset,
    pub epoch_length: u64,
}

impl Suite {
    pub fn next_block(&mut self, time: u64) {
        self.app.update_block(|block| {
            block.time = block.time.plus_seconds(time);
            block.height += 1
        });
    }

    /// Creates a new pair, provides 1_000_000 liquidity to it and returns the addresses of the staking and lp token contracts
    pub fn create_pair_staking(
        &mut self,
        first_asset: AssetInfo,
        second_asset: AssetInfo,
    ) -> AnyResult<(StakingContract, Addr)> {
        // collect native token funds to send to pair
        let funds: Vec<_> = [&first_asset, &second_asset]
            .into_iter()
            .filter_map(|a| match a {
                AssetInfo::Native(denom) => Some(coin(1_000_000, denom)),
                _ => None,
            })
            .collect();

        let pair = self
            .create_pair_and_provide_liquidity(
                wyndex::factory::PairType::Xyk {},
                (first_asset, 1_000_000),
                (second_asset, 1_000_000),
                funds,
            )
            .unwrap();
        let pair_info = pair.query_pair_info(&self.app).unwrap();
        Ok((
            StakingContract(pair_info.staking_addr),
            pair_info.liquidity_token,
        ))
    }

    fn create_pair(
        &mut self,
        sender: &str,
        pair_type: PairType,
        tokens: [AssetInfo; 2],
    ) -> AnyResult<PairContract> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.factory.clone(),
            &FactoryExecuteMsg::CreatePair {
                pair_type,
                asset_infos: tokens.to_vec(),
                staking_config: PartialStakeConfig::default(),
                init_params: None,
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
        Ok(PairContract(res.contract_addr))
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
        first_asset: (AssetInfo, u128),
        second_asset: (AssetInfo, u128),
        native_tokens: Vec<Coin>,
    ) -> AnyResult<PairContract> {
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
                    pair.0.as_str(),
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
                    pair.0.as_str(),
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

        pair.provide_liquidity(
            &mut self.app,
            whale,
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
        sender: &str,
        asset_infos: Vec<AssetInfo>,
        asset: AssetInfo,
        rewards: Vec<(UnbondingPeriod, Decimal)>,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
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

    pub fn sample_gauge_msgs(&self, selected: Vec<(String, Decimal)>) -> Vec<CosmosMsg> {
        let msgs: SampleGaugeMsgsResponse = self
            .app
            .wrap()
            .query_wasm_smart(
                self.gauge_adapter.clone(),
                &AdapterQueryMsg::SampleGaugeMsgs { selected },
            )
            .unwrap();
        msgs.execute
    }

    pub fn query_cw20_balance(&self, user: &str, contract: &Addr) -> AnyResult<u128> {
        let balance: BalanceResponse = self.app.wrap().query_wasm_smart(
            contract,
            &Cw20QueryMsg::Balance {
                address: user.to_owned(),
            },
        )?;
        Ok(balance.balance.into())
    }

    pub fn query_all_options(&self) -> AnyResult<Vec<String>> {
        let res: AllOptionsResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.gauge_adapter.clone(), &AdapterQueryMsg::AllOptions {})?;

        Ok(res.options)
    }

    pub fn query_check_option(&self, option: String) -> AnyResult<bool> {
        let res: CheckOptionResponse = self.app.wrap().query_wasm_smart(
            self.gauge_adapter.clone(),
            &AdapterQueryMsg::CheckOption { option },
        )?;

        Ok(res.valid)
    }
}

pub struct PairContract(pub Addr);

impl PairContract {
    pub fn query_pair_info(&self, app: &App) -> AnyResult<PairInfo> {
        Ok(app
            .wrap()
            .query_wasm_smart(&self.0, &PairQueryMsg::Pair {})?)
    }

    pub fn provide_liquidity(
        &self,
        app: &mut App,
        owner: &str,
        assets: [Asset; 2],
        send_funds: &[Coin],
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            Addr::unchecked(owner),
            self.0.clone(),
            &PairExecuteMsg::ProvideLiquidity {
                assets: assets.to_vec(),
                slippage_tolerance: None,
                receiver: None,
            },
            send_funds,
        )
    }
}

pub struct StakingContract(pub Addr);

impl StakingContract {
    pub fn stake(
        &self,
        app: &mut App,
        owner: &str,
        amount: u128,
        unbonding_period: u64,
        token: Addr,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            Addr::unchecked(owner),
            token,
            &Cw20ExecuteMsg::Send {
                contract: self.0.to_string(),
                amount: amount.into(),
                msg: to_binary(&ReceiveMsg::Delegate {
                    unbonding_period,
                    delegate_as: None,
                })
                .unwrap(),
            },
            &[],
        )
    }

    pub fn query_withdrawable_rewards(
        &self,
        app: &App,
        owner: &str,
    ) -> AnyResult<Vec<AssetValidated>> {
        let rewards: WithdrawableRewardsResponse = app
            .wrap()
            .query_wasm_smart(
                &self.0,
                &StakingQueryMsg::WithdrawableRewards {
                    owner: owner.to_string(),
                },
            )
            .unwrap();
        Ok(rewards.rewards)
    }

    pub fn withdraw_rewards(&self, app: &mut App, owner: &str) -> AnyResult<AppResponse> {
        app.execute_contract(
            Addr::unchecked(owner),
            self.0.clone(),
            &StakingExecuteMsg::WithdrawRewards {
                owner: None,
                receiver: None,
            },
            &[],
        )
    }

    pub fn distribute_rewards(&self, app: &mut App, owner: &str) -> AnyResult<AppResponse> {
        app.execute_contract(
            Addr::unchecked(owner),
            self.0.clone(),
            &StakingExecuteMsg::DistributeRewards { sender: None },
            &[],
        )
    }
}
