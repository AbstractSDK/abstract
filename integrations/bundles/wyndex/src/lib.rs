use cw20::msg::Cw20ExecuteMsgFns;
pub mod suite;

use std::fmt::Debug;

use abstract_interface::{Abstract, AbstractInterfaceError};
use abstract_std::{
    ans_host::ExecuteMsgFns,
    objects::{
        pool_id::PoolAddressBase, AssetEntry, LpToken, PoolMetadata, UncheckedContractEntry,
    },
};
use cosmwasm_std::{coin, Decimal, Uint128};
use cw20::Cw20Coin;
use cw_orch::prelude::*;
use cw_plus_interface::cw20_base::Cw20Base as AbstractCw20Base;
use wyndex::{
    asset::{AssetInfo, AssetInfoExt},
    factory::{DefaultStakeConfig, PartialStakeConfig},
};

use self::suite::{Suite, SuiteBuilder};

pub const STAKING: &str = "wyndex:staking";
pub const FACTORY: &str = "wyndex:factory";
pub const WYND_TOKEN: &str = "wynd";
const EUR_USD_PAIR: &str = "wyndex:eur_usd_pair";
pub const EUR_USD_STAKE: &str = "wyndex:eur_usd_staking";
pub const RAW_EUR_STAKE: &str = "wyndex:raw_eur_staking";
pub const RAW_RAW_2_STAKE: &str = "wyndex:raw_raw_2_staking";
pub const EUR_USD_LP: &str = "wyndex/eur,usd";
const WYND_EUR_PAIR: &str = "wyndex:wynd_eur_pair";
pub const WYND_EUR_LP: &str = "wyndex/wynd,eur";
pub const EUR: &str = "eur";
pub const USD: &str = "usd";
pub const WYNDEX: &str = "wyndex";
pub const WYNDEX_OWNER: &str = "wyndex_owner";
pub const POOL_FACTORY: &str = "pool_factory";
pub const MULTI_HOP: &str = "multi_hop";
pub const RAW_TOKEN: &str = "raw";
pub const RAW_2_TOKEN: &str = "raw_2";
pub const RAW_EUR_LP: &str = "wyndex/eur,raw";
pub const RAW_RAW_2_LP: &str = "wyndex/eur,raw";
const RAW_EUR_PAIR: &str = "wyndex:eur_raw_pair";
const RAW_RAW_2_PAIR: &str = "wyndex:raw_raw_2_pair";

pub struct WynDex {
    /// Suite can be used to create new pools and register new rewards.
    pub suite: Suite,
    pub eur_usd_staking: Addr,
    pub raw_eur_staking: Addr,
    pub raw_raw_2_staking: Addr,
    pub eur_token: AssetInfo,
    pub usd_token: AssetInfo,
    // incentivized pair
    // rewarded in wynd
    pub eur_usd_pair: Addr,
    pub eur_usd_lp: AbstractCw20Base<MockBech32>,
    pub wynd_token: AssetInfo,
    pub wynd_eur_pair: Addr,
    pub wynd_eur_lp: AbstractCw20Base<MockBech32>,
    pub raw_token: AbstractCw20Base<MockBech32>,
    pub raw_2_token: AbstractCw20Base<MockBech32>,
    pub raw_eur_pair: Addr,
    pub raw_eur_lp: AbstractCw20Base<MockBech32>,
    pub raw_raw_2_pair: Addr,
    pub raw_raw_2_lp: AbstractCw20Base<MockBech32>,
}

// Shitty implementation until https://github.com/AbstractSDK/cw-orchestrator/issues/60 is done
impl PartialEq for WynDex {
    fn eq(&self, other: &Self) -> bool {
        self.suite == other.suite
            && self.eur_usd_staking == other.eur_usd_staking
            && self.eur_token == other.eur_token
            && self.usd_token == other.usd_token
            && self.eur_usd_pair == other.eur_usd_pair
            && self.wynd_token == other.wynd_token
            && self.wynd_eur_pair == other.wynd_eur_pair
    }
}

impl Debug for WynDex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WynDex")
            .field("eur_usd_staking", &self.eur_usd_staking)
            .field("eur_token", &self.eur_token)
            .field("usd_token", &self.usd_token)
            .field("eur_usd_pair", &self.eur_usd_pair)
            .field("wynd_token", &self.wynd_token)
            .field("wynd_eur_pair", &self.wynd_eur_pair)
            .finish()
    }
}

/// Instantiate a new token instance with some initial balance given to the minter
pub fn create_new_cw20<Chain: CwEnv, T: Into<Uint128>>(
    cw20: &AbstractCw20Base<Chain>,
    minter: &Addr,
    balance: T,
) -> Result<TxResponse<Chain>, AbstractInterfaceError> {
    let msg = cw20_base::msg::InstantiateMsg {
        decimals: 6,
        mint: None,
        symbol: "test".to_string(),
        name: "test".to_string(),
        initial_balances: vec![Cw20Coin {
            address: minter.clone().into(),
            amount: balance.into(),
        }],
        marketing: None,
    };

    cw20.instantiate(&msg, Some(minter), None)
        .map_err(Into::into)
}

// Two step deploy process for WyndDex
// First create Suite with SuiteBuilder, this uploads contracts and instantiates factory
// Then create first pair and stake config and return WyndDex object
impl Deploy<MockBech32> for WynDex {
    type Error = AbstractInterfaceError;
    type DeployData = Empty;

    fn store_on(chain: MockBech32) -> Result<Self, Self::Error> {
        let eur_usd_lp: AbstractCw20Base<MockBech32> =
            AbstractCw20Base::new(EUR_USD_LP, chain.clone());
        let wynd_eur_lp: AbstractCw20Base<MockBech32> =
            AbstractCw20Base::new(WYND_EUR_LP, chain.clone());
        let raw_eur_lp: AbstractCw20Base<MockBech32> =
            AbstractCw20Base::new(RAW_EUR_LP, chain.clone());
        let raw_raw_2_lp: AbstractCw20Base<MockBech32> =
            AbstractCw20Base::new(RAW_RAW_2_LP, chain.clone());

        let owner = chain.addr_make(WYNDEX_OWNER);

        let eur_info = AssetInfo::Native(EUR.to_string());
        let usd_info = AssetInfo::Native(USD.to_string());
        let wynd_info = AssetInfo::Native(WYND_TOKEN.to_string());
        let raw = AbstractCw20Base::new(RAW_TOKEN, chain.clone());
        let raw_2 = AbstractCw20Base::new(RAW_2_TOKEN, chain.clone());
        raw.upload()?;
        raw_2.set_code_id(raw.code_id()?);
        create_new_cw20(&raw, &owner, Uint128::new(100_000_000_000))?;
        create_new_cw20(&raw_2, &owner, Uint128::new(100_000_000_000))?;
        let raw_info = AssetInfo::Token(raw.addr_str()?);
        let raw_2_info = AssetInfo::Token(raw_2.addr_str()?);

        chain.set_balance(
            &owner,
            vec![
                coin(30_000, EUR),
                coin(10_000, USD),
                coin(20_000, WYND_TOKEN),
            ],
        )?;

        // Instantiate test suite with default stake config
        // uploads contracts and instantiates factory
        let mut suite = SuiteBuilder::new()
            .with_stake_config(DefaultStakeConfig {
                staking_code_id: 0,
                tokens_per_power: Uint128::new(1),
                min_bond: Uint128::new(1),
                unbonding_periods: vec![1, 2],
                max_distributions: 1,
            })
            .build(&chain);

        // let mut app = chain.app.borrow_mut();
        let mut state = chain.state.clone();

        state.set_address(POOL_FACTORY, &suite.factory);
        state.set_address(MULTI_HOP, &suite.multi_hop);

        // create euro_usd pair
        let eur_usd_pair = suite
            .create_pair(
                &owner,
                wyndex::factory::PairType::Xyk {},
                [eur_info.clone(), usd_info.clone()],
                Some(PartialStakeConfig {
                    tokens_per_power: Some(Uint128::new(100)),
                    min_bond: Some(Uint128::new(100)),
                    ..Default::default()
                }),
                None,
            )
            .unwrap();

        let pair_info = suite
            .query_pair(vec![eur_info.clone(), usd_info.clone()])
            .unwrap();

        let eur_usd_lp_token = pair_info.liquidity_token;
        eur_usd_lp.set_address(&eur_usd_lp_token);
        let eur_usd_staking = pair_info.staking_addr;
        state.set_address(EUR_USD_PAIR, &eur_usd_pair);
        state.set_address(EUR_USD_STAKE, &eur_usd_staking);

        // owner provides some initial liquidity
        suite
            .provide_liquidity(
                &owner,
                &eur_usd_pair,
                [
                    eur_info.with_balance(10_000u128),
                    usd_info.with_balance(10_000u128),
                ],
                &[coin(10_000, EUR), coin(10_000, USD)],
            )
            .unwrap();

        // create wynd_eur pair
        let wynd_eur_pair = suite
            .create_pair(
                &owner,
                wyndex::factory::PairType::Xyk {},
                [eur_info.clone(), wynd_info.clone()],
                Some(PartialStakeConfig {
                    tokens_per_power: Some(Uint128::new(100)),
                    min_bond: Some(Uint128::new(100)),
                    ..Default::default()
                }),
                None,
            )
            .unwrap();

        let pair_info = suite
            .query_pair(vec![eur_info.clone(), wynd_info.clone()])
            .unwrap();

        let wynd_eur_lp_token = pair_info.liquidity_token;
        wynd_eur_lp.set_address(&wynd_eur_lp_token);
        state.set_address(WYND_EUR_PAIR, &wynd_eur_pair);

        // owner provides some initial liquidity
        suite
            .provide_liquidity(
                &owner,
                &wynd_eur_pair,
                [
                    eur_info.with_balance(10_000u128),
                    wynd_info.with_balance(10_000u128),
                ],
                &[coin(10_000, EUR), coin(10_000, WYND_TOKEN)],
            )
            .unwrap();

        // create rewards distribution
        // wynd tokens are distributed to the pool's stakers.
        suite
            .create_distribution_flow(
                &owner,
                vec![eur_info.clone(), usd_info.clone()],
                wynd_info.clone(),
                vec![(1, Decimal::percent(50)), (2, Decimal::one())],
            )
            .unwrap();

        state.set_address(FACTORY, &suite.factory);

        // create raw_eur pair
        let raw_eur_pair = suite
            .create_pair(
                &owner,
                wyndex::factory::PairType::Xyk {},
                [eur_info.clone(), raw_info.clone()],
                Some(PartialStakeConfig {
                    tokens_per_power: Some(Uint128::new(100)),
                    min_bond: Some(Uint128::new(100)),
                    ..Default::default()
                }),
                None,
            )
            .unwrap();

        let pair_info = suite
            .query_pair(vec![eur_info.clone(), raw_info.clone()])
            .unwrap();

        let raw_eur_lp_token = pair_info.liquidity_token;
        let raw_eur_staking = pair_info.staking_addr;
        raw_eur_lp.set_address(&raw_eur_lp_token);
        state.set_address(RAW_EUR_PAIR, &raw_eur_pair);
        state.set_address(RAW_EUR_STAKE, &raw_eur_staking);

        // set allowance
        raw.call_as(&owner).increase_allowance(
            10_000u128.into(),
            raw_eur_pair.to_string(),
            None,
        )?;
        // owner provides some initial liquidity
        suite
            .provide_liquidity(
                &owner,
                &raw_eur_pair,
                [
                    eur_info.with_balance(10_000u128),
                    raw_info.with_balance(10_000u128),
                ],
                &[coin(10_000, EUR)],
            )
            .unwrap();

        // create rewards distribution
        // wynd tokens are distributed to the pool's stakers.
        suite
            .create_distribution_flow(
                &owner,
                vec![raw_info.clone(), eur_info.clone()],
                wynd_info.clone(),
                vec![(1, Decimal::percent(50)), (2, Decimal::one())],
            )
            .unwrap();

        // create raw_raw_2 pair
        let raw_raw_2_pair = suite
            .create_pair(
                &owner,
                wyndex::factory::PairType::Xyk {},
                [raw_2_info.clone(), raw_info.clone()],
                Some(PartialStakeConfig {
                    tokens_per_power: Some(Uint128::new(100)),
                    min_bond: Some(Uint128::new(100)),
                    ..Default::default()
                }),
                None,
            )
            .unwrap();

        let pair_info = suite
            .query_pair(vec![raw_2_info.clone(), raw_info.clone()])
            .unwrap();

        let raw_raw_2_lp_token = pair_info.liquidity_token;
        let raw_raw_2_staking = pair_info.staking_addr;
        raw_raw_2_lp.set_address(&raw_raw_2_lp_token);
        state.set_address(RAW_RAW_2_PAIR, &raw_raw_2_pair);
        state.set_address(RAW_RAW_2_STAKE, &raw_raw_2_staking);

        // set allowance
        raw.call_as(&owner).increase_allowance(
            10_000u128.into(),
            raw_raw_2_pair.to_string(),
            None,
        )?;
        raw_2.call_as(&owner).increase_allowance(
            10_000u128.into(),
            raw_raw_2_pair.to_string(),
            None,
        )?;
        // owner provides some initial liquidity
        suite
            .provide_liquidity(
                &owner,
                &raw_raw_2_pair,
                [
                    raw_2_info.with_balance(10_000u128),
                    raw_info.with_balance(10_000u128),
                ],
                &[],
            )
            .unwrap();

        // create rewards distribution
        // wynd tokens are distributed to the pool's stakers.
        suite
            .create_distribution_flow(
                &owner,
                vec![raw_info, raw_2_info.clone()],
                wynd_info.clone(),
                vec![(1, Decimal::percent(50)), (2, Decimal::one())],
            )
            .unwrap();

        let wyndex = Self {
            suite,
            eur_usd_pair,
            eur_usd_staking,
            raw_eur_staking,
            raw_raw_2_staking,
            wynd_eur_pair,
            wynd_eur_lp,
            raw_token: raw,
            raw_2_token: raw_2,
            raw_eur_pair,
            raw_raw_2_pair,
            raw_eur_lp,
            eur_usd_lp,
            raw_raw_2_lp,
            wynd_token: wynd_info,
            eur_token: eur_info,
            usd_token: usd_info,
        };

        // register contracts in abstract host
        let abstract_ = Abstract::load_from(chain)?;
        wyndex.register_info_on_abstract(&abstract_)?;

        Ok(wyndex)
    }

    // Loads WynDex addresses from state
    fn load_from(chain: MockBech32) -> Result<Self, Self::Error> {
        let state = chain.state.borrow();
        // load all addresses for Self from state
        let eur_usd_pair = state.get_address(EUR_USD_PAIR)?;
        let eur_usd_lp: AbstractCw20Base<MockBech32> =
            AbstractCw20Base::new(EUR_USD_LP, chain.clone());
        let wynd_eur_pair = state.get_address(WYND_EUR_PAIR)?;
        let wynd_eur_lp: AbstractCw20Base<MockBech32> =
            AbstractCw20Base::new(WYND_EUR_LP, chain.clone());
        let raw_eur_pair = state.get_address(RAW_EUR_PAIR)?;
        let raw_eur_lp = AbstractCw20Base::new(RAW_EUR_LP, chain.clone());
        let raw_raw_2_pair = state.get_address(RAW_RAW_2_PAIR)?;
        let raw_raw_2_lp = AbstractCw20Base::new(RAW_RAW_2_LP, chain.clone());

        let eur_info = AssetInfo::Native(EUR.to_string());
        let usd_info = AssetInfo::Native(USD.to_string());
        let wynd_info = AssetInfo::Native(WYND_TOKEN.to_string());
        let raw = AbstractCw20Base::new(RAW_TOKEN, chain.clone());
        let raw_2 = AbstractCw20Base::new(RAW_2_TOKEN, chain.clone());

        Ok(Self {
            suite: Suite::load_from(&chain),
            eur_usd_pair,
            eur_usd_lp,
            raw_token: raw,
            raw_2_token: raw_2,
            raw_eur_pair,
            raw_eur_lp,
            wynd_eur_pair,
            raw_raw_2_pair,
            wynd_eur_lp,
            raw_raw_2_lp,
            wynd_token: wynd_info,
            eur_token: eur_info,
            usd_token: usd_info,
            eur_usd_staking: state.get_address(EUR_USD_STAKE)?,
            raw_eur_staking: state.get_address(RAW_EUR_STAKE)?,
            raw_raw_2_staking: state.get_address(RAW_RAW_2_STAKE)?,
        })
    }
    fn get_contracts_mut(&mut self) -> Vec<Box<&mut dyn ContractInstance<MockBech32>>> {
        vec![
            Box::new(&mut self.eur_usd_lp),
            Box::new(&mut self.wynd_eur_lp),
            Box::new(&mut self.raw_token),
            Box::new(&mut self.raw_eur_lp),
        ]
    }

    // This is only a mock deploy
    fn deployed_state_file_path() -> Option<String> {
        None
    }
}
impl WynDex {
    /// registers the WynDex contracts and assets on Abstract
    /// this includes:
    /// - registering the assets on ANS
    ///   - EUR
    ///   - USD
    ///   - WYND
    ///   - RAW
    ///   - RAW_2
    ///   - EUR/USD LP
    ///   - EUR/WYND LP
    ///   - EUR/RAW LP
    ///   - RAW/RAW_2 LP
    /// - Register the staking contract
    ///   - wyndex:staking/wyndex/eur,usd (native)
    ///   - wyndex:staking/wyndex/eur,raw (native-cw20)
    ///   - wyndex:staking/wyndex/raw,raw_2 (cw20-cw20)
    /// - Register the pair contracts
    ///   - wyndex/eur,usd
    ///   - wyndex/eur,wynd
    pub(crate) fn register_info_on_abstract(
        &self,
        abstrct: &Abstract<MockBech32>,
    ) -> Result<(), CwOrchError> {
        let eur_asset = AssetEntry::new(EUR);
        let usd_asset = AssetEntry::new(USD);
        let raw_asset = AssetEntry::new(RAW_TOKEN);
        let raw_2_asset = AssetEntry::new(RAW_2_TOKEN);
        let wynd_asset = AssetEntry::new(WYND_TOKEN);
        let eur_usd_lp_asset = LpToken::new(WYNDEX, vec![EUR, USD]);
        let eur_wynd_lp_asset = LpToken::new(WYNDEX, vec![WYND_TOKEN, EUR]);
        let eur_raw_lp_asset = LpToken::new(WYNDEX, vec![RAW_TOKEN, EUR]);
        let raw_raw_2_lp_asset = LpToken::new(WYNDEX, vec![RAW_TOKEN, RAW_2_TOKEN]);

        // Register addresses on ANS
        abstrct
            .ans_host
            .update_asset_addresses(
                vec![
                    (
                        eur_asset.to_string(),
                        cw_asset::AssetInfoBase::native(self.eur_token.to_string()),
                    ),
                    (
                        usd_asset.to_string(),
                        cw_asset::AssetInfoBase::native(self.usd_token.to_string()),
                    ),
                    (
                        WYND_TOKEN.to_string(),
                        cw_asset::AssetInfoBase::native(self.wynd_token.to_string()),
                    ),
                    (
                        raw_asset.to_string(),
                        cw_asset::AssetInfoBase::cw20(self.raw_token.addr_str()?),
                    ),
                    (
                        raw_2_asset.to_string(),
                        cw_asset::AssetInfoBase::cw20(self.raw_2_token.addr_str()?),
                    ),
                    (
                        eur_usd_lp_asset.to_string(),
                        cw_asset::AssetInfoBase::cw20(self.eur_usd_lp.addr_str()?),
                    ),
                    (
                        eur_wynd_lp_asset.to_string(),
                        cw_asset::AssetInfoBase::cw20(self.wynd_eur_lp.addr_str()?),
                    ),
                    (
                        eur_raw_lp_asset.to_string(),
                        cw_asset::AssetInfoBase::cw20(self.raw_eur_lp.addr_str()?),
                    ),
                    (
                        raw_raw_2_lp_asset.to_string(),
                        cw_asset::AssetInfoBase::cw20(self.raw_raw_2_lp.addr_str()?),
                    ),
                ],
                vec![],
            )
            .unwrap();

        abstrct
            .ans_host
            .update_contract_addresses(
                vec![(
                    UncheckedContractEntry::new(
                        WYNDEX.to_string(),
                        format!("staking/{eur_usd_lp_asset}"),
                    ),
                    self.eur_usd_staking.to_string(),
                )],
                vec![],
            )
            .unwrap();

        abstrct
            .ans_host
            .update_contract_addresses(
                vec![(
                    UncheckedContractEntry::new(
                        WYNDEX.to_string(),
                        format!("staking/{eur_raw_lp_asset}"),
                    ),
                    self.raw_eur_staking.to_string(),
                )],
                vec![],
            )
            .unwrap();

        abstrct
            .ans_host
            .update_contract_addresses(
                vec![(
                    UncheckedContractEntry::new(
                        WYNDEX.to_string(),
                        format!("staking/{raw_raw_2_lp_asset}"),
                    ),
                    self.raw_raw_2_staking.to_string(),
                )],
                vec![],
            )
            .unwrap();

        abstrct
            .ans_host
            .update_dexes(vec![WYNDEX.into()], vec![])
            .unwrap();

        abstrct
            .ans_host
            .update_pools(
                vec![
                    (
                        PoolAddressBase::contract(self.eur_usd_pair.to_string()),
                        PoolMetadata::constant_product(WYNDEX, vec![eur_asset.clone(), usd_asset]),
                    ),
                    (
                        PoolAddressBase::contract(self.wynd_eur_pair.to_string()),
                        PoolMetadata::constant_product(WYNDEX, vec![wynd_asset, eur_asset.clone()]),
                    ),
                    (
                        PoolAddressBase::contract(self.raw_eur_pair.to_string()),
                        PoolMetadata::constant_product(WYNDEX, vec![raw_asset.clone(), eur_asset]),
                    ),
                    (
                        PoolAddressBase::contract(self.raw_raw_2_pair.to_string()),
                        PoolMetadata::constant_product(WYNDEX, vec![raw_asset, raw_2_asset]),
                    ),
                ],
                vec![],
            )
            .unwrap();

        Ok(())
    }
}
