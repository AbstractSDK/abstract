use abstract_app::objects::UncheckedContractEntry;
use abstract_client::Environment;
use abstract_modules_interchain_tests::common::load_abstr;
use abstract_money_market_adapter::tester::{MockMoneyMarket, MoneyMarketTester};
use cosmwasm_std::Addr;
use cw_orch_clone_testing::CloneTesting;

pub struct MarsMoneymarket {
    pub chain: CloneTesting,
    pub lending_asset: (String, String),
    pub collateral_asset: (String, String),
    pub redbank_address: String,
    pub oracle_address: String,
}

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
        vec![
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    contract: "red-bank".to_string(),
                },
                self.redbank_address.to_string(),
            ),
            (
                UncheckedContractEntry {
                    protocol: self.name(),
                    contract: "oracle".to_string(),
                },
                self.oracle_address.to_string(),
            ),
        ]
    }
}

mod osmosis_tests {
    use super::*;

    use cw_orch::daemon::networks::OSMOSIS_1;

    pub const OSMOSIS_REDBANK_ADDRESS: &str =
        "osmo1c3ljch9dfw5kf52nfwpxd2zmj2ese7agnx0p9tenkrryasrle5sqf3ftpg";

    pub const OSMOSIS_ORACLE_ADDRESS: &str =
        "osmo1mhznfr60vjdp2gejhyv2gax9nvyyzhd3z0qcwseyetkfustjauzqycsy2g";

    // Abstract admin address
    pub const SENDER: &str = "osmo1t07t5ejcwtlclnelvtsdf3rx30kxvczlng8p24";

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
                redbank_address: OSMOSIS_REDBANK_ADDRESS.to_owned(),
                oracle_address: OSMOSIS_ORACLE_ADDRESS.to_owned(),
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
        mm_tester.test_borrow()?;
        Ok(())
    }

    #[test]
    fn repay() -> anyhow::Result<()> {
        let mm_tester = setup()?;
        mm_tester.test_repay()?;
        Ok(())
    }

    // Queries
    #[test]
    fn price() -> anyhow::Result<()> {
        let mm_tester = setup()?;
        mm_tester.test_price()?;
        Ok(())
    }

    #[test]
    fn user_ltv() -> anyhow::Result<()> {
        let mm_tester = setup()?;
        mm_tester.test_user_ltv()?;
        Ok(())
    }

    #[test]
    fn max_ltv() -> anyhow::Result<()> {
        let mm_tester = setup()?;
        mm_tester.test_max_ltv()?;
        Ok(())
    }
}
mod neutron_tests {
    use super::*;

    use cw_orch::daemon::networks::NEUTRON_1;

    pub const NEUTRON_REDBANK_ADDRESS: &str =
        "neutron1n97wnm7q6d2hrcna3rqlnyqw2we6k0l8uqvmyqq6gsml92epdu7quugyph";

    pub const NEUTRON_ORACLE_ADDRESS: &str =
        "neutron1dwp6m7pdrz6rnhdyrx5ha0acsduydqcpzkylvfgspsz60pj2agxqaqrr7g";

    // Abstract admin address
    pub const SENDER: &str = "neutron1kjzpqv393k4g064xh04j4hwy5d0s03wf7dnt4x";

    fn setup() -> anyhow::Result<MoneyMarketTester<CloneTesting, MarsMoneymarket>> {
        let chain_info = NEUTRON_1;
        let sender = Addr::unchecked(SENDER);
        let abstr_deployment = load_abstr(chain_info, sender)?;
        let chain = abstr_deployment.environment();
        let collateral_asset = ("ntrn".to_owned(), "untrn".to_owned());
        let lending_asset = (
            "usdc".to_owned(),
            "ibc/F082B65C88E4B6D5EF1DB243CDA1D331D002759E938A0F5CD3FFDC5D53B3E349".to_owned(),
        );
        MoneyMarketTester::new(
            abstr_deployment,
            MarsMoneymarket {
                chain,
                lending_asset,
                collateral_asset,
                redbank_address: NEUTRON_REDBANK_ADDRESS.to_owned(),
                oracle_address: NEUTRON_ORACLE_ADDRESS.to_owned(),
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
        mm_tester.test_borrow()?;
        Ok(())
    }

    #[test]
    fn repay() -> anyhow::Result<()> {
        let mm_tester = setup()?;
        mm_tester.test_repay()?;
        Ok(())
    }

    // Queries
    #[test]
    fn price() -> anyhow::Result<()> {
        let mm_tester = setup()?;
        mm_tester.test_price()?;
        Ok(())
    }

    #[test]
    fn user_ltv() -> anyhow::Result<()> {
        let mm_tester = setup()?;
        mm_tester.test_user_ltv()?;
        Ok(())
    }

    #[test]
    fn max_ltv() -> anyhow::Result<()> {
        let mm_tester = setup()?;
        mm_tester.test_max_ltv()?;
        Ok(())
    }
}
