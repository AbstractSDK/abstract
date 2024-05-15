//! Currently you can run only 1 test at a time: `cargo ct`
//! Otherwise you will have too many requests

use abstract_app::objects::{
    pool_id::PoolAddressBase, AssetEntry, PoolMetadata, PoolType, UncheckedContractEntry,
};
use abstract_client::{AbstractClient, Environment};
use abstract_interface::ExecuteMsgFns;
use abstract_modules_interchain_tests::common::load_abstr;
use anyhow::Ok;
use cosmwasm_std::{Addr, Decimal};
use cw_asset::AssetInfoUnchecked;
use cw_orch::daemon::networks::HARPOON_4;
use cw_orch_clone_testing::CloneTesting;

use abstract_dex_adapter::dex_tester::{DexTester, MockDex};

// testnet addr of abstract
const SENDER: &str = "kujira14cl2dthqamgucg9sfvv4relp3aa83e40yjx3f5";

// Instantiation of Bow Market Maker results in
// `Unexpected exec msg Empty from Addr("kujira18ef6ylkdcwdgdzeyuwrtpa9vzc06cmd24crcxnfjt24dn8qn9sfqe0vg6k")`
// And Kujira have closed-source contracts, so we have no way of knowing what's happening inside. Because of that testing done against existing kujira contracts

const FIN_ADDR: &str = "kujira1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrsqq4jjh";
const BOW_MM_ADDR: &str = "kujira19kxd9sqk09zlzqfykk7tzyf70hl009hkekufq8q0ud90ejtqvvxs8xg5cq";
const BOW_ADDR: &str = "kujira1e7hxytqdg6v05f8ev3wrfcm5ecu3qyhl7y4ga73z76yuufnlk2rqd4uwf4";

const ASSET_A: &str = "ukuji";
const ASSET_B: &str = "factory/kujira1ltvwg69sw3c5z99c6rr08hal7v0kdzfxz07yj5/demo";
const LP_DENOM: &str =
    "factory/kujira19kxd9sqk09zlzqfykk7tzyf70hl009hkekufq8q0ud90ejtqvvxs8xg5cq/ulp";
pub struct KujiraDex {
    chain: CloneTesting,
    asset_a: (String, String),
    asset_b: (String, String),
}

impl MockDex for KujiraDex {
    fn name(&self) -> String {
        "fin".to_owned()
    }

    fn asset_a(&self) -> (String, cw_asset::AssetInfoUnchecked) {
        let (asset_entry, denom) = &self.asset_a;
        (
            asset_entry.to_owned(),
            cw_asset::AssetInfoUnchecked::native(denom),
        )
    }

    fn asset_b(&self) -> (String, cw_asset::AssetInfoUnchecked) {
        let (asset_entry, denom) = &self.asset_b;
        (
            asset_entry.to_owned(),
            cw_asset::AssetInfoUnchecked::native(denom),
        )
    }

    fn create_pool(
        &self,
    ) -> anyhow::Result<(PoolAddressBase<String>, PoolMetadata, AssetInfoUnchecked)> {
        // Register pair contract address and liquidity token address
        let abstr_deployment = AbstractClient::new(self.chain.clone())?;
        abstr_deployment.name_service().update_contract_addresses(
            vec![(
                UncheckedContractEntry {
                    protocol: "bow".to_owned(),
                    contract: format!(
                        "staking/{dex}/{asset_a},{asset_b}",
                        dex = self.name(),
                        asset_a = &self.asset_a.0,
                        asset_b = &self.asset_b.0
                    ),
                },
                BOW_ADDR.to_owned(),
            )],
            vec![],
        )?;

        let pool = PoolAddressBase::SeparateAddresses {
            swap: FIN_ADDR.to_owned(),
            liquidity: BOW_MM_ADDR.to_owned(),
        };
        let pool_metadata = PoolMetadata {
            dex: self.name(),
            pool_type: PoolType::ConstantProduct,
            assets: vec![
                AssetEntry::new(&self.asset_a.0),
                AssetEntry::new(&self.asset_b.0),
            ],
        };
        let lp_asset = AssetInfoUnchecked::native(LP_DENOM);
        Ok((pool, pool_metadata, lp_asset))
    }
}

fn setup() -> anyhow::Result<DexTester<CloneTesting, KujiraDex>> {
    let chain_info = HARPOON_4;
    let sender = Addr::unchecked(SENDER);
    let abstr_deployment = load_abstr(chain_info, sender)?;
    let chain = abstr_deployment.environment();
    let asset_a = ("tao".to_owned(), ASSET_A.to_owned());
    let asset_b = ("tat".to_owned(), ASSET_B.to_owned());
    DexTester::new(
        abstr_deployment,
        KujiraDex {
            chain,
            asset_a,
            asset_b,
        },
    )
}

#[test]
fn test_swap() -> anyhow::Result<()> {
    let dex_tester = setup()?;
    dex_tester.test_swap()?;
    Ok(())
}

#[test]
fn test_swap_slippage() -> anyhow::Result<()> {
    let dex_tester = setup()?;
    // This demo pool is 1:1
    dex_tester.test_swap_slippage(Decimal::one(), Decimal::one())?;
    Ok(())
}

#[test]
fn test_queries() -> anyhow::Result<()> {
    let dex_tester = setup()?;
    dex_tester.test_queries()?;
    Ok(())
}

// Can't test liquidity related functionality, because bow market maker uses custom modules

// #[test]
// fn test_provide_liquidity() -> anyhow::Result<()> {
//     let dex_tester = setup()?;
//     dex_tester.test_provide_liquidity_two_sided()?;
//     dex_tester.test_provide_liquidity_one_sided()?;
//     Ok(())
// }

// #[test]
// fn test_provide_liquidity_symmetric() -> anyhow::Result<()> {
//     let dex_tester = setup()?;
//     dex_tester.test_provide_liquidity_symmetric()?;
//     Ok(())
// }

// #[test]
// fn test_provide_liquidity_spread() -> anyhow::Result<()> {
//     let dex_tester = setup()?;
//     dex_tester.test_provide_liquidity_spread()?;
//     Ok(())
// }

// #[test]
// fn test_withdraw_liquidity() -> anyhow::Result<()> {
//     let dex_tester = setup()?;
//     dex_tester.test_withdraw_liquidity()?;
//     Ok(())
// }
