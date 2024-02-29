use abstract_app::objects::{
    pool_id::PoolAddressBase, AssetEntry, LpToken, PoolMetadata, PoolType, UncheckedContractEntry,
};
use abstract_client::{AbstractClient, Environment};
use abstract_cw_staking::staking_tester::{MockStaking, StakingTester};
use abstract_interface::ExecuteMsgFns;
use cosmwasm_std::{coin, Addr};
use cw_asset::AssetInfoUnchecked;
use cw_orch::daemon::networks::NEUTRON_1;
use cw_orch::prelude::*;
use cw_orch_clone_testing::CloneTesting;

use crate::clone_testing::dex::astroport::{
    FACTORY_ADDR, GENERATOR_ADDR,
};

use super::load_abstr;

// mainnet addr of abstract
const SENDER: &str = "neutron1kjzpqv393k4g064xh04j4hwy5d0s03wf7dnt4x";

const ASSET_A: &str = "test-asset-one";
const ASSET_B: &str = "test-asset-two";
const ASSET_AMOUNT: u128 = 1_000_000_000_000;

pub struct AstroportStake {
    chain: CloneTesting,
    lp_asset: (String, AssetInfoUnchecked),
    minter: Addr,
}

impl AstroportStake {
    fn name() -> String {
        "astroport".to_owned()
    }

    fn new(chain: CloneTesting) -> anyhow::Result<Self> {
        let (ans_asset_a, asset_a) = (
            "tao".to_owned(),
            AssetInfoUnchecked::Native(ASSET_A.to_owned()),
        );
        let (ans_asset_b, asset_b) = (
            "tat".to_owned(),
            AssetInfoUnchecked::Native(ASSET_B.to_owned()),
        );
        let asset_a_astroport = astroport::asset::AssetInfo::native(ASSET_A);
        let asset_b_astroport = astroport::asset::AssetInfo::native(ASSET_B);

        // Create pool
        let asset_infos = vec![asset_a_astroport.clone(), asset_b_astroport.clone()];
        let resp = chain.execute(
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

        let pair_addr = Addr::unchecked(pair_contract_addr);

        // Add some liquidity
        let assets = vec![
            astroport::asset::Asset::new(asset_a_astroport, ASSET_AMOUNT),
            astroport::asset::Asset::new(asset_b_astroport, ASSET_AMOUNT),
        ];
        let amount = vec![coin(ASSET_AMOUNT, ASSET_A), coin(ASSET_AMOUNT, ASSET_B)];
        chain.add_balance(&chain.sender, amount.clone())?;
        chain.execute(
            &astroport::pair::ExecuteMsg::ProvideLiquidity {
                assets,
                slippage_tolerance: None,
                auto_stake: None,
                receiver: None,
            },
            &amount,
            &pair_addr,
        )?;

        let pool = PoolAddressBase::Contract(pair_addr.to_string());
        let pool_metadata = PoolMetadata {
            dex: Self::name(),
            pool_type: PoolType::ConstantProduct,
            assets: vec![AssetEntry::new(&ans_asset_a), AssetEntry::new(&ans_asset_b)],
        };
        let lp_asset = AssetInfoUnchecked::Cw20(liquidity_token_addr);

        // Register everything on ans
        let abstr_deployment = AbstractClient::new(chain.clone())?;
        abstr_deployment.name_service().update_contract_addresses(
            vec![(
                UncheckedContractEntry {
                    protocol: Self::name(),
                    contract: format!(
                        "staking/{dex}/{asset_a},{asset_b}",
                        dex = Self::name(),
                        asset_a = &ans_asset_a,
                        asset_b = &ans_asset_b,
                    ),
                },
                GENERATOR_ADDR.to_owned(),
            )],
            vec![],
        )?;
        // Add assets
        abstr_deployment.name_service().update_asset_addresses(
            vec![
                (ans_asset_a.clone(), asset_a),
                (ans_asset_b.clone(), asset_b),
            ],
            vec![],
        )?;
        // Add dex
        abstr_deployment
            .name_service()
            .update_dexes(vec![Self::name()], vec![])?;
        // Add pool
        abstr_deployment
            .name_service()
            .update_pools(vec![(pool, pool_metadata)], vec![])?;
        // Add lp asset
        let lp_token = LpToken::new(Self::name(), vec![ans_asset_a, ans_asset_b]);
        abstr_deployment
            .name_service()
            .update_asset_addresses(vec![(lp_token.to_string(), lp_asset.clone())], vec![])?;

        Ok(Self {
            chain,
            lp_asset: (lp_token.to_string(), lp_asset),
            minter: pair_addr,
        })
    }
}

impl MockStaking for AstroportStake {
    fn name(&self) -> String {
        Self::name()
    }

    fn stake_token(&self) -> (String, AssetInfoUnchecked) {
        self.lp_asset.clone()
    }

    fn mint_lp(&self, addr: &Addr, amount: u128) -> anyhow::Result<()> {
        let chain = &self.chain;

        let AssetInfoUnchecked::Cw20(contract_addr) = &self.lp_asset.1 else {
            unreachable!();
        };
        chain.call_as(&self.minter).execute(
            &cw20::Cw20ExecuteMsg::Mint {
                recipient: addr.to_string(),
                amount: amount.into(),
            },
            &[],
            &Addr::unchecked(contract_addr),
        )?;
        Ok(())
    }
}

fn setup() -> anyhow::Result<StakingTester<CloneTesting, AstroportStake>> {
    let chain_info = NEUTRON_1;
    let sender = Addr::unchecked(SENDER);
    let abstr_deployment = load_abstr(chain_info, sender)?;
    let chain = abstr_deployment.environment();
    StakingTester::new(abstr_deployment, AstroportStake::new(chain)?)
}

#[test]
fn test_stake() -> anyhow::Result<()> {
    let stake_tester = setup()?;
    stake_tester.test_stake()?;
    Ok(())
}
