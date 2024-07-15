use abstract_app::objects::{
    pool_id::PoolAddressBase, AssetEntry, LpToken, PoolMetadata, PoolType, UncheckedContractEntry,
};
use abstract_client::{AbstractClient, Environment};
use abstract_cw_staking::staking_tester::{MockStaking, StakingTester};
use abstract_interface::ExecuteMsgFns;
use abstract_modules_interchain_tests::common::load_abstr;
use cw_asset::AssetInfoUnchecked;
use cw_orch::daemon::networks::ARCHWAY_1;
use cw_orch::prelude::*;
use cw_orch_clone_testing::CloneTesting;
use serde::{Deserialize, Serialize};

// Astrovault uses custom types for creating pools: https://github.com/archway-network/archway/blob/c2f92ce09f7a2e91046ba494546d157ad7f99ded/contracts/go/voter/src/pkg/archway/custom/msg.go
// Meaning we have to use existing pools

// aarch - xaarch pool
const LP_STAKING_ADDR: &str = "archway13xeat9u6s0x7vphups0r096fl3tkr3zenhdvfjrsc2t0t70ayugscdw46g";
const LP_ASSET_ADDR: &str = "archway123h0jfnk3rhhuapkytrzw22u6w4xkf563lqhy42a9r5lmv32w73s8f6ql2";
const POOL_ADDR: &str = "archway1vq9jza8kuz80f7ypyvm3pttvpcwlsa5fvum9hxhew5u95mffknxsjy297r";

// mainnet addr of abstract
const SENDER: &str = "archway1kjzpqv393k4g064xh04j4hwy5d0s03wf0exd9k";

const ASSET_A: &str = "archway>archv2";
const ASSET_B: &str = "archway>xarchv2";
const ASSET_A_DENOM: &str = "aarch";
const ASSET_B_ADDR: &str = "archway1cutfh7m87cyq5qgqqw49f289qha7vhsg6wtr6rl5fvm28ulnl9ssg0vk0n";

pub struct AstrovaultStake {
    chain: CloneTesting,
    lp_asset: (String, AssetInfoUnchecked),
    minter: Addr,
    rewards_source: RewardSourceResponse,
}

impl AstrovaultStake {
    fn name() -> String {
        "astrovault".to_owned()
    }

    fn new(chain: CloneTesting) -> anyhow::Result<Self> {
        let (ans_asset_a, asset_a) = (
            ASSET_A.to_owned(),
            AssetInfoUnchecked::Native(ASSET_A_DENOM.to_owned()),
        );
        let (ans_asset_b, asset_b) = (
            ASSET_B.to_owned(),
            AssetInfoUnchecked::Cw20(ASSET_B_ADDR.to_owned()),
        );

        let pool = PoolAddressBase::Contract(POOL_ADDR.to_owned());
        let pool_metadata = PoolMetadata {
            dex: Self::name(),
            pool_type: PoolType::ConstantProduct,
            assets: vec![AssetEntry::new(&ans_asset_a), AssetEntry::new(&ans_asset_b)],
        };
        let lp_asset = AssetInfoUnchecked::Cw20(LP_ASSET_ADDR.to_owned());

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
                LP_STAKING_ADDR.to_owned(),
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

        let rewards_sources: Vec<RewardSourceResponse> = chain.query(
            &astrovault::lp_staking::query_msg::QueryMsg::RewardSources {
                reward_source: None,
            },
            &Addr::unchecked(LP_STAKING_ADDR),
        )?;
        Ok(Self {
            chain,
            lp_asset: (lp_token.to_string(), lp_asset),
            minter: Addr::unchecked(POOL_ADDR),
            rewards_source: rewards_sources[0].clone(),
        })
    }
}

// astrovault have broken response types here
#[derive(Serialize, Deserialize, Clone)]
struct RewardSourceResponse {
    pub address: String,
    pub info: RewardSourceInfo,
}

#[derive(Serialize, Deserialize, Clone)]
struct RewardSourceInfo {
    pub reward_asset: astrovault::assets::asset::AssetInfo,
}

impl MockStaking for AstrovaultStake {
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

    fn generate_rewards(&self, addr: &Addr, amount: u128) -> anyhow::Result<()> {
        let chain = &self.chain;

        let mint_distributor_addr = &self.rewards_source.address;

        let config: astrovault::mint_distributor::query_msg::ConfigResponse =
            chain.wasm_querier().smart_query(
                mint_distributor_addr,
                &astrovault::mint_distributor::query_msg::QueryMsg::Config {},
            )?;

        chain.call_as(&Addr::unchecked(config.owner)).execute(
            &astrovault::mint_distributor::handle_msg::ExecuteMsg::UpdateExternalMintAllowlist {
                allowlist: Some(vec![chain.sender_addr().to_string()]),
            },
            &[],
            &Addr::unchecked(mint_distributor_addr),
        )?;

        chain.execute(
            &astrovault::mint_distributor::handle_msg::ExecuteMsg::ExternalMint {
                recipient: addr.to_string(),
                amount: amount.into(),
            },
            &[],
            &Addr::unchecked(mint_distributor_addr),
        )?;

        Ok(())
    }

    fn reward_asset(&self) -> AssetInfoUnchecked {
        match self.rewards_source.info.reward_asset.clone() {
            astrovault::assets::asset::AssetInfo::Token { contract_addr } => {
                AssetInfoUnchecked::Cw20(contract_addr)
            }
            astrovault::assets::asset::AssetInfo::NativeToken { denom } => {
                AssetInfoUnchecked::Native(denom)
            }
        }
    }

    fn staking_target(&self) -> abstract_cw_staking::msg::StakingTarget {
        abstract_cw_staking::msg::StakingTarget::Contract(Addr::unchecked(LP_STAKING_ADDR))
    }
}

fn setup() -> anyhow::Result<StakingTester<CloneTesting, AstrovaultStake>> {
    let chain_info = ARCHWAY_1;
    let sender = Addr::unchecked(SENDER);
    let abstr_deployment = load_abstr(chain_info, sender)?;
    let chain = abstr_deployment.environment();
    StakingTester::new(abstr_deployment, AstrovaultStake::new(chain)?)
}

#[test]
fn test_stake() -> anyhow::Result<()> {
    let stake_tester = setup()?;

    stake_tester.test_stake()?;
    Ok(())
}

#[test]
fn test_unstake() -> anyhow::Result<()> {
    let stake_tester = setup()?;

    stake_tester.test_unstake()?;
    Ok(())
}

#[test]
fn test_claim() -> anyhow::Result<()> {
    let stake_tester = setup()?;

    stake_tester.test_claim()?;
    Ok(())
}

#[test]
fn test_queries() -> anyhow::Result<()> {
    let stake_tester = setup()?;

    stake_tester.test_staking_info()?;
    stake_tester.test_query_rewards()?;
    Ok(())
}
