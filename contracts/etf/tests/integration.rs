// #[cfg(test)]
// mod test_utils;

use std::f32::consts::E;

use abstract_boot::{Abstract, AbstractBootError, AppDeployer, ManagerQueryFns};
use abstract_etf::boot::ETF;
use abstract_etf::msg::EtfQueryMsgFns;
use abstract_etf::ETF_ID;
use abstract_os::api::{BaseExecuteMsgFns, BaseQueryMsgFns};
use abstract_os::objects::{AnsAsset, AssetEntry};
use abstract_sdk::os as abstract_os;

use abstract_boot::boot_core::*;
use boot_cw_plus::Cw20;
use cosmwasm_std::{
    coin, coins, to_binary, Addr, Binary, Decimal, Empty, StdResult, Timestamp, Uint128, Uint64,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_asset::Asset;
use cw_staking::CW_STAKING;
use dex::msg::*;
use dex::EXCHANGE;
use speculoos::assert_that;
use speculoos::prelude::OrderedAssertions;

type AResult = anyhow::Result<()>;

use wyndex_bundle::*;

const COMMISSION_TRADER: &str = "commission_receiver";
const ETF_TOKEN: &str = "etf_token";

fn create_etf(mock: Mock) -> Result<ETF<Mock>, AbstractBootError> {
    let version = "1.0.0".parse().unwrap();
    // Deploy abstract
    let abstract_ = Abstract::deploy_on(mock.clone(), version)?;
    // create first OS
    abstract_.os_factory.create_default_os(
        abstract_os::objects::gov_type::GovernanceDetails::Monarchy {
            monarch: mock.sender.to_string(),
        },
    )?;

    // Deploy mock dex
    let wyndex = WynDex::deploy_on(mock.clone(), Empty {})?;
    let eur_asset = AssetEntry::new(EUR);
    let usd_asset = AssetEntry::new(USD);

    let mut etf = ETF::new(ETF_ID, mock.clone());
    etf.deploy(version)?;

    let mut etf_token = Cw20::new(ETF_TOKEN, mock.clone());
    // upload the etf token code
    let etf_toke_code_id = etf_token.upload()?.uploaded_code_id()?;

    // Create an OS that we will turn into a etf
    let os = abstract_.os_factory.create_default_os(
        abstract_os::objects::gov_type::GovernanceDetails::Monarchy {
            monarch: mock.sender.to_string(),
        },
    )?;

    // install etf
    os.manager.install_module(
        ETF_ID,
        &abstract_os::app::InstantiateMsg {
            app: abstract_etf::msg::EtfInstantiateMsg {
                fee: Decimal::percent(5),
                provider_addr: COMMISSION_TRADER.into(),
                token_code_id: etf_toke_code_id,
                token_name: Some("Test ETF Shares".into()),
                token_symbol: Some("TETF".into()),
            },
            base: abstract_os::app::BaseInstantiateMsg {
                ans_host_address: abstract_.ans_host.addr_str()?,
            },
        },
    )?;
    // get its address
    let etf_addr = os.manager.module_addresses(vec![ETF_ID.into()])?.modules[0]
        .1
        .clone();
    // set the address on the contract
    etf.set_address(&Addr::unchecked(etf_addr.clone()));

    // set the etf token address
    let etf_config = etf.state()?;
    etf_token.set_address(&Addr::unchecked(etf_config.liquidity_token));

    Ok(etf {
        os,
        etf,
        etf_token,
        abstract_os: abstract_,
        wyndex,
        dex: exchange_api,
        staking: staking_api,
    })
}

#[test]
fn proper_initialisation() {
    // initialize with non existing pair
    // initialize with non existing fee token
    // initialize with non existing reward token
    // initialize with no pool for the fee token and reward token
}

/// This test covers:
/// - Create a etf and check its configuration setup.
/// - Deposit balanced funds into the auto-compounder and check the minted etf token.
/// - Withdraw a part from the auto-compounder and check the pending claims.
/// - Check that the pending claims are updated after another withdraw.
/// - Batch unbond and check the pending claims are removed.
/// - Withdraw and check the removal of claims.
/// - Check the balances and staked balances.
/// - Withdraw all from the auto-compounder and check the balances again.
#[test]
fn generator_without_reward_proxies_balanced_assets() -> AResult {
    let owner = Addr::unchecked(test_utils::OWNER);

    // create testing environment
    let (_state, mock) = instantiate_default_mock_env(&owner)?;

    // create a etf
    let etf = crate::create_etf(mock.clone())?;
    let WynDex {
        eur_token,
        usd_token,
        eur_usd_lp,
        ..
    } = etf.wyndex;
    let etf_token = etf.etf_token;
    let etf_addr = etf.etf.addr_str()?;
    let eur_asset = AssetEntry::new("eur");
    let usd_asset = AssetEntry::new("usd");
    let asset_infos = vec![eur_token.clone(), usd_token.clone()];

    // check config setup
    let config = etf.etf.config()?;
    assert_that!(config.liquidity_token).is_equal_to(eur_usd_lp.address()?);

    // give user some funds
    mock.set_balances(&[(
        &owner,
        &[
            coin(100_000u128, eur_token.to_string()),
            coin(100_000u128, usd_token.to_string()),
        ],
    )])?;

    // initial deposit must be > 1000 (of both assets)
    // this is set by WynDex
    etf.etf.deposit(
        vec![
            AnsAsset::new(eur_asset, 10000u128),
            AnsAsset::new(usd_asset, 10000u128),
        ],
        &[coin(10000u128, EUR), coin(10000u128, USD)],
    )?;

    // check that the etf token is minted
    let etf_token_balance = etf_token.balance(&owner)?;
    assert_that!(etf_token_balance).is_equal_to(10000u128);

    // and eur balance decreased and usd balance stayed the same
    let balances = mock.query_all_balances(&owner)?;

    // .sort_by(|a, b| a.denom.cmp(&b.denom));
    assert_that!(balances).is_equal_to(vec![
        coin(90_000u128, eur_token.to_string()),
        coin(90_000u128, usd_token.to_string()),
    ]);

    // withdraw part from the auto-compounder
    etf_token.send(&Cw20HookMsg::Redeem {}, 2000, etf_addr.clone())?;
    // check that the etf token decreased
    let etf_token_balance = etf_token.balance(&owner)?;
    let pending_claims: Uint128 = etf.etf.pending_claims(owner.to_string())?;
    assert_that!(etf_token_balance).is_equal_to(8000u128);
    assert_that!(pending_claims.u128()).is_equal_to(2000u128);

    // check that the pending claims are updated
    etf_token.send(&Cw20HookMsg::Redeem {}, 2000, etf_addr.clone())?;
    let pending_claims: Uint128 = etf.etf.pending_claims(owner.to_string())?;
    assert_that!(pending_claims.u128()).is_equal_to(4000u128);

    etf.etf.batch_unbond()?;

    // checks if the pending claims are now removed
    let pending_claims: Uint128 = etf.etf.pending_claims(owner.to_string())?;
    assert_that!(pending_claims.u128()).is_equal_to(0u128);

    mock.next_block()?;
    let claims = etf.etf.claims(owner.to_string())?;
    let unbonding: Expiration = claims[0].unbonding_timestamp;
    if let Expiration::AtTime(time) = unbonding {
        mock.app.borrow_mut().update_block(|b| {
            b.time = time.plus_seconds(10);
        });
    }
    mock.next_block()?;
    etf.etf.withdraw()?;

    // check that the claim is removed
    let claims: Vec<Claim> = etf.etf.claims(owner.to_string())?;
    assert_that!(claims.len()).is_equal_to(0);

    let balances = mock.query_all_balances(&owner)?;
    // .sort_by(|a, b| a.denom.cmp(&b.denom));
    assert_that!(balances).is_equal_to(vec![
        coin(94_000u128, eur_token.to_string()),
        coin(94_000u128, usd_token.to_string()),
    ]);

    let staked = etf
        .wyndex
        .suite
        .query_all_staked(asset_infos, &etf.os.proxy.addr_str()?)?;

    let generator_staked_balance = staked.stakes.first().unwrap();
    assert_that!(generator_staked_balance.stake.u128()).is_equal_to(6000u128);

    // withdraw all from the auto-compounder
    etf_token.send(&Cw20HookMsg::Redeem {}, 6000, etf_addr)?;
    etf.etf.batch_unbond()?;
    mock.wait_blocks(60 * 60 * 24 * 21)?;
    etf.etf.withdraw()?;

    // and eur balance decreased and usd balance stayed the same
    let balances = mock.query_all_balances(&owner)?;

    // .sort_by(|a, b| a.denom.cmp(&b.denom));
    assert_that!(balances).is_equal_to(vec![
        coin(100_000u128, eur_token.to_string()),
        coin(100_000u128, usd_token.to_string()),
    ]);
    Ok(())
}

/// This test covers:
/// - depositing with 2 assets
/// - depositing and withdrawing with a single sided asset
/// - querying the state of the auto-compounder
/// - querying the balance of a users position in the auto-compounder
/// - querying the total lp balance of the auto-compounder
#[test]
fn generator_without_reward_proxies_single_sided() -> AResult {
    let owner = Addr::unchecked(test_utils::OWNER);

    // create testing environment
    let (_state, mock) = instantiate_default_mock_env(&owner)?;

    // create a etf
    let etf = crate::create_etf(mock.clone())?;
    let WynDex {
        eur_token,
        usd_token,
        eur_usd_lp,
        ..
    } = etf.wyndex;
    let etf_token = etf.etf_token;
    let etf_addr = etf.etf.addr_str()?;
    let eur_asset = AssetEntry::new("eur");
    let usd_asset = AssetEntry::new("usd");
    let asset_infos = vec![eur_token.clone(), usd_token.clone()];

    // check config setup
    let config: Config = etf.etf.config()?;
    let position = etf.etf.total_lp_position()?;
    assert_that!(position).is_equal_to(Uint128::zero());

    assert_that!(config.liquidity_token).is_equal_to(eur_usd_lp.address()?);

    // give user some funds
    mock.set_balances(&[(
        &owner,
        &[
            coin(100_000u128, eur_token.to_string()),
            coin(100_000u128, usd_token.to_string()),
        ],
    )])?;

    // initial deposit must be > 1000 (of both assets)
    // this is set by WynDex
    etf.etf.deposit(
        vec![
            AnsAsset::new(eur_asset.clone(), 10000u128),
            AnsAsset::new(usd_asset.clone(), 10000u128),
        ],
        &[coin(10_000u128, EUR), coin(10_000u128, USD)],
    )?;

    let position = etf.etf.total_lp_position()?;
    assert_that!(position).is_greater_than(Uint128::zero());

    // single asset deposit
    etf.etf.deposit(
        vec![AnsAsset::new(eur_asset, 1000u128)],
        &[coin(1000u128, EUR)],
    )?;

    // check that the etf token is minted
    let etf_token_balance = etf_token.balance(&owner)?;
    assert_that!(etf_token_balance).is_equal_to(10487u128);
    let new_position = etf.etf.total_lp_position()?;
    assert_that!(new_position).is_greater_than(position);

    etf.etf.deposit(
        vec![AnsAsset::new(usd_asset, 1000u128)],
        &[coin(1000u128, USD)],
    )?;

    // check that the etf token is increased
    let etf_token_balance = etf_token.balance(&owner)?;
    assert_that!(etf_token_balance).is_equal_to(10986u128);
    // check if the etf balance query functions properly:
    let etf_balance_queried = etf.etf.balance(owner.to_string())?;
    assert_that!(etf_balance_queried).is_equal_to(Uint128::from(etf_token_balance));

    let position = new_position;
    let new_position = etf.etf.total_lp_position()?;
    assert_that!(new_position).is_greater_than(position);

    // and eur balance decreased and usd balance stayed the same
    let balances = mock.query_all_balances(&owner)?;
    assert_that!(balances).is_equal_to(vec![
        coin(89_000u128, eur_token.to_string()),
        coin(89_000u128, usd_token.to_string()),
    ]);

    // withdraw part from the auto-compounder
    etf_token.send(&Cw20HookMsg::Redeem {}, 4986, etf_addr.clone())?;
    // check that the etf token decreased
    let etf_token_balance = etf_token.balance(&owner)?;
    assert_that!(etf_token_balance).is_equal_to(6000u128);

    let pending_claim = etf.etf.pending_claims(owner.to_string())?;
    assert_that!(pending_claim.u128()).is_equal_to(4986u128);
    let etf_token_balance = etf_token.balance(&etf.etf.address()?)?;
    assert_that!(etf_token_balance).is_equal_to(4986u128);

    let total_lp_balance = etf.etf.total_lp_position()?;
    assert_that!(total_lp_balance).is_equal_to(new_position);

    // Batch unbond pending claims
    etf.etf.batch_unbond()?;

    // query the claims of the auto-compounder
    let claims = etf.etf.claims(owner.to_string())?;
    let expected_claim = Claim {
        unbonding_timestamp: Expiration::AtTime(mock.block_info()?.time.plus_seconds(1)),
        amount_of_etf_tokens_to_burn: 4986u128.into(),
        amount_of_lp_tokens_to_unbond: 4985u128.into(),
    };
    assert_that!(claims).is_equal_to(vec![expected_claim]);

    // let the time pass and withdraw the claims
    mock.wait_blocks(60 * 60 * 24 * 10)?;

    // let total_lp_balance = etf.etf.total_lp_position()?;
    // assert_that!(total_lp_balance).is_equal_to(new_position);
    etf.etf.withdraw()?;

    // and eur and usd balance increased
    let balances = mock.query_all_balances(&owner)?;
    assert_that!(balances).is_equal_to(vec![
        coin(93_988u128, eur_token.to_string()),
        coin(93_988u128, usd_token.to_string()),
    ]);

    let position = new_position;
    let new_position = etf.etf.total_lp_position()?;
    assert_that!(new_position).is_less_than(position);

    let generator_staked_balance = etf
        .wyndex
        .suite
        .query_all_staked(asset_infos, &etf.os.proxy.addr_str()?)?
        .stakes[0]
        .stake;
    assert_that!(generator_staked_balance.u128()).is_equal_to(6001u128);

    // withdraw all from the auto-compounder
    etf_token.send(&Cw20HookMsg::Redeem {}, 6000, etf_addr)?;

    // testing general non unbonding staking contract functionality
    let pending_claims = etf.etf.pending_claims(owner.to_string())?.into();
    assert_that!(pending_claims).is_equal_to(6000u128); // no unbonding period, so no pending claims

    etf.etf.batch_unbond()?; // batch unbonding not enabled
    mock.wait_blocks(60 * 60 * 24 * 10)?;
    etf.etf.withdraw()?; // withdraw wont have any effect, because there are no pending claims
                         // mock.next_block()?;

    let balances = mock.query_all_balances(&owner)?;
    assert_that!(balances).is_equal_to(vec![
        coin(99_993u128, eur_token.to_string()),
        coin(99_993u128, usd_token.to_string()),
    ]);

    let new_position = etf.etf.total_lp_position()?;
    assert_that!(new_position).is_equal_to(Uint128::zero());

    Ok(())
}

/// This test covers the following scenario:
/// - create a pool with rewards
/// - deposit into the pool in-balance
/// - compound rewards
/// - checks if the fee distribution is correct
/// - checks if the rewards are distributed correctly
#[test]
fn generator_with_rewards_test_fee_and_reward_distribution() -> AResult {
    let owner = Addr::unchecked(test_utils::OWNER);
    let commission_addr = Addr::unchecked(COMMISSION_RECEIVER);
    let wyndex_owner = Addr::unchecked(WYNDEX_OWNER);

    // create testing environment
    let (_state, mock) = instantiate_default_mock_env(&owner)?;

    // create a etf
    let mut etf = crate::create_etf(mock.clone())?;
    let WynDex {
        eur_token,
        usd_token,
        eur_usd_lp,
        eur_usd_staking,
        ..
    } = etf.wyndex;

    let etf_token = etf.etf_token;
    let etf_addr = etf.etf.addr_str()?;
    let eur_asset = AssetEntry::new("eur");
    let usd_asset = AssetEntry::new("usd");

    // check config setup
    let config = etf.etf.config()?;
    assert_that!(config.liquidity_token).is_equal_to(eur_usd_lp.address()?);

    // give user some funds
    mock.set_balances(&[
        (
            &owner,
            &[
                coin(100_000u128, eur_token.to_string()),
                coin(100_000u128, usd_token.to_string()),
            ],
        ),
        (&wyndex_owner, &[coin(100_000u128, WYND_TOKEN.to_string())]),
    ])?;

    // initial deposit must be > 1000 (of both assets)
    // this is set by WynDex
    etf.etf.deposit(
        vec![
            AnsAsset::new(eur_asset, 100_000u128),
            AnsAsset::new(usd_asset, 100_000u128),
        ],
        &[coin(100_000u128, EUR), coin(100_000u128, USD)],
    )?;

    // query how much lp tokens are in the etf
    let etf_lp_balance = etf.etf.total_lp_position()? as Uint128;

    // check that the etf token is minted
    let etf_token_balance = etf_token.balance(&owner)?;
    assert_that!(etf_token_balance).is_equal_to(100_000u128);
    let ownerbalance = mock.query_balance(&owner, EUR)?;
    assert_that!(ownerbalance.u128()).is_equal_to(0u128);

    // process block -> the AC should have pending rewards at the staking contract
    mock.next_block()?;
    etf.wyndex.suite.distribute_funds(
        eur_usd_staking,
        wyndex_owner.as_str(),
        &coins(1000, WYND_TOKEN),
    )?; // distribute 1000 EUR

    // rewards are 1_000 WYND each block for the entire amount of staked lp.
    // the fee received should be equal to 3% of the rewarded tokens which is then swapped using the astro/EUR pair.
    // the fee is 3% of 1K = 30, rewards are then 970
    // the fee is then swapped using the astro/EUR pair
    // the price of the WYND/EUR pair is 10K:10K
    // which will result in a 29 EUR fee for the autocompounder due to spread + rounding.
    etf.etf.compound()?;

    let commission_received: Uint128 = mock.query_balance(&commission_addr, EUR)?;
    assert_that!(commission_received.u128()).is_equal_to(29u128);

    // The reward for the user is then 970 WYND which is then swapped using the WYND/EUR pair
    // this will be swapped for ~880 EUR, which then is provided using single sided provide_liquidity
    let new_etf_lp_balance: Uint128 = etf.etf.total_lp_position()?;
    let new_lp: Uint128 = new_etf_lp_balance - etf_lp_balance;
    let expected_new_value: Uint128 = Uint128::from(etf_lp_balance.u128() * 4u128 / 1000u128); // 0.4% of the previous position
    assert_that!(new_lp).is_greater_than(expected_new_value);

    let owner_balance_eur = mock.query_balance(&owner, EUR)?;
    let owner_balance_usd = mock.query_balance(&owner, USD)?;

    // Redeem etf tokens and create pending claim of user tokens to see if the user actually received more of EUR and USD then they deposited
    etf_token.send(&Cw20HookMsg::Redeem {}, etf_token_balance, etf_addr)?;

    // Unbond tokens & clear pending claims
    etf.etf.batch_unbond()?;

    mock.wait_blocks(1)?;

    // Withdraw EUR and USD tokens to user
    etf.etf.withdraw()?;

    let new_owner_balance = mock.query_all_balances(&owner)?;
    let eur_diff = new_owner_balance[0].amount.u128() - owner_balance_eur.u128();
    let usd_diff = new_owner_balance[1].amount.u128() - owner_balance_usd.u128();

    // the user should have received more of EUR and USD then they deposited
    assert_that!(eur_diff).is_greater_than(100_000u128); // estimated value
    assert_that!(usd_diff).is_greater_than(100_000u128);

    Ok(())
}

fn generator_with_rewards_test_rewards_distribution_with_multiple_users() -> AResult {
    // test multiple user deposits and withdrawals
    todo!()
}
