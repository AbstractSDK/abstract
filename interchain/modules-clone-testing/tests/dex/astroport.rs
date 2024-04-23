//! Currently you can run only 1 test at a time: `cargo ct`
//! Otherwise you will have too many requests

use abstract_app::objects::{
    pool_id::PoolAddressBase, AssetEntry, PoolMetadata, PoolType, UncheckedContractEntry,
};
use abstract_client::{AbstractClient, Environment};
use abstract_interface::ExecuteMsgFns;
use abstract_modules_interchain_tests::common::load_abstr;
use anyhow::Ok;
use cosmwasm_std::{coins, Addr};
use cw_asset::AssetInfoUnchecked;
use cw_orch::{daemon::networks::NEUTRON_1, prelude::*};
use cw_orch_clone_testing::CloneTesting;

use abstract_dex_adapter::dex_tester::{DexTester, MockDex};

// mainnet addr of abstract
const SENDER: &str = "neutron1kjzpqv393k4g064xh04j4hwy5d0s03wf7dnt4x";

// https://docs.astroport.fi/docs/develop/smart-contracts/contract-addresses#neutron
pub const INCENTIVES_ADDR: &str =
    "neutron173fd8wpfzyqnfnpwq2zhtgdstujrjz2wkprkjfr6gqg4gknctjyq6m3tch";
pub const FACTORY_ADDR: &str = "neutron1hptk0k5kng7hjy35vmh009qd5m6l33609nypgf2yc6nqnewduqasxplt4e";

const ASSET_A: &str = "test-asset-one";
const ASSET_B: &str = "test-asset-two";
const ASSET_AMOUNT: u128 = 1_000_000_000_000_000_000;

pub struct AstroportDex {
    chain: CloneTesting,
    asset_a: (String, cw_asset::AssetInfoUnchecked),
    asset_b: (String, cw_asset::AssetInfoUnchecked),
}

impl AstroportDex {
    fn add_sender_balance(&self) -> anyhow::Result<()> {
        let chain = &self.chain;

        for asset in [&self.asset_a.1, &self.asset_b.1] {
            match asset {
                cw_asset::AssetInfoBase::Native(denom) => {
                    chain.add_balance(&self.chain.sender, coins(ASSET_AMOUNT, denom))?;
                }
                cw_asset::AssetInfoBase::Cw20(addr) => {
                    chain.execute(
                        &cw20::Cw20ExecuteMsg::Mint {
                            recipient: self.chain.sender.to_string(),
                            amount: ASSET_AMOUNT.into(),
                        },
                        &[],
                        &Addr::unchecked(addr),
                    )?;
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    fn give_allowance(&self, pair_contract_addr: Addr) -> anyhow::Result<()> {
        let chain = &self.chain;

        for asset in [&self.asset_a.1, &self.asset_b.1] {
            match asset {
                cw_asset::AssetInfoBase::Cw20(addr) => {
                    chain.execute(
                        &cw20::Cw20ExecuteMsg::IncreaseAllowance {
                            spender: pair_contract_addr.to_string(),
                            amount: ASSET_AMOUNT.into(),
                            expires: None,
                        },
                        &[],
                        &Addr::unchecked(addr),
                    )?;
                }
                cw_asset::AssetInfoBase::Native(_) => {}
                _ => unreachable!(),
            }
        }
        Ok(())
    }
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

    fn create_pool(
        &self,
    ) -> anyhow::Result<(PoolAddressBase<String>, PoolMetadata, AssetInfoUnchecked)> {
        let asset_1_astroport = cw_asset_info_to_astroport(&self.asset_a.1);
        let asset_2_astroport = cw_asset_info_to_astroport(&self.asset_b.1);

        // Create pool
        let asset_infos = vec![asset_1_astroport.clone(), asset_2_astroport.clone()];
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

        let abstr_deployment = AbstractClient::new(self.chain.clone())?;
        // Register pair contract address and liquidity token address
        abstr_deployment.name_service().update_contract_addresses(
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
                INCENTIVES_ADDR.to_owned(),
            )],
            vec![],
        )?;

        let addr = Addr::unchecked(pair_contract_addr);

        // Add some liquidity
        let assets = vec![
            astroport::asset::Asset::new(asset_1_astroport, ASSET_AMOUNT),
            astroport::asset::Asset::new(asset_2_astroport, ASSET_AMOUNT),
        ];
        let amount = coins_in_astroport_assets(&assets);
        self.add_sender_balance()?;
        self.give_allowance(addr.clone())?;
        self.chain.execute(
            &astroport::pair::ExecuteMsg::ProvideLiquidity {
                assets,
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

pub fn cw_asset_info_to_astroport(
    asset: &cw_asset::AssetInfoUnchecked,
) -> astroport::asset::AssetInfo {
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

fn coins_in_astroport_assets(assets: &[astroport::asset::Asset]) -> Vec<Coin> {
    let mut coins = cosmwasm_std::Coins::default();
    for asset in assets {
        if let astroport::asset::AssetInfo::NativeToken { denom } = &asset.info {
            coins
                .add(Coin::new(asset.amount.u128(), denom.clone()))
                .unwrap();
        }
    }
    coins.into_vec()
}

mod native_tests {

    use cosmwasm_std::Decimal;

    use super::*;

    fn setup_native() -> anyhow::Result<DexTester<CloneTesting, AstroportDex>> {
        let chain_info = NEUTRON_1;
        let sender = Addr::unchecked(SENDER);
        let abstr_deployment = load_abstr(chain_info, sender)?;
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
    fn test_swap() -> anyhow::Result<()> {
        let dex_tester = setup_native()?;
        dex_tester.test_swap()?;
        Ok(())
    }

    #[test]
    fn test_swap_slippage() -> anyhow::Result<()> {
        let dex_tester = setup_native()?;
        dex_tester.test_swap_slippage(Decimal::one(), Decimal::one())?;
        Ok(())
    }

    #[test]
    fn test_provide_liquidity() -> anyhow::Result<()> {
        let dex_tester = setup_native()?;
        dex_tester.test_provide_liquidity_two_sided(None, None)?;
        dex_tester.test_provide_liquidity_one_sided()?;
        Ok(())
    }

    #[test]
    fn test_provide_liquidity_symmetric() -> anyhow::Result<()> {
        let dex_tester = setup_native()?;
        dex_tester.test_provide_liquidity_symmetric(None, None)?;
        Ok(())
    }

    #[test]
    fn test_provide_liquidity_spread() -> anyhow::Result<()> {
        let dex_tester = setup_native()?;
        dex_tester.test_provide_liquidity_spread()?;
        Ok(())
    }

    #[test]
    fn test_withdraw_liquidity() -> anyhow::Result<()> {
        let dex_tester = setup_native()?;
        dex_tester.test_withdraw_liquidity(None, None)?;
        Ok(())
    }

    #[test]
    fn test_queries() -> anyhow::Result<()> {
        let dex_tester = setup_native()?;
        dex_tester.test_queries()?;
        Ok(())
    }
}

mod cw20_tests {
    use cosmwasm_std::Decimal;

    use super::*;

    fn setup_cw20() -> anyhow::Result<DexTester<CloneTesting, AstroportDex>> {
        let chain_info = NEUTRON_1;
        let sender = Addr::unchecked(SENDER);
        let abstr_deployment = load_abstr(chain_info, sender)?;
        let chain = abstr_deployment.environment();
        let cw20_a = abstr_deployment
            .cw20_builder(ASSET_A, "symbol-a", 6)
            .mint(abstract_client::builder::cw20_builder::MinterResponse {
                minter: chain.sender.to_string(),
                cap: None,
            })
            .instantiate_with_id("cw20_a")?;
        let cw20_b = abstr_deployment
            .cw20_builder(ASSET_B, "symbol-b", 6)
            .mint(abstract_client::builder::cw20_builder::MinterResponse {
                minter: chain.sender.to_string(),
                cap: None,
            })
            .instantiate_with_id("cw20_b")?;

        let asset_a = (
            "tao".to_owned(),
            AssetInfoUnchecked::Cw20(cw20_a.addr_str()?),
        );
        let asset_b = (
            "tat".to_owned(),
            AssetInfoUnchecked::Cw20(cw20_b.addr_str()?),
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
    fn test_swap() -> anyhow::Result<()> {
        let dex_tester = setup_cw20()?;
        dex_tester.test_swap()?;
        Ok(())
    }

    #[test]
    fn test_swap_slippage() -> anyhow::Result<()> {
        let dex_tester = setup_cw20()?;
        dex_tester.test_swap_slippage(Decimal::one(), Decimal::one())?;
        Ok(())
    }

    #[test]
    fn test_provide_liquidity() -> anyhow::Result<()> {
        let dex_tester = setup_cw20()?;
        dex_tester.test_provide_liquidity_two_sided(None, None)?;
        dex_tester.test_provide_liquidity_one_sided()?;
        Ok(())
    }

    #[test]
    fn test_provide_liquidity_symmetric() -> anyhow::Result<()> {
        let dex_tester = setup_cw20()?;
        dex_tester.test_provide_liquidity_symmetric(None, None)?;
        Ok(())
    }

    #[test]
    fn test_provide_liquidity_spread() -> anyhow::Result<()> {
        let dex_tester = setup_cw20()?;
        dex_tester.test_provide_liquidity_spread()?;
        Ok(())
    }

    #[test]
    fn test_withdraw_liquidity() -> anyhow::Result<()> {
        let dex_tester = setup_cw20()?;
        dex_tester.test_withdraw_liquidity(None, None)?;
        Ok(())
    }

    #[test]
    fn test_queries() -> anyhow::Result<()> {
        let dex_tester = setup_cw20()?;
        dex_tester.test_queries()?;
        Ok(())
    }
}
