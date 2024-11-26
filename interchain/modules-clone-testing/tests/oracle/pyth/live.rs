use std::str::FromStr;

use super::pyth_api::PythApiResponse;
use abstract_app::{objects::ContractEntry, std::ans_host::QueryMsgFns};
use abstract_client::AbstractClient;
use abstract_oracle_adapter::{
    oracle_tester::{MockOracle, OracleTester},
    oracles::PYTH,
};
use cosmwasm_std::{Binary, Uint128};
use cw_orch::daemon::networks::XION_TESTNET_1;
use cw_orch::prelude::*;
use cw_orch_clone_testing::CloneTesting;
use networks::{NEUTRON_1, OSMOSIS_1, OSMO_5, PION_1};

pub use super::{ORACLE_PRICE_API, PRICE_SOURCE_KEY};

pub struct PythOracleTester {
    pub current_oracle_price_data: PythApiResponse,
}

impl MockOracle<CloneTesting> for PythOracleTester {
    const MAX_AGE: u64 = 60;
    fn price_source_key(&self) -> String {
        PRICE_SOURCE_KEY.to_string()
    }

    fn name(&self) -> String {
        PYTH.to_string()
    }

    fn ans_setup(&self, _abstr_deployment: &AbstractClient<CloneTesting>) -> anyhow::Result<()> {
        Ok(())
    }
}

fn setup_clone_testing(
    chain: ChainInfo,
) -> anyhow::Result<OracleTester<CloneTesting, PythOracleTester>> {
    let clone_testing = CloneTesting::new(chain.clone())?;
    let abstr_deployment = AbstractClient::new(clone_testing.clone())?;

    let pyth_address = abstr_deployment
        .name_service()
        .contracts(vec![ContractEntry {
            protocol: PYTH.to_string(),
            contract: "oracle".to_string(),
        }])?
        .contracts[0]
        .1
        .clone();

    let price_data: PythApiResponse =
        reqwest::blocking::get(format!("{}{}", ORACLE_PRICE_API, PRICE_SOURCE_KEY))?.json()?;

    let update_data: Vec<Binary> = price_data
        .binary
        .data
        .iter()
        .map(|d| Binary::new(hex::decode(d).unwrap()))
        .collect();

    // We send an update to the oracle contract
    let update_fee: Coin = clone_testing.query(
        &pyth_sdk_cw::QueryMsg::GetUpdateFee {
            vaas: update_data.clone(),
        },
        &pyth_address,
    )?;
    clone_testing.add_balance(&clone_testing.sender, vec![update_fee.clone()])?;
    clone_testing.execute(
        &pyth_sdk_cw::ExecuteMsg::UpdatePriceFeeds {
            data: update_data.clone(),
        },
        &[update_fee],
        &pyth_address,
    )?;

    let tester = PythOracleTester {
        current_oracle_price_data: price_data,
    };
    OracleTester::new_live(abstr_deployment, tester)
}

fn test_price_query(chain: ChainInfo) -> anyhow::Result<()> {
    let oracle_tester = setup_clone_testing(chain)?;
    let current_price = oracle_tester.test_price()?;

    let raw_price = oracle_tester.oracle.current_oracle_price_data.parsed[0]
        .price
        .price
        .clone();
    // We assume this price has 8 decimals
    let price = Uint128::from_str(&raw_price)? / Uint128::from(100_000_000u128);
    assert_eq!(current_price.price.to_uint_floor(), price);

    Ok(())
}

#[test]
fn test_xion() {
    test_price_query(XION_TESTNET_1).unwrap();
}
#[test]
fn test_osmo_test() {
    test_price_query(OSMO_5).unwrap();
}
#[test]
fn test_pion() {
    test_price_query(PION_1).unwrap();
}
#[test]
fn test_osmosis() {
    test_price_query(OSMOSIS_1).unwrap();
}
#[test]
fn test_neutron() {
    test_price_query(NEUTRON_1).unwrap();
}
