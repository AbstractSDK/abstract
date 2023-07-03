// // #[cfg(test)]
// // mod test_utils;
//
// use abstract_boot::{
//     Abstract, AbstractAccount, AbstractBootError, AppDeployer, ManagerQueryFns, ProxyExecFns,
//     ProxyQueryFns,
// };
// use abstract_etf::{
//     boot::ETF,
//     msg::{Cw20HookMsg, EtfExecuteMsgFns, EtfQueryMsgFns},
//     ETF_ID,
// };
//
// use abstract_core::{objects::price_source::UncheckedPriceSource, objects::AssetEntry};
// use abstract_sdk::core as abstract_core;
//
// use abstract_boot::boot_core::*;
// use abstract_testing::prelude::TEST_ADMIN;
// use boot_cw_plus::{Cw20Base, Cw20ExecuteMsgFns, Cw20QueryMsgFns};
// use cosmwasm_std::{coin, Addr, Decimal, Empty};
//
// use cw_asset::{AssetInfo, AssetUnchecked};
//
// use semver::Version;
// use speculoos::assert_that;
//
// use wyndex_bundle::*;
//
// type AResult = anyhow::Result<()>;
//
// const ETF_MANAGER: &str = "etf_manager";
// const ETF_TOKEN: &str = "etf_token";
//
// pub struct EtfEnv<Chain: CwEnv> {
//     pub os: AbstractAccount<Chain>,
//     pub etf: ETF<Chain>,
//     pub share_token: Cw20Base<Chain>,
//     pub wyndex: WynDex,
//     pub abstract_core: Abstract<Chain>,
// }
//
// fn create_etf(mock: Mock) -> Result<EtfEnv<Mock>, AbstractBootError> {
//     let version: Version = "1.0.0".parse().unwrap();
//     // Deploy abstract
//     let abstract_ = Abstract::deploy_on(mock.clone(), version.clone())?;
//     // create first AbstractAccount
//     abstract_.account_factory.create_default_account(
//         abstract_core::objects::gov_type::GovernanceDetails::Monarchy {
//             monarch: mock.sender.to_string(),
//         },
//     )?;
//
//     // Deploy mock dex
//     let wyndex = WynDex::deploy_on(mock.clone(), Empty {})?;
//
//     let mut etf = ETF::new(ETF_ID, mock.clone());
//     etf.deploy(version)?;
//
//     let mut etf_token = Cw20Base::new(ETF_TOKEN, mock.clone());
//     // upload the etf token code
//     let etf_toke_code_id = etf_token.upload()?.uploaded_code_id()?;
//
//     // Create an AbstractAccount that we will turn into a etf
//     let os = abstract_.account_factory.create_default_account(
//         abstract_core::objects::gov_type::GovernanceDetails::Monarchy {
//             monarch: mock.sender.to_string(),
//         },
//     )?;
//
//     // install etf
//     os.manager.install_module(
//         ETF_ID,
//         &abstract_core::app::InstantiateMsg {
//             module: abstract_etf::msg::EtfInstantiateMsg {
//                 fee: Decimal::percent(5),
//                 manager_addr: ETF_MANAGER.into(),
//                 token_code_id: etf_toke_code_id,
//                 token_name: Some("Test ETF Shares".into()),
//                 token_symbol: Some("TETF".into()),
//             },
//             base: abstract_core::app::BaseInstantiateMsg {
//                 ans_host_address: abstract_.ans_host.addr_str()?,
//             },
//         },
//     )?;
//     // get its address
//     let etf_addr = os.manager.module_addresses(vec![ETF_ID.into()])?.modules[0]
//         .1
//         .clone();
//     // set the address on the contract
//     etf.set_address(&Addr::unchecked(etf_addr.clone()));
//
//     // set the etf token address
//     let etf_config = etf.state()?;
//     etf_token.set_address(&Addr::unchecked(etf_config.share_token_address));
//
//     Ok(EtfEnv {
//         os,
//         etf,
//         share_token: etf_token,
//         abstract_core: abstract_,
//         wyndex,
//     })
// }
//
// #[test]
// fn proper_initialization() -> AResult {
//     let owner = Addr::unchecked(TEST_ADMIN);
//
//     // create testing environment
//     let (_state, mock) = instantiate_default_mock_env(&owner)?;
//
//     // create a etf
//     let etf_env = crate::create_etf(mock.clone())?;
//     let WynDex { usd_token, .. } = etf_env.wyndex;
//     let etf = etf_env.etf;
//     let etf_token = etf_env.share_token;
//     let etf_addr = etf.addr_str()?;
//     let proxy = &etf_env.os.proxy;
//     let manager = &etf_env.os.manager;
//
//     // Set usd as base asset
//     proxy.call_as(&manager.address()?).update_assets(
//         vec![(AssetEntry::new("usd"), UncheckedPriceSource::None)],
//         vec![],
//     )?;
//     let base_asset = proxy.base_asset()?;
//     assert_that!(base_asset.base_asset).is_equal_to(AssetInfo::native("usd"));
//
//     // check config setup
//     let state = etf.state()?;
//     assert_that!(state.share_token_address).is_equal_to(etf_token.addr_str()?);
//
//     // give user some funds
//     mock.set_balances(&[(&owner, &[coin(1_000u128, usd_token.to_string())])])?;
//
//     etf.deposit(
//         AssetUnchecked::new(AssetInfo::native("usd".to_string()), 1000u128),
//         &[coin(1_000u128, USD)],
//     )?;
//
//     // check that the etf token is minted
//     let etf_token_balance = etf_token.balance(owner.to_string())?;
//     assert_that!(etf_token_balance).is_equal_to(1000u128);
//
//     // the proxy contract received the funds
//     let balances = mock.query_all_balances(&proxy.address()?)?;
//     assert_that!(balances).is_equal_to(vec![coin(1_000u128, usd_token.to_string())]);
//
//     // withdraw from the etf
//     etf_token.send(500u128.into(), etf_addr.clone(), cosmwasm_std::to_binary(&Cw20HookMsg::Claim {})?)?;
//     // check that the etf token decreased
//     let etf_token_balance = etf_token.balance(owner.to_string())?;
//     assert_that!(etf_token_balance).is_equal_to(500u128);
//     // check that the proxy USD balance decreased (by 500 - fee (5%) = 475)))
//     let balances = mock.query_all_balances(&proxy.address()?)?;
//     assert_that!(balances).is_equal_to(vec![coin(525u128, usd_token.to_string())]);
//
//     // and the owner USD balance increased (by 500 - fee (5%) = 475)
//     let balances = mock.query_all_balances(&owner)?;
//     assert_that!(balances).is_equal_to(vec![coin(475u128, usd_token.to_string())]);
//     Ok(())
// }
