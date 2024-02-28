//! Currently you can run only 1 test at a time: `cargo mt`
//! Otherwise you will have too many requests

use abstract_app::objects::{
    pool_id::PoolAddressBase, AssetEntry, PoolMetadata, PoolType, UncheckedContractEntry,
};
use abstract_client::Environment;
use abstract_interface::{Abstract, ExecuteMsgFns};
use anyhow::Ok;
use cosmwasm_std::{coin, Addr};
use cw_asset::AssetInfoUnchecked;
use cw_orch::{daemon::networks::NEUTRON_1, prelude::*};
use cw_orch_clone_testing::CloneTesting;

use abstract_dex_adapter::dex_tester::{DexTester, MockDex};

// testnet addr of abstract
const SENDER: &str = "neutron1kjzpqv393k4g064xh04j4hwy5d0s03wf7dnt4x";

const GENERATOR_ADDR: &str = "neutron1jz58yjay8uq8zkfw95ngyv3m2wfs2zjef9vdz75d9pa46fdtxc5sxtafny";
const FACTORY_ADDR: &str = "neutron1hptk0k5kng7hjy35vmh009qd5m6l33609nypgf2yc6nqnewduqasxplt4e";

const ASSET_A: &str = "test_asset_one";
const ASSET_B: &str = "test_asset_two";
const ASSET_AMOUNT: u128 = 1_000_000_000_000;

pub struct AstroportDex {
    chain: CloneTesting,
    asset_a: (String, cw_asset::AssetInfoUnchecked),
    asset_b: (String, cw_asset::AssetInfoUnchecked),
}

impl MockDex for AstroportDex {
    fn name(&self) -> String {
        "astroport".to_owned()
    }

    fn asset_a(&self) -> (String, cw_asset::AssetInfoUnchecked) {
        self.asset_a.clone()
    }

    fn asset_b(&self) -> (String, cw_asset::AssetInfoUnchecked) {
        self.asset_b.clone()
    }

    fn create_pool(&self) -> anyhow::Result<(PoolAddressBase<String>, PoolMetadata, AssetInfoUnchecked)> {
        let asset_1_astroport = cw_asset_info_to_astroport(&self.asset_a.1);
        let asset_2_astroport = cw_asset_info_to_astroport(&self.asset_b.1);

        // Create pool
        let asset_infos = vec![asset_1_astroport, asset_2_astroport];
        let resp = self.chain.execute(
            &astroport::factory::ExecuteMsg::CreatePair {
                pair_type: astroport::factory::PairType::Xyk {},
                asset_infos,
                init_params: None,
            },
            &[],
            &Addr::unchecked(FACTORY_ADDR),
        )?;
        let pair_contract_addr = resp.event_attr_value("wasm", "pair_contract_addr")?;
        let liquidity_token_addr = resp.event_attr_value("wasm", "liquidity_token_addr")?;

        let abstr_deployment = Abstract::load_from(self.chain.clone())?;
        // Register pair contract address and liquidity token address
        abstr_deployment.ans_host.update_contract_addresses(
            vec![(
                UncheckedContractEntry {
                    protocol: self.name(),
                    contract: format!(
                        "staking/{dex}/{asset_a},{asset_b}",
                        dex = self.name(),
                        asset_a = &self.asset_a.0,
                        asset_b = &self.asset_b.0
                    ),
                },
                GENERATOR_ADDR.to_owned(),
            )],
            vec![],
        )?;

        let addr = Addr::unchecked(pair_contract_addr);

        // Add some liquidity
        let amount = vec![coin(ASSET_AMOUNT, ASSET_A), coin(ASSET_AMOUNT, ASSET_B)];
        self.chain.add_balance(&self.chain.sender, amount.clone())?;
        self.chain.execute(
            &astroport::pair::ExecuteMsg::ProvideLiquidity {
                assets: amount.iter().map(Into::into).collect(),
                slippage_tolerance: None,
                auto_stake: None,
                receiver: None,
            },
            &amount,
            &addr,
        )?;

        let pool = PoolAddressBase::Contract(addr.to_string());
        let pool_metadata = PoolMetadata {
            dex: self.name(),
            pool_type: PoolType::ConstantProduct,
            assets: vec![
                AssetEntry::new(&self.asset_a.0),
                AssetEntry::new(&self.asset_b.0),
            ],
        };
        let lp_asset = AssetInfoUnchecked::Cw20(liquidity_token_addr);
        Ok((pool, pool_metadata, lp_asset))
    }
}

fn cw_asset_info_to_astroport(asset: &cw_asset::AssetInfoUnchecked) -> astroport::asset::AssetInfo {
    match asset {
        cw_asset::AssetInfoBase::Native(denom) => astroport::asset::AssetInfo::NativeToken {
            denom: denom.clone(),
        },
        cw_asset::AssetInfoBase::Cw20(contract_addr) => astroport::asset::AssetInfo::Token {
            contract_addr: Addr::unchecked(contract_addr),
        },
        _ => unreachable!(),
    }
}

// fn cw_asset_to_astroport(asset: &cw_asset::Asset) -> astroport::asset::Asset {
//     match &asset.info {
//         cw_asset::AssetInfoBase::Native(denom) => astroport::asset::Asset {
//             amount: asset.amount,
//             info: astroport::asset::AssetInfo::NativeToken {
//                 denom: denom.clone(),
//             },
//         },
//         cw_asset::AssetInfoBase::Cw20(contract_addr) => astroport::asset::Asset {
//             amount: asset.amount,
//             info: astroport::asset::AssetInfo::Token {
//                 contract_addr: contract_addr.clone(),
//             },
//         },
//         _ => unreachable!(),
//     }
// }

fn setup_native() -> anyhow::Result<DexTester<CloneTesting, AstroportDex>> {
    let chain_info = NEUTRON_1;
    let sender = Addr::unchecked(SENDER);
    let abstr_deployment = super::load_abstr(chain_info, sender)?;
    let chain = abstr_deployment.environment();
    let asset_a = (
        "tao".to_owned(),
        AssetInfoUnchecked::Native(ASSET_A.to_owned()),
    );
    let asset_b = (
        "tat".to_owned(),
        AssetInfoUnchecked::Native(ASSET_B.to_owned()),
    );
    DexTester::new(
        abstr_deployment,
        AstroportDex {
            chain,
            asset_a,
            asset_b,
        },
    )
}

#[test]
fn test_native_swap() -> anyhow::Result<()> {
    let dex_tester = setup_native()?;
    dex_tester.test_swap()?;
    Ok(())
}

#[test]
fn test_native_provide_liquidity() -> anyhow::Result<()> {
    let dex_tester = setup_native()?;
    dex_tester.test_provide_liquidity_two_sided()?;
    dex_tester.test_provide_liquidity_one_sided()?;
    Ok(())
}

#[test]
fn test_native_provide_liquidity_symmetric() -> anyhow::Result<()> {
    let dex_tester = setup_native()?;
    dex_tester.test_provide_liquidity_symmetric()?;
    Ok(())
}