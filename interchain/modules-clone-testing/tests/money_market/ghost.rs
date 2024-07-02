use abstract_app::objects::UncheckedContractEntry;
use abstract_client::Environment;
use abstract_modules_interchain_tests::common::load_abstr;
use abstract_money_market_adapter::tester::{MockMoneyMarket, MoneyMarketTester};
use cosmwasm_std::Addr;
use cw_orch::daemon::networks::HARPOON_4;
use cw_orch_clone_testing::CloneTesting;
pub struct GhostMoneymarket {
    pub chain: CloneTesting,
    pub lending_asset: (String, String),
    pub collateral_asset: (String, String),
}

pub const KUJIRA_MARKET_ADDRESS: &str =
    "kujira1t88e60depax2zz43mt5ggxdzqe225guhn0jdyl3ccq5x30lcu2hqcnzt2t";

pub const KUJIRA_KUJI_VAULT_ADDRESS: &str =
    "kujira18txj8dep8n9cfgmhgshut05d9t4vjdphcd3dwl32vu4898w9uxnslaflur";

pub const KUJIRA_USK_VAULT_ADDRESS: &str =
    "kujira19r7998wj50nss7r4f6dpqwmyetevw0udl5udlrt8jtswa65nplvsflv7we";
// Abstract admin address
pub const SENDER: &str = "kujira14cl2dthqamgucg9sfvv4relp3aa83e40yjx3f5";

impl MockMoneyMarket for GhostMoneymarket {
    fn name(&self) -> String {
        "ghost".to_owned()
    }

    fn lending_asset(&self) -> (String, cw_asset::AssetInfoUnchecked) {
        let (asset_entry, denom) = &self.lending_asset;
        (
            asset_entry.to_owned(),
            cw_asset::AssetInfoUnchecked::native(denom),
        )
    }

    fn collateral_asset(&self) -> (String, cw_asset::AssetInfoUnchecked) {
        let (asset_entry, denom) = &self.collateral_asset;
        (
            asset_entry.to_owned(),
            cw_asset::AssetInfoUnchecked::native(denom),
        )
    }

    fn setup(&self) -> Vec<(UncheckedContractEntry, String)> {
        // We need to register the red bank and the oracle inside abstract
        vec![
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    contract: "market/usk/kuji".to_string(),
                },
                KUJIRA_MARKET_ADDRESS.to_string(),
            ),
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    // contract: "market/kujira>kuji/kujira>usk".to_string(),
                    contract: "market/kuji/usk".to_string(),
                },
                KUJIRA_MARKET_ADDRESS.to_string(),
            ),
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    contract: "vault/usk".to_string(),
                },
                KUJIRA_USK_VAULT_ADDRESS.to_string(),
            ),
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    // contract: "vault/kujira>kuji".to_string(),
                    contract: "vault/kuji".to_string(),
                },
                KUJIRA_KUJI_VAULT_ADDRESS.to_string(),
            ),
        ]
    }
}

fn setup() -> anyhow::Result<MoneyMarketTester<CloneTesting, GhostMoneymarket>> {
    let chain_info = HARPOON_4;
    let sender = Addr::unchecked(SENDER);
    let abstr_deployment = load_abstr(chain_info, sender)?;
    let chain = abstr_deployment.environment();
    let collateral_asset = ("kuji".to_owned(), "ukuji".to_owned());
    let lending_asset = (
        "usk".to_owned(),
        "factory/kujira1r85reqy6h0lu02vyz0hnzhv5whsns55gdt4w0d7ft87utzk7u0wqr4ssll/uusk".to_owned(),
    );
    MoneyMarketTester::new(
        abstr_deployment,
        GhostMoneymarket {
            chain,
            lending_asset,
            collateral_asset,
        },
    )
}

// Kujira deposit uses custom module for deposit

// #[test]
// fn deposit() -> anyhow::Result<()> {
//     let mm_tester = setup()?;
//     mm_tester.test_deposit()?;
//     Ok(())
// }

// #[test]
// fn withdraw() -> anyhow::Result<()> {
//     let mm_tester = setup()?;
//     mm_tester.test_withdraw()?;
//     Ok(())
// }

// Collateral provision uses deposit whish uses custom module

// #[test]
// fn provide_collateral() -> anyhow::Result<()> {
//     let mm_tester = setup()?;
//     mm_tester.test_provide_collateral()?;
//     Ok(())
// }

// #[test]
// fn withdraw_collateral() -> anyhow::Result<()> {
//     let mm_tester = setup()?;
//     mm_tester.test_withdraw_collateral()?;
//     Ok(())
// }

// Borrow and repay needs deposit to borrow, which uses custom module

// #[test]
// fn borrow() -> anyhow::Result<()> {
//     let mm_tester = setup()?;
//     mm_tester.test_borrow()?;
//     Ok(())
// }

// #[test]
// fn repay() -> anyhow::Result<()> {
//     let mm_tester = setup()?;
//     mm_tester.test_repay()?;
//     Ok(())
// }

// Queries

// Price uses stargate query

// #[test]
// fn price() -> anyhow::Result<()> {
//     let mm_tester = setup()?;
//     mm_tester.test_price()?;
//     Ok(())
// }

// ltv uses price, which uses stargate query

// #[test]
// fn user_ltv() -> anyhow::Result<()> {
//     let mm_tester = setup()?;
//     mm_tester.test_user_ltv()?;
//     Ok(())
// }

#[test]
fn max_ltv() -> anyhow::Result<()> {
    let mm_tester = setup()?;
    mm_tester.test_max_ltv()?;
    Ok(())
}
