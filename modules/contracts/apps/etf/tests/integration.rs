// #[cfg(test)]
// mod test_utils;

use abstract_interface::{
    Abstract, AbstractAccount, AbstractInterfaceError, AppDeployer, DeployStrategy, ProxyExecFns,
    ProxyQueryFns,
};

use abstract_core::{objects::price_source::UncheckedPriceSource, objects::AssetEntry};
use abstract_sdk::core as abstract_core;

use abstract_testing::OWNER;
use cosmwasm_std::{coin, Addr, Decimal, Empty};
use cw20::msg::Cw20ExecuteMsgFns;
use cw_orch::prelude::*;
use cw_plus_interface::cw20_base::Cw20Base as AbstractCw20Base;

use cw20_base::msg::QueryMsgFns;
use cw_asset::{AssetInfo, AssetUnchecked};
use cw_orch::deploy::Deploy;

use etf_app::{
    contract::interface::Etf,
    msg::Cw20HookMsg,
    msg::{EtfExecuteMsgFns, EtfQueryMsgFns},
    ETF_APP_ID,
};
use semver::Version;
use speculoos::prelude::*;

use wyndex_bundle::*;

type AResult = anyhow::Result<()>;

const ETF_MANAGER: &str = "etf_manager";
const ETF_TOKEN: &str = "etf_token";

pub struct EtfEnv<Chain: CwEnv> {
    pub account: AbstractAccount<Chain>,
    pub etf: Etf<Chain>,
    pub share_token: AbstractCw20Base<Chain>,
    pub wyndex: WynDex,
    pub abstract_core: Abstract<Chain>,
}

fn create_etf(mock: Mock) -> Result<EtfEnv<Mock>, AbstractInterfaceError> {
    let version: Version = env!("CARGO_PKG_VERSION").parse().unwrap();
    // Deploy abstract
    let abstract_ = Abstract::deploy_on(mock.clone(), mock.sender.to_string())?;
    // create first AbstractAccount
    abstract_.account_factory.create_default_account(
        abstract_core::objects::gov_type::GovernanceDetails::Monarchy {
            monarch: mock.sender.to_string(),
        },
    )?;

    // Deploy mock dex
    let wyndex = WynDex::deploy_on(mock.clone(), Empty {})?;

    let etf = Etf::new(ETF_APP_ID, mock.clone());
    etf.deploy(version, DeployStrategy::Try)?;

    let etf_token = AbstractCw20Base::new(ETF_TOKEN, mock.clone());
    // upload the etf token code
    let etf_token_code_id = etf_token.upload()?.uploaded_code_id()?;

    // Create an AbstractAccount that we will turn into a etf
    let account = abstract_.account_factory.create_default_account(
        abstract_core::objects::gov_type::GovernanceDetails::Monarchy {
            monarch: mock.sender.to_string(),
        },
    )?;

    // install etf
    account.install_app(
        &etf,
        &etf_app::msg::EtfInstantiateMsg {
            fee: Decimal::percent(5),
            manager_addr: ETF_MANAGER.into(),
            token_code_id: etf_token_code_id,
            token_name: Some("Test ETF Shares".into()),
            token_symbol: Some("TETF".into()),
        },
        None,
    )?;

    // set the etf token address
    let etf_config = etf.state()?;
    etf_token.set_address(&Addr::unchecked(etf_config.share_token_address));

    Ok(EtfEnv {
        account,
        etf,
        share_token: etf_token,
        abstract_core: abstract_,
        wyndex,
    })
}

#[test]
fn proper_initialization() -> AResult {
    let owner = Addr::unchecked(OWNER);

    // create testing environment
    let mock = Mock::new(&owner);

    // create a etf
    let etf_env = crate::create_etf(mock.clone())?;
    let WynDex { usd_token, .. } = etf_env.wyndex;
    let etf = etf_env.etf;
    let etf_token = etf_env.share_token;
    let etf_addr = etf.addr_str()?;
    let proxy = &etf_env.account.proxy;
    let manager = &etf_env.account.manager;

    // Set usd as base asset
    proxy.call_as(&manager.address()?).update_assets(
        vec![(AssetEntry::new("usd"), UncheckedPriceSource::None)],
        vec![],
    )?;
    let base_asset = proxy.base_asset()?;
    assert_that!(base_asset.base_asset).is_equal_to(AssetInfo::native("usd"));

    // check config setup
    let state = etf.state()?;
    assert_that!(state.share_token_address).is_equal_to(etf_token.address()?);

    // give user some funds
    mock.set_balances(&[(&owner, &[coin(1_000u128, usd_token.to_string())])])?;

    etf.deposit(
        AssetUnchecked::new(AssetInfo::native("usd".to_string()), 1000u128),
        &[coin(1_000u128, USD)],
    )?;

    // check that the etf token is minted
    let etf_token_balance = etf_token.balance(owner.to_string())?.balance;
    assert_that!(etf_token_balance.u128()).is_equal_to(1000u128);

    // the proxy contract received the funds
    let balances = mock.query_all_balances(&proxy.address()?)?;
    assert_that!(balances).is_equal_to(vec![coin(1_000u128, usd_token.to_string())]);

    // withdraw from the etf
    etf_token.call_as(&owner).send(
        500u128.into(),
        etf_addr.clone(),
        cosmwasm_std::to_json_binary(&Cw20HookMsg::Claim {})?,
    )?;

    // check that the etf token decreased
    let etf_token_balance = etf_token.balance(owner.to_string())?.balance;
    assert_that!(etf_token_balance.u128()).is_equal_to(500u128);

    // check that the proxy USD balance decreased (by 500 - fee (5%) = 475)))
    let balances = mock.query_all_balances(&proxy.address()?)?;
    assert_that!(balances).is_equal_to(vec![coin(525u128, usd_token.to_string())]);

    // and the owner USD balance increased (by 500 - fee (5%) = 475)
    let balances = mock.query_all_balances(&owner)?;
    assert_that!(balances).is_equal_to(vec![coin(475u128, usd_token.to_string())]);
    Ok(())
}
