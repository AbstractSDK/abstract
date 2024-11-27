//! Currently you can run only 1 test at a time: `cargo ct`
//! Otherwise you will have too many requests

use abstract_app::objects::{pool_id::PoolAddressBase, AssetEntry, PoolMetadata, PoolType};
use abstract_client::Environment;
use abstract_modules_interchain_tests::common::load_abstr;
use anyhow::Ok;
use cosmwasm_std::Addr;
use cw_asset::AssetInfoUnchecked;
use cw_orch::{daemon::networks::ARCHWAY_1, prelude::*};
use cw_orch_clone_testing::CloneTesting;

use abstract_dex_adapter::dex_tester::{DexTester, MockDex};

// Astrovault uses custom types for creating pools: https://github.com/archway-network/archway/blob/c2f92ce09f7a2e91046ba494546d157ad7f99ded/contracts/go/voter/src/pkg/archway/custom/msg.go
// Meaning we have to use existing pools

// const STANDARD_POOL_FACTORY: &str =
//     "archway1cq6tgc32az7zpq5w7t2d89taekkn9q95g2g79ka6j46ednw7xkkq7n55a2";
// const STABLE_POOL_FACTORY: &str =
//     "archway19yzx44k7w7gsjjhumkd4sh9r0z6lscq583hgpu9s4yyl00z9lahq0ptra0";
// const RATIO_POOL_FACTORY: &str =
//     "archway1zlc00gjw4ecan3tkk5g0lfd78gyfldh4hvkv2g8z5qnwlkz9vqmsdfvs7q";

pub struct AstrovaultDex {
    // factory_owner: Addr,
    pool_addr: Addr,
    liquidity_token: Addr,
    chain: CloneTesting,
    pool_type: PoolType,
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
                            new_minter: Some(self.chain.sender_addr().to_string()),
                        },
                        &[],
                        &addr,
                    )?;
            }
        }
        Ok(())
    }
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
            pool_type: self.pool_type,
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

    use super::*;
    use abstract_dex_adapter::{interface::DexAdapter, msg::DexInstantiateMsg, DEX_ADAPTER_ID};
    use abstract_interface::AdapterDeployer;
    use abstract_interface::DeployStrategy;
    use cosmwasm_std::Decimal;

    fn setup_standard_pool() -> anyhow::Result<DexTester<CloneTesting, AstrovaultDex>> {
        let chain_info = ARCHWAY_1;
        let abstr_deployment = load_abstr(chain_info)?;
        // Deploy the dex adapter
        DexAdapter::new(DEX_ADAPTER_ID, abstr_deployment.environment()).deploy(
            abstract_dex_adapter::contract::CONTRACT_VERSION.parse()?,
            DexInstantiateMsg {
                recipient_account: 0,
                swap_fee: Decimal::permille(3),
            },
            DeployStrategy::Try,
        )?;
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
                pool_type: PoolType::ConstantProduct,
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

    // TODO: Slippage got deprecated on astrovault in favor of "expected_return"
    // #[test]
    // #[ignore]
    // fn test_swap_slippage() -> anyhow::Result<()> {
    //     let dex_tester = setup_standard_pool()?;
    //     let pool_response: astrovault::standard_pool::query_msg::PoolResponse =
    //         dex_tester.dex.chain.query(
    //             &astrovault::standard_pool::query_msg::QueryMsg::Pool {},
    //             &dex_tester.dex.pool_addr,
    //         )?;

    //     let amount_a =
    //         astrovault::utils::normalize_amount(&pool_response.assets[1].amount, &6).unwrap();
    //     let amount_b =
    //         astrovault::utils::normalize_amount(&pool_response.assets[0].amount, &18).unwrap();

    //     let belief_price_a_to_b = Decimal::from_ratio(amount_a, amount_b);
    //     let belief_price_b_to_a = Decimal::from_ratio(amount_b, amount_a);

    //     dex_tester.test_swap_slippage(belief_price_a_to_b, belief_price_b_to_a)?;
    //     Ok(())
    // }

    #[test]
    fn test_provide_liquidity() -> anyhow::Result<()> {
        let dex_tester = setup_standard_pool()?;
        let provide_value_a = 40_000_u128;
        let simulate_response: astrovault::standard_pool::query_msg::SimulationResponse =
            dex_tester.dex.chain.query(
                &astrovault::standard_pool::query_msg::QueryMsg::Simulation {
                    offer_asset: astrovault::assets::asset::Asset {
                        info: cw_asset_info_to_astrovault(&dex_tester.dex.asset_a.1),
                        amount: provide_value_a.into(),
                    },
                },
                &dex_tester.dex.pool_addr,
            )?;

        let provide_value_b = simulate_response.return_amount + simulate_response.spread_amount;

        dex_tester.test_provide_liquidity_two_sided(
            Some(provide_value_a),
            Some(provide_value_b.u128()),
        )?;
        dex_tester.test_provide_liquidity_one_sided()?;
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

        let provide_value_a = 40_000_u128;
        let simulate_response: astrovault::standard_pool::query_msg::SimulationResponse =
            dex_tester.dex.chain.query(
                &astrovault::standard_pool::query_msg::QueryMsg::Simulation {
                    offer_asset: astrovault::assets::asset::Asset {
                        info: cw_asset_info_to_astrovault(&dex_tester.dex.asset_a.1),
                        amount: provide_value_a.into(),
                    },
                },
                &dex_tester.dex.pool_addr,
            )?;

        let provide_value_b = simulate_response.return_amount + simulate_response.spread_amount;

        dex_tester.test_withdraw_liquidity(
            Some(provide_value_a),
            Some(provide_value_b.u128()),
            None,
        )?;
        Ok(())
    }

    #[test]
    fn test_queries() -> anyhow::Result<()> {
        let dex_tester = setup_standard_pool()?;
        dex_tester.test_queries()?;
        Ok(())
    }
}

mod xasset_stable_pool_tests {
    //   "asset_decimals": [
    //     18,
    //     18
    //   ],
    //   "asset_infos": [
    //     {
    //       "native_token": {
    //         "denom": "aarch"
    //       }
    //     },
    //     {
    //       "token": {
    //         "contract_addr": "archway1cutfh7m87cyq5qgqqw49f289qha7vhsg6wtr6rl5fvm28ulnl9ssg0vk0n"
    //       }
    //     }
    //   ],
    //   "cashback": "archway14cdu335ljp6rst337070nejhg7h0j2az7zmx8q0sah88s4uhcczq20fv84",
    //   "contract_addr": "archway1vq9jza8kuz80f7ypyvm3pttvpcwlsa5fvum9hxhew5u95mffknxsjy297r",
    //   "liquidity_token": "archway123h0jfnk3rhhuapkytrzw22u6w4xkf563lqhy42a9r5lmv32w73s8f6ql2",
    //   "lockups": "archway1qydzzm0tnta98v9tk8fd3rwnhxwwjlz8sqdsy4z6w0hu7yyq7jpsvk7dyk",
    //   "lp_staking": "archway13xeat9u6s0x7vphups0r096fl3tkr3zenhdvfjrsc2t0t70ayugscdw46g"

    const ASSET_A: &str = "archway>archv2";
    const ASSET_B: &str = "archway>xarchv2";
    const ASSET_A_DENOM: &str = "aarch";
    const ASSET_B_ADDR: &str = "archway1cutfh7m87cyq5qgqqw49f289qha7vhsg6wtr6rl5fvm28ulnl9ssg0vk0n";
    const STABLE_POOL_ADDR: &str =
        "archway1vq9jza8kuz80f7ypyvm3pttvpcwlsa5fvum9hxhew5u95mffknxsjy297r";
    const LIQUIDITY_TOKEN: &str =
        "archway123h0jfnk3rhhuapkytrzw22u6w4xkf563lqhy42a9r5lmv32w73s8f6ql2";

    use super::*;

    fn setup_stable_pool() -> anyhow::Result<DexTester<CloneTesting, AstrovaultDex>> {
        let chain_info = ARCHWAY_1;
        let abstr_deployment = load_abstr(chain_info)?;
        let chain = abstr_deployment.environment();

        let asset_a = (
            ASSET_A.to_owned(),
            AssetInfoUnchecked::Native(ASSET_A_DENOM.to_owned()),
        );
        let asset_b = (
            ASSET_B.to_owned(),
            AssetInfoUnchecked::Cw20(ASSET_B_ADDR.to_owned()),
        );
        DexTester::new(
            abstr_deployment,
            AstrovaultDex {
                pool_addr: Addr::unchecked(STABLE_POOL_ADDR),
                liquidity_token: Addr::unchecked(LIQUIDITY_TOKEN),
                pool_type: PoolType::Stable,
                chain,
                asset_a,
                asset_b,
            },
        )
    }

    #[test]
    fn test_swap() -> anyhow::Result<()> {
        let dex_tester = setup_stable_pool()?;
        dex_tester.test_swap()?;
        Ok(())
    }

    // Skipping slippage swap test as it's not applicable to stable pool

    #[test]
    fn test_provide_liquidity() -> anyhow::Result<()> {
        let dex_tester = setup_stable_pool()?;

        let asset_to_provide = AssetEntry::new(&dex_tester.dex.asset_a.0);
        dex_tester.test_provide_liquidity_one_direction(asset_to_provide)?;
        Ok(())
    }

    // Skipping slippage provide_liquidity test as it's not applicable to stable pool

    #[test]
    fn test_withdraw_liquidity() -> anyhow::Result<()> {
        let dex_tester = setup_stable_pool()?;

        dex_tester.test_withdraw_liquidity(
            Some(1_000_000_000_000_000),
            Some(0),
            Some(vec![dex_tester.dex.asset_b.1.clone()]),
        )?;
        Ok(())
    }

    #[test]
    fn test_queries() -> anyhow::Result<()> {
        let dex_tester = setup_stable_pool()?;
        dex_tester.test_queries()?;
        Ok(())
    }
}

mod stable_pool_tests {
    // "asset_decimals": [
    //     6,
    //     6
    //   ],
    //   "asset_infos": [
    //     {
    //       "native_token": {
    //         "denom": "ibc/C0336ECF2DF64E7D2C98B1422EC2B38DE9EF33C34AAADF18C6F2E3FFC7BE3615"
    //       }
    //     },
    //     {
    //       "native_token": {
    //         "denom": "ibc/43897B9739BD63E3A08A88191999C632E052724AB96BD4C74AE31375C991F48D"
    //       }
    //     }
    //   ],
    //   "cashback": "archway14cdu335ljp6rst337070nejhg7h0j2az7zmx8q0sah88s4uhcczq20fv84",
    //   "contract_addr": "archway102gh7tqaeptt88nckg73mfx8j8du64hw4qqm53zwwykcchwar86sza46ge",
    //   "liquidity_token": "archway1xrmvl87p7mmyntyg6dydmlawjzktled2cvrl8wpeja5qp0xupdvqq0lwuf",
    //   "lockups": "archway14ujr0zy8n5wly4khydsndeuzft0fmt8w9dkchv4rzzn9jx7d3luswkzwsk",
    //   "lp_staking": "archway1aqn5mp6f8gg3fxs5y2dt8k3mk92xms9rq5gpw3rqhgyr6ar9k0vs8ehnfm"

    const ASSET_A: &str = "agoric>istv2";
    const ASSET_B: &str = "axelar>usdcv2";
    const ASSET_A_DENOM: &str =
        "ibc/C0336ECF2DF64E7D2C98B1422EC2B38DE9EF33C34AAADF18C6F2E3FFC7BE3615";
    const ASSET_B_DENOM: &str =
        "ibc/43897B9739BD63E3A08A88191999C632E052724AB96BD4C74AE31375C991F48D";
    const STABLE_POOL_ADDR: &str =
        "archway102gh7tqaeptt88nckg73mfx8j8du64hw4qqm53zwwykcchwar86sza46ge";
    const LIQUIDITY_TOKEN: &str =
        "archway1xrmvl87p7mmyntyg6dydmlawjzktled2cvrl8wpeja5qp0xupdvqq0lwuf";

    use cosmwasm_std::coins;

    use super::*;

    fn setup_stable_pool() -> anyhow::Result<DexTester<CloneTesting, AstrovaultDex>> {
        let chain_info = ARCHWAY_1;
        let abstr_deployment = load_abstr(chain_info)?;
        let chain = abstr_deployment.environment();

        let asset_a = (
            ASSET_A.to_owned(),
            AssetInfoUnchecked::Native(ASSET_A_DENOM.to_owned()),
        );
        let asset_b = (
            ASSET_B.to_owned(),
            AssetInfoUnchecked::Native(ASSET_B_DENOM.to_owned()),
        );
        let pool_addr = Addr::unchecked(STABLE_POOL_ADDR);
        // Normalize stable pool
        {
            let pool_response: astrovault::stable_pool::query_msg::PoolResponse = chain.query(
                &astrovault::stable_pool::query_msg::QueryMsg::Pool {},
                &pool_addr,
            )?;
            match pool_response.assets[0]
                .amount
                .cmp(&pool_response.assets[1].amount)
            {
                std::cmp::Ordering::Less => {
                    let amount = pool_response.assets[1].amount - pool_response.assets[0].amount;
                    let funds = coins(amount.u128(), ASSET_A_DENOM);
                    chain.add_balance(&chain.sender, funds.clone())?;
                    chain.execute(
                        &astrovault::stable_pool::handle_msg::ExecuteMsg::Deposit {
                            assets_amount: vec![amount, 0u128.into()],
                            receiver: None,
                            direct_staking: None,
                        },
                        &funds,
                        &pool_addr,
                    )?;
                }
                std::cmp::Ordering::Greater => {
                    let amount = pool_response.assets[0].amount - pool_response.assets[1].amount;
                    let funds = coins(amount.u128(), ASSET_B_DENOM);
                    chain.add_balance(&chain.sender, funds.clone())?;
                    chain.execute(
                        &astrovault::stable_pool::handle_msg::ExecuteMsg::Deposit {
                            assets_amount: vec![0u128.into(), amount],
                            receiver: None,
                            direct_staking: None,
                        },
                        &funds,
                        &pool_addr,
                    )?;
                }
                // Already normalized, no need to do anything
                std::cmp::Ordering::Equal => (),
            }
        }
        DexTester::new(
            abstr_deployment,
            AstrovaultDex {
                pool_addr,
                liquidity_token: Addr::unchecked(LIQUIDITY_TOKEN),
                pool_type: PoolType::Stable,
                chain,
                asset_a,
                asset_b,
            },
        )
    }

    #[test]
    fn test_swap() -> anyhow::Result<()> {
        let dex_tester = setup_stable_pool()?;
        dex_tester.test_swap()?;
        Ok(())
    }

    // Skipping slippage swap test as it's not applicable to stable pool

    #[test]
    fn test_provide_liquidity() -> anyhow::Result<()> {
        let dex_tester = setup_stable_pool()?;

        dex_tester
            .test_provide_liquidity_two_sided(None, None)
            .unwrap();
        Ok(())
    }

    // Skipping slippage provide_liquidity test as it's not applicable to stable pool

    #[test]
    fn test_withdraw_liquidity() -> anyhow::Result<()> {
        let dex_tester = setup_stable_pool()?;

        dex_tester.test_withdraw_liquidity(None, None, None)?;
        Ok(())
    }

    #[test]
    fn test_queries() -> anyhow::Result<()> {
        let dex_tester = setup_stable_pool()?;
        dex_tester.test_queries()?;
        Ok(())
    }
}

mod ratio_pool_tests {

    // "asset_decimals": [
    //    6,
    //    8
    //  ],
    //  "asset_infos": [
    //    {
    //  "native_token": {
    //    "denom": "ibc/43897B9739BD63E3A08A88191999C632E052724AB96BD4C74AE31375C991F48D"
    //  }
    //    },
    //    {
    //  "native_token": {
    //    "denom": "ibc/3A2DEEBCD51D0B74FE7CE058D40B0BF4C0E556CE9219E8F25F92CF288FF35F56"
    //  }
    //    }
    //  ],
    //  "cashback": "archway14cdu335ljp6rst337070nejhg7h0j2az7zmx8q0sah88s4uhcczq20fv84",
    //  "contract_addr": "archway1alukarfvkx5m2uzazlye7yu0vmyre76rvm63znytjl996thwjtzst5mjx0",
    //  "liquidity_token": "archway1ql5u34l2uglurzyeq59p434uk7dapj9j4skjh6zxjhed6tupwjmqvxvzyx",
    //  "lockups": "archway1nv29h7rw5xe9rmk4erlwfpnp3y5nvvs4jtf003t849f46qnnyerstv2pgm",
    //  "lp_staking": "archway1ncqhffzqsdah8se5w7tpwesw3lh8ryvuvw5pkpprffc97m6lcstsnnke0e"

    const ASSET_A: &str = "usdcnobl";
    const ASSET_B: &str = "wbtcaxl";
    const ASSET_A_DENOM: &str =
        "ibc/43897B9739BD63E3A08A88191999C632E052724AB96BD4C74AE31375C991F48D";
    const ASSET_B_DENOM: &str =
        "ibc/3A2DEEBCD51D0B74FE7CE058D40B0BF4C0E556CE9219E8F25F92CF288FF35F56";
    const RATIO_POOL_ADDR: &str =
        "archway1alukarfvkx5m2uzazlye7yu0vmyre76rvm63znytjl996thwjtzst5mjx0";
    const LIQUIDITY_TOKEN: &str =
        "archway1ql5u34l2uglurzyeq59p434uk7dapj9j4skjh6zxjhed6tupwjmqvxvzyx";

    const PRECISION: Uint128 = Uint128::new(1_000_000);

    use astrovault::utils::{denormalize_amount, normalize_amount};
    use cosmwasm_std::{coins, Uint128};

    use super::*;

    //  Astrovault ratio calculator reference
    //
    // export const hybridStakeCheckUnbalancingThreshold = (pool: IPool, amountsToDeposit: bigint[]) => {
    //   const PRECISION = BigInt(1000000);
    //
    //   // normalize the assets_amount
    //   const assets_amount_normalized = [
    // normalize_amount(amountsToDeposit[0], pool.assetDecimals[0]),
    // normalize_amount(amountsToDeposit[1], pool.assetDecimals[1]),
    //   ];
    //
    //   const pools_amount_normalized = [
    // normalize_amount(pool.poolAssets[0].amount, pool.assetDecimals[0]),
    // normalize_amount(pool.poolAssets[1].amount, pool.assetDecimals[1]),
    //   ];
    //
    //   const pool0_value = BigInt(pools_amount_normalized[0]);
    //   const pool1_value = (BigInt(pools_amount_normalized[1]) * BigInt(pool.hybridRatioDetails.ratio)) / PRECISION;
    //
    //   const pool_total_value = pool0_value + pool1_value;
    //
    //   const asset0_deposit_value = assets_amount_normalized[0];
    //   const asset1_deposit_value = (assets_amount_normalized[1] * BigInt(pool.hybridRatioDetails.ratio)) / PRECISION;
    //
    //   const pool_asset0_after_per =
    // ((pool0_value + asset0_deposit_value) * PRECISION) /
    // (pool_total_value + asset0_deposit_value + asset1_deposit_value);
    //   const pool_asset1_after_per =
    // ((pool1_value + asset1_deposit_value) * PRECISION) /
    // (pool_total_value + asset0_deposit_value + asset1_deposit_value);
    //
    //   if (pool_asset0_after_per > BigInt(pool.settings.max_deposit_unbalancing_threshold)) {
    // if (amountsToDeposit[0] > BigInt(0)) {
    //   return true;
    // }
    //   }
    //   if (pool_asset1_after_per > BigInt(pool.settings.max_deposit_unbalancing_threshold)) {
    // if (amountsToDeposit[1] > BigInt(0)) {
    //   return true;
    // }
    //   }
    //
    //   return false;
    // };

    fn setup_ratio_pool() -> anyhow::Result<DexTester<CloneTesting, AstrovaultDex>> {
        let chain_info = ARCHWAY_1;
        let abstr_deployment = load_abstr(chain_info)?;
        let chain = abstr_deployment.environment();

        let asset_a = (
            ASSET_A.to_owned(),
            AssetInfoUnchecked::Native(ASSET_A_DENOM.to_owned()),
        );
        let asset_b = (
            ASSET_B.to_owned(),
            AssetInfoUnchecked::Native(ASSET_B_DENOM.to_owned()),
        );
        // Normalize ratio pool
        let ratio: Uint128 = chain.query(
            &astrovault::ratio_pool::query_msg::QueryMsg::Ratio {},
            &Addr::unchecked(RATIO_POOL_ADDR),
        )?;
        let pool: astrovault::ratio_pool::query_msg::PoolResponse = chain.query(
            &astrovault::ratio_pool::query_msg::QueryMsg::Pool {},
            &Addr::unchecked(RATIO_POOL_ADDR),
        )?;
        let pool0_value = Uint128::new(normalize_amount(&pool.assets[0].amount, &6).unwrap());
        let pool1_value =
            Uint128::new(normalize_amount(&pool.assets[1].amount, &8).unwrap()) * ratio / PRECISION;
        match pool0_value.cmp(&pool1_value) {
            std::cmp::Ordering::Less => {
                let amount =
                    denormalize_amount(&(pool1_value - pool0_value).u128().into(), &6).unwrap();
                let funds = coins(amount, ASSET_A_DENOM);
                chain.add_balance(&chain.sender, funds.clone())?;
                chain.execute(
                    &astrovault::ratio_pool::handle_msg::ExecuteMsg::Deposit {
                        assets_amount: [amount.into(), 0u128.into()],
                        receiver: None,
                        direct_staking: None,
                        expected_return: None,
                    },
                    &funds,
                    &Addr::unchecked(RATIO_POOL_ADDR),
                )?;
            }
            std::cmp::Ordering::Greater => {
                let amount = denormalize_amount(
                    &((pool0_value - pool1_value) / ratio * PRECISION)
                        .u128()
                        .into(),
                    &8,
                )
                .unwrap();
                let funds = coins(amount, ASSET_B_DENOM);
                chain.add_balance(&chain.sender, funds.clone())?;
                chain.execute(
                    &astrovault::ratio_pool::handle_msg::ExecuteMsg::Deposit {
                        assets_amount: [0_u128.into(), amount.into()],
                        receiver: None,
                        direct_staking: None,
                        expected_return: None,
                    },
                    &funds,
                    &Addr::unchecked(RATIO_POOL_ADDR),
                )?;
            }
            // Already normalized, no need to do anything
            std::cmp::Ordering::Equal => (),
        }
        DexTester::new(
            abstr_deployment,
            AstrovaultDex {
                pool_addr: Addr::unchecked(RATIO_POOL_ADDR),
                liquidity_token: Addr::unchecked(LIQUIDITY_TOKEN),
                pool_type: PoolType::Weighted,
                chain,
                asset_a,
                asset_b,
            },
        )
    }

    #[test]
    fn test_swap() -> anyhow::Result<()> {
        let dex_tester = setup_ratio_pool()?;
        dex_tester.test_swap()?;
        Ok(())
    }

    // Skipping slippage swap test as it's not applicable to ratio pool

    #[test]
    fn test_provide_liquidity() -> anyhow::Result<()> {
        let dex_tester = setup_ratio_pool()?;

        let deposit_amount = 1_000_000;

        // Pool is normalized right now so should be fine to provide "non-equal" amounts

        dex_tester.test_provide_liquidity_two_sided(Some(deposit_amount), Some(deposit_amount))?;
        Ok(())
    }

    // Skipping slippage provide_liquidity test as it's not applicable to ratio pool

    #[test]
    fn test_withdraw_liquidity() -> anyhow::Result<()> {
        let dex_tester = setup_ratio_pool()?;

        let deposit_amount = 1_000_000;

        dex_tester.test_withdraw_liquidity(
            Some(deposit_amount),
            Some(deposit_amount),
            Some(vec![dex_tester.dex.asset_b.1.clone()]),
        )?;
        Ok(())
    }

    #[test]
    fn test_queries() -> anyhow::Result<()> {
        let dex_tester = setup_ratio_pool()?;
        dex_tester.test_queries()?;
        Ok(())
    }
}
