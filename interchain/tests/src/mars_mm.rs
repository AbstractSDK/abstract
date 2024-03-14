use abstract_app::objects::UncheckedContractEntry;
use abstract_client::Environment;
use abstract_moneymarket_adapter::tester::{MockMoneyMarket, MoneyMarketTester};
use cosmwasm_std::Addr;
use cw_orch::daemon::networks::OSMOSIS_1;
use cw_orch_clone_testing::CloneTesting;

use crate::common::load_abstr;

pub struct MarsMoneymarket {
    pub chain: CloneTesting,
    pub lending_asset: (String, String),
    pub collateral_asset: (String, String),
}

pub const OSMOSIS_REDBANK_ADDRESS: &str =
    "osmo1c3ljch9dfw5kf52nfwpxd2zmj2ese7agnx0p9tenkrryasrle5sqf3ftpg";

pub const OSMOSIS_ORACLE_ADDRESS: &str =
    "osmo1mhznfr60vjdp2gejhyv2gax9nvyyzhd3z0qcwseyetkfustjauzqycsy2g";

// Abstract admin address
pub const SENDER: &str = "osmo1t07t5ejcwtlclnelvtsdf3rx30kxvczlng8p24";

impl MockMoneyMarket for MarsMoneymarket {
    fn name(&self) -> String {
        "mars".to_owned()
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
        // ANNNNND that's it

        vec![
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    contract: "red-bank".to_string(),
                },
                OSMOSIS_REDBANK_ADDRESS.to_string(),
            ),
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    contract: "oracle".to_string(),
                },
                OSMOSIS_ORACLE_ADDRESS.to_string(),
            ),
        ]
    }
}

fn setup() -> anyhow::Result<MoneyMarketTester<CloneTesting, MarsMoneymarket>> {
    let chain_info = OSMOSIS_1;
    let sender = Addr::unchecked(SENDER);
    let abstr_deployment = load_abstr(chain_info, sender)?;
    let chain = abstr_deployment.environment();
    let collateral_asset = ("osmo".to_owned(), "uosmo".to_owned());
    let lending_asset = (
        "usdc".to_owned(),
        "ibc/D189335C6E4A68B513C10AB227BF1C1D38C746766278BA3EEB4FB14124F1D858".to_owned(),
    );
    MoneyMarketTester::new(
        abstr_deployment,
        MarsMoneymarket {
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
    mm_tester.test_withdraw()?;
    Ok(())
}
