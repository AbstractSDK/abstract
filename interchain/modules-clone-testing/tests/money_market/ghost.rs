use abstract_app::objects::UncheckedContractEntry;
use abstract_client::Environment;
use abstract_modules_interchain_tests::common::load_abstr;
use abstract_money_market_adapter::tester::{MockMoneyMarket, MoneyMarketTester, DEPOSIT_VALUE};
use cosmwasm_std::Addr;
use cw_orch::daemon::networks::HARPOON_4;
use cw_orch_clone_testing::CloneTesting;
pub struct KujiraMoneyMarket {
    pub chain: CloneTesting,
    pub lending_asset: (String, String),
    /// Ans asset, actual denom
    pub collateral_asset: (String, String),
}

/*
borrow
{
        "pair": "KUJI/USK",
        "vault_address": "kujira1yqh4gfa75jh2q82e9ada98l9qz7xf0xvwa399cl52a4vrv3kxzvstrjuy0",
        "performance": "0.0%",
        "profit_in_usdc": "$0.00",
        "liquidity": "$456",
        "base_denom": "ukuji",
        "quote_denom": "factory/kujira1qk00h5atutpsv900x202pxx42npjr9thg58dnqpa72f2p7m2luase444a7/uusk",
        "fin_address": "kujira193dzcmy7lwuj4eda3zpwwt9ejal00xva0vawcvhgsyyp5cfh6jyq66wfrf",
        "btoken_value": "$2.95"
    }
 */

pub const HARPOON_VAULT_ADDRESS: &str =
    "kujira1yqh4gfa75jh2q82e9ada98l9qz7xf0xvwa399cl52a4vrv3kxzvstrjuy0";

pub const HARPOON_MARKET_ADDRESS: &str =
    "kujira193dzcmy7lwuj4eda3zpwwt9ejal00xva0vawcvhgsyyp5cfh6jyq66wfrf";

// Abstract admin address
pub const SENDER: &str = "kujira14cl2dthqamgucg9sfvv4relp3aa83e40yjx3f5";

impl MockMoneyMarket for KujiraMoneyMarket {
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

        vec![
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    contract: "vault/kujira>kuji".to_string(),
                },
                HARPOON_VAULT_ADDRESS.to_string(),
            ),
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    contract: "market/kujira>kuji/kujira>usk".to_string(),
                },
                HARPOON_MARKET_ADDRESS.to_string(),
            ),
        ]
    }
}

fn setup() -> cw_orch::anyhow::Result<MoneyMarketTester<CloneTesting, KujiraMoneyMarket>> {
    let chain_info = HARPOON_4;
    let sender = Addr::unchecked(SENDER);
    let abstr_deployment = load_abstr(chain_info, sender)?;
    let chain = abstr_deployment.environment();
    let lending_asset = (
        "kujira>kuji".to_owned(),
        "ukuji".to_owned(),
    );
    let collateral_asset = (
        "kujira>usk".to_owned(),
        "factory/kujira1qk00h5atutpsv900x202pxx42npjr9thg58dnqpa72f2p7m2luase444a7/uusk".to_owned(),
    );
    MoneyMarketTester::new(
        abstr_deployment,
        KujiraMoneyMarket {
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
