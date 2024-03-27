use abstract_app::objects::UncheckedContractEntry;
use abstract_client::Environment;
use abstract_modules_interchain_tests::common::load_abstr;
use abstract_money_market_adapter::tester::{MockMoneyMarket, MoneyMarketTester, DEPOSIT_VALUE};
use cosmwasm_std::Addr;
use cw_orch::daemon::networks::PHOENIX_1;
use cw_orch_clone_testing::CloneTesting;
pub struct CavernMoneyMarket {
    pub chain: CloneTesting,
    pub lending_asset: (String, String),
    /// Ans asset, actual denom, cavern specific wrapper token
    pub collateral_asset: (String, String, String),
}

pub const TERRA_MARKET_ADDRESS: &str =
    "terra1zqlcp3aty4p4rjv96h6qdascdn953v6crhwedu5vddxjnp349upscluex6";

pub const TERRA_ORACLE_ADDRESS: &str =
    "terra1gp3a4cz9magxuvj6n0x8ra8jqc79zqvquw85xrn0suwvml2cqs4q4l7ss7";

pub const TERRA_OVERSEER_ADDRESS: &str =
    "terra1l6rq7905263uqmayurtulzc09sfcgxdedsfen7m0y6wf28s49tvqdkwau9";

// Abstract admin address
pub const SENDER: &str = "terra1uycc6xnufjv9s54apy6mlz77q24ln94qrh8z50";

impl MockMoneyMarket for CavernMoneyMarket {
    fn name(&self) -> String {
        "cavern".to_owned()
    }

    fn lending_asset(&self) -> (String, cw_asset::AssetInfoUnchecked) {
        let (asset_entry, denom) = &self.lending_asset;
        (
            asset_entry.to_owned(),
            cw_asset::AssetInfoUnchecked::native(denom),
        )
    }

    fn collateral_asset(&self) -> (String, cw_asset::AssetInfoUnchecked) {
        let (asset_entry, denom, _) = &self.collateral_asset;
        (
            asset_entry.to_owned(),
            cw_asset::AssetInfoUnchecked::native(denom),
        )
    }

    fn setup(&self) -> Vec<(UncheckedContractEntry, String)> {
        // We need to register the red bank and the oracle inside abstract
        // ANNNNND that's it

        vec![
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    contract: "market".to_string(),
                },
                TERRA_MARKET_ADDRESS.to_string(),
            ),
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    contract: "oracle".to_string(),
                },
                TERRA_ORACLE_ADDRESS.to_string(),
            ),
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    contract: "overseer".to_string(),
                },
                TERRA_OVERSEER_ADDRESS.to_string(),
            ),
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    contract: "custody/bWhale".to_string(),
                },
                "terra1vmr33lncm0jhkm9gfj8824ahk50asysjgzt3ex7e94clecss8nzqftzzv2".to_string(),
            ),
        ]
    }
}

fn setup() -> anyhow::Result<MoneyMarketTester<CloneTesting, CavernMoneyMarket>> {
    let chain_info = PHOENIX_1;
    let sender = Addr::unchecked(SENDER);
    let abstr_deployment = load_abstr(chain_info, sender)?;
    let chain = abstr_deployment.environment();
    let lending_asset = (
        "usdc".to_owned(),
        "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_owned(),
    );
    let collateral_asset = (
        "bWhale".to_owned(),
        "ibc/517E13F14A1245D4DE8CF467ADD4DA0058974CDCC880FA6AE536DBCA1D16D84E".to_owned(),
        "terra1ze3c86la6wynenrqewhq4j9hw24yrvardudsl5mkq3mhgs6ag4cqrva0pg".to_owned(),
    );
    MoneyMarketTester::new(
        abstr_deployment,
        CavernMoneyMarket {
            chain,
            lending_asset,
            collateral_asset,
        },
    )
}

#[test]
fn deposit() -> anyhow::Result<()> {
    let mm_tester = setup()?;
    mm_tester.test_deposit()?;
    Ok(())
}

#[test]
fn withdraw() -> anyhow::Result<()> {
    let mm_tester = setup()?;

    // We execute that to make sure there is enough deposited inside cavern when doing the deposit-withdraw cycle
    mm_tester.test_deposit()?;
    mm_tester.test_withdraw()?;
    Ok(())
}

#[test]
fn provide_collateral() -> anyhow::Result<()> {
    let mm_tester = setup()?;
    mm_tester.test_provide_collateral()?;
    Ok(())
}
#[test]
fn withdraw_collateral() -> anyhow::Result<()> {
    let mm_tester = setup()?;
    mm_tester.test_withdraw_collateral()?;
    Ok(())
}
#[test]
fn borrow() -> anyhow::Result<()> {
    let mm_tester = setup()?;
    mm_tester.deposit(DEPOSIT_VALUE * 100)?;
    mm_tester.test_borrow()?;
    Ok(())
}
#[test]
fn repay() -> anyhow::Result<()> {
    let mm_tester = setup()?;
    mm_tester.deposit(DEPOSIT_VALUE * 100)?;
    mm_tester.test_repay()?;
    Ok(())
}

#[test]
fn user_ltv() -> anyhow::Result<()> {
    let mm_tester = setup()?;
    mm_tester.deposit(DEPOSIT_VALUE * 100)?;
    mm_tester.test_user_ltv()?;
    Ok(())
}
