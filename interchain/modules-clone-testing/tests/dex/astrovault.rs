//! Currently you can run only 1 test at a time: `cargo ct`
//! Otherwise you will have too many requests

use abstract_app::objects::{
    pool_id::PoolAddressBase, AssetEntry, PoolMetadata, PoolType, UncheckedContractEntry,
};
use abstract_client::{AbstractClient, Environment};
use abstract_modules_interchain_tests::common::load_abstr;
use anyhow::Ok;
use cosmwasm_std::{coins, Addr};
use cw_asset::AssetInfoUnchecked;
use cw_orch::{daemon::networks::ARCHWAY_1, prelude::*};
use cw_orch_clone_testing::CloneTesting;

use abstract_dex_adapter::dex_tester::{DexTester, MockDex};

// Astrovault uses custom types for creating pools: https://github.com/archway-network/archway/blob/c2f92ce09f7a2e91046ba494546d157ad7f99ded/contracts/go/voter/src/pkg/archway/custom/msg.go
// Meaning we have to use existing pools

const STANDARD_POOL_FACTORY: &str =
    "archway1cq6tgc32az7zpq5w7t2d89taekkn9q95g2g79ka6j46ednw7xkkq7n55a2";
const STABLE_POOL_FACTORY: &str =
    "archway19yzx44k7w7gsjjhumkd4sh9r0z6lscq583hgpu9s4yyl00z9lahq0ptra0";
const RATIO_POOL_FACTORY: &str =
    "archway1zlc00gjw4ecan3tkk5g0lfd78gyfldh4hvkv2g8z5qnwlkz9vqmsdfvs7q";

// mainnet addr of abstract
const SENDER: &str = "archway1kjzpqv393k4g064xh04j4hwy5d0s03wf0exd9k";

pub struct AstrovaultDex {
    // factory_owner: Addr,
    pool_addr: Addr,
    liquidity_token: Addr,
    chain: CloneTesting,
    asset_a: (String, cw_asset::AssetInfoUnchecked),
    asset_b: (String, cw_asset::AssetInfoUnchecked),
}

impl AstrovaultDex {
    fn make_sender_minter(&self) -> anyhow::Result<()> {
        for asset in [&self.asset_a.1, &self.asset_b.1] {
            if let cw_asset::AssetInfoBase::Cw20(addr) = asset {
                let addr = Addr::unchecked(addr);
                let cw20_minter: cw20::MinterResponse =
                    self.chain.query(&cw20::Cw20QueryMsg::Minter {}, &addr)?;
                self.chain
                    .call_as(&Addr::unchecked(cw20_minter.minter))
                    .execute(
                        &cw20::Cw20ExecuteMsg::UpdateMinter {
                            new_minter: Some(self.chain.sender().to_string()),
                        },
                        &[],
                        &addr,
                    )?;
            }
        }
        Ok(())
    }

    // Helpful methods, currently unused, unless astrovault stops using custom modules

    // fn add_sender_balance(&self) -> anyhow::Result<()> {
    //     let chain = &self.chain;

    //     for asset in [&self.asset_a.1, &self.asset_b.1] {
    //         match asset {
    //             cw_asset::AssetInfoBase::Native(denom) => {
    //                 chain.add_balance(&self.chain.sender, coins(ASSET_AMOUNT, denom))?;
    //             }
    //             cw_asset::AssetInfoBase::Cw20(addr) => {
    //                 chain.execute(
    //                     &cw20::Cw20ExecuteMsg::Mint {
    //                         recipient: self.chain.sender.to_string(),
    //                         amount: ASSET_AMOUNT.into(),
    //                     },
    //                     &[],
    //                     &Addr::unchecked(addr),
    //                 )?;
    //             }
    //             _ => unreachable!(),
    //         }
    //     }
    //     Ok(())
    // }

    // fn give_allowance(&self, pair_contract_addr: Addr) -> anyhow::Result<()> {
    //     let chain = &self.chain;

    //     for asset in [&self.asset_a.1, &self.asset_b.1] {
    //         match asset {
    //             cw_asset::AssetInfoBase::Cw20(addr) => {
    //                 chain.execute(
    //                     &cw20::Cw20ExecuteMsg::IncreaseAllowance {
    //                         spender: pair_contract_addr.to_string(),
    //                         amount: ASSET_AMOUNT.into(),
    //                         expires: None,
    //                     },
    //                     &[],
    //                     &Addr::unchecked(addr),
    //                 )?;
    //             }
    //             cw_asset::AssetInfoBase::Native(_) => {}
    //             _ => unreachable!(),
    //         }
    //     }
    //     Ok(())
    // }

    // fn register_native_assets(&self) -> anyhow::Result<()> {
    //     for asset in [&self.asset_a.1, &self.asset_b.1] {
    //         match asset {
    //             cw_asset::AssetInfoBase::Native(denom) => {
    //                 self.chain.call_as(&Addr::unchecked(&self.factory_owner)).execute(
    //                     &astrovault::standard_pool_factory::handle_msg::ExecuteMsg::AddNativeTokenDecimals {
    //                         denom: denom.to_owned(),
    //                         decimals: 6,
    //                     },
    //                     &[],
    //                     &Addr::unchecked(STANDARD_POOL_FACTORY),
    //                 )?;
    //             }
    //             _ => (),
    //         }
    //     }
    //     Ok(())
    // }
}

impl MockDex for AstrovaultDex {
    fn name(&self) -> String {
        "astrovault".to_owned()
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
        // Make sender minter
        self.make_sender_minter()?;

        let pool = PoolAddressBase::Contract(self.pool_addr.to_string());
        let pool_metadata = PoolMetadata {
            dex: self.name(),
            pool_type: PoolType::ConstantProduct,
            assets: vec![
                AssetEntry::new(&self.asset_a.0),
                AssetEntry::new(&self.asset_b.0),
            ],
        };
        let lp_asset = AssetInfoUnchecked::Cw20(self.liquidity_token.to_string());
        Ok((pool, pool_metadata, lp_asset))
    }
}

pub fn cw_asset_info_to_astrovault(
    asset: &cw_asset::AssetInfoUnchecked,
) -> astrovault::assets::asset::AssetInfo {
    match asset {
        cw_asset::AssetInfoBase::Native(denom) => {
            astrovault::assets::asset::AssetInfo::NativeToken {
                denom: denom.clone(),
            }
        }
        cw_asset::AssetInfoBase::Cw20(contract_addr) => {
            astrovault::assets::asset::AssetInfo::Token {
                contract_addr: contract_addr.to_owned(),
            }
        }
        _ => unreachable!(),
    }
}

mod standard_pool_tests {
    // "asset_decimals": [18, 6 ],
    // "asset_infos": [
    //         {
    //           "token": {
    //             "contract_addr": "archway1cutfh7m87cyq5qgqqw49f289qha7vhsg6wtr6rl5fvm28ulnl9ssg0vk0n"
    //           }
    //         },
    //         {
    //           "token": {
    //             "contract_addr": "archway1yjdgfut7jkq5xwzyp6p5hs7hdkmszn34zkhun6mglu3falq3yh8sdkaj7j"
    //           }
    //         }
    //       ],
    //       "cashback": "archway14cdu335ljp6rst337070nejhg7h0j2az7zmx8q0sah88s4uhcczq20fv84",
    //       "contract_addr": "archway139hgd4rm3xyuqyrn63ardjxkg7puzafne7u3pj04qag7ld9cyhnqk9540y",
    //       "liquidity_token": "archway1j5vevvsrm5ayqmfvhng7rkkgjqad37pk35j3nanzmevlq4ntwpfqayv6z4",
    //       "lp_staking": "archway1kzqddgfzdma4pxeh78207k6nakcqjluyu3xum4twpcfe6c6dpdyq2mmuf0"

    const ASSET_A: &str = "archway>xjklv2";
    const ASSET_B: &str = "archway>xarchv2";
    const ASSET_A_ADDR: &str = "archway1yjdgfut7jkq5xwzyp6p5hs7hdkmszn34zkhun6mglu3falq3yh8sdkaj7j";
    const ASSET_B_ADDR: &str = "archway1cutfh7m87cyq5qgqqw49f289qha7vhsg6wtr6rl5fvm28ulnl9ssg0vk0n";
    const STANDARD_POOL_ADDR: &str =
        "archway139hgd4rm3xyuqyrn63ardjxkg7puzafne7u3pj04qag7ld9cyhnqk9540y";
    const LIQUIDITY_TOKEN: &str =
        "archway1j5vevvsrm5ayqmfvhng7rkkgjqad37pk35j3nanzmevlq4ntwpfqayv6z4";

    use cosmwasm_std::Decimal;

    use super::*;

    fn setup_standard_pool() -> anyhow::Result<DexTester<CloneTesting, AstrovaultDex>> {
        let chain_info = ARCHWAY_1;
        let sender = Addr::unchecked(SENDER);
        let abstr_deployment = load_abstr(chain_info, sender)?;
        let chain = abstr_deployment.environment();

        let asset_a = (
            ASSET_A.to_owned(),
            AssetInfoUnchecked::Cw20(ASSET_A_ADDR.to_owned()),
        );
        let asset_b = (
            ASSET_B.to_owned(),
            AssetInfoUnchecked::Cw20(ASSET_B_ADDR.to_owned()),
        );
        DexTester::new(
            abstr_deployment,
            AstrovaultDex {
                pool_addr: Addr::unchecked(STANDARD_POOL_ADDR),
                liquidity_token: Addr::unchecked(LIQUIDITY_TOKEN),
                chain,
                asset_a,
                asset_b,
            },
        )
    }

    #[test]
    fn test_swap() -> anyhow::Result<()> {
        let dex_tester = setup_standard_pool()?;
        dex_tester.test_swap()?;
        Ok(())
    }

    #[test]
    // TODO: Something weird inside astrovault contract
    #[ignore = "Generic error: Generic error: Parsing u256: (bnum) attempt to parse integer from string containing invalid digit"]
    fn test_swap_slippage() -> anyhow::Result<()> {
        let dex_tester = setup_standard_pool()?;
        let pool_response: astrovault::standard_pool::query_msg::PoolResponse =
            dex_tester.dex.chain.query(
                &astrovault::standard_pool::query_msg::QueryMsg::Pool {},
                &dex_tester.dex.pool_addr,
            )?;

        let belief_price_a_to_b = Decimal::from_ratio(
            pool_response.assets[1].amount,
            pool_response.assets[0].amount,
        );
        let belief_price_b_to_a = Decimal::from_ratio(
            pool_response.assets[0].amount,
            pool_response.assets[1].amount,
        );

        dex_tester.test_swap_slippage(belief_price_a_to_b, belief_price_b_to_a)?;
        Ok(())
    }

    #[test]
    fn test_provide_liquidity() -> anyhow::Result<()> {
        let dex_tester = setup_standard_pool()?;
        let provide_value_a = cosmwasm_std::Uint128::new(40_000);
        let simulate_response: astrovault::standard_pool::query_msg::SimulationResponse =
            dex_tester.dex.chain.query(
                &astrovault::standard_pool::query_msg::QueryMsg::Simulation {
                    offer_asset: astrovault::assets::asset::Asset {
                        info: cw_asset_info_to_astrovault(&dex_tester.dex.asset_a.1),
                        amount: provide_value_a,
                    },
                },
                &dex_tester.dex.pool_addr,
            )?;

        let provide_value_b = simulate_response.return_amount + simulate_response.spread_amount;

        dex_tester.test_provide_liquidity_two_sided(
            Some(provide_value_a.u128()),
            Some(provide_value_b.u128()),
        )?;
        dex_tester.test_provide_liquidity_one_sided()?;
        dex_tester.test_provide_liquidity_symmetric(
            Some(provide_value_a.u128()),
            Some(provide_value_b.u128()),
        )?;
        Ok(())
    }

    #[test]
    fn test_provide_liquidity_spread() -> anyhow::Result<()> {
        let dex_tester = setup_standard_pool()?;
        dex_tester.test_provide_liquidity_spread()?;
        Ok(())
    }

    #[test]
    fn test_withdraw_liquidity() -> anyhow::Result<()> {
        let dex_tester = setup_standard_pool()?;

        let provide_value_a = cosmwasm_std::Uint128::new(40_000);
        let simulate_response: astrovault::standard_pool::query_msg::SimulationResponse =
            dex_tester.dex.chain.query(
                &astrovault::standard_pool::query_msg::QueryMsg::Simulation {
                    offer_asset: astrovault::assets::asset::Asset {
                        info: cw_asset_info_to_astrovault(&dex_tester.dex.asset_a.1),
                        amount: provide_value_a,
                    },
                },
                &dex_tester.dex.pool_addr,
            )?;

        let provide_value_b = simulate_response.return_amount + simulate_response.spread_amount;

        dex_tester
            .test_withdraw_liquidity(Some(provide_value_a.u128()), Some(provide_value_b.u128()))?;
        Ok(())
    }

    #[test]
    fn test_queries() -> anyhow::Result<()> {
        let dex_tester = setup_standard_pool()?;
        dex_tester.test_queries()?;
        Ok(())
    }
}
