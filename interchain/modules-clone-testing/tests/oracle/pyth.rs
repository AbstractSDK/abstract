use std::str::FromStr;

use abstract_app::{objects::UncheckedContractEntry, std::ans_host::QueryMsgFns};
use abstract_client::AbstractClient;
use abstract_interface::ExecuteMsgFns;
use abstract_oracle_adapter::{
    oracle_tester::{MockOracle, OracleTester},
    oracles::PYTH,
};
use cosmwasm_std::{Addr, Binary, Uint128};
use cw_orch::daemon::networks::XION_TESTNET_1;
use cw_orch::prelude::*;
use cw_orch_clone_testing::CloneTesting;
use pyth_api::PythApiResponse;

pub const PYTH_XION_ADDRESS: &str =
    "xion1w39ctwxxhxxc2kxarycjxj9rndn65gf8daek7ggarwh3rq3zl0lqqllnmt";

// Use https://hermes.pyth.network/docs/#/rest/latest_price_updates to query latest update
pub const ORACLE_PRICE_API: &str = "https://hermes.pyth.network/v2/updates/price/latest?ids%5B%5D=";
pub const PRICE_SOURCE_KEY: &str =
    "e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43";

pub mod pyth_api {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct PythApiResponse {
        pub binary: PythApiResponseBinary,
        pub parsed: Vec<PythApiResponseparsed>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct PythApiResponseBinary {
        pub data: Vec<String>,
    }
    #[derive(Serialize, Deserialize)]
    pub struct PythApiResponseparsed {
        pub price: PythApiResponsePrice,
    }
    #[derive(Serialize, Deserialize)]
    pub struct PythApiResponsePrice {
        pub price: String,
    }
}

pub struct PythOracleTester {
    pub current_oracle_price_data: PythApiResponse,
    pub chain: CloneTesting,
}

impl MockOracle<CloneTesting> for PythOracleTester {
    const MAX_AGE: u64 = 60;
    fn price_source_key(&self) -> String {
        PRICE_SOURCE_KEY.to_string()
    }

    fn name(&self) -> String {
        PYTH.to_string()
    }

    fn ans_setup(&self, abstr_deployment: &AbstractClient<CloneTesting>) -> anyhow::Result<()> {
        abstr_deployment.name_service().update_contract_addresses(
            vec![(
                UncheckedContractEntry {
                    protocol: PYTH.to_string(),
                    contract: "oracle".to_string(),
                },
                PYTH_XION_ADDRESS.to_string(),
            )],
            vec![],
        )?;
        Ok(())
    }
}

fn setup_clone_testing() -> anyhow::Result<OracleTester<CloneTesting, PythOracleTester>> {
    let clone_testing = CloneTesting::new(XION_TESTNET_1)?;
    let pyth_addr = Addr::unchecked(PYTH_XION_ADDRESS);

    let price_data: PythApiResponse =
        reqwest::blocking::get(format!("{}{}", ORACLE_PRICE_API, PRICE_SOURCE_KEY))?.json()?;

    let update_data: Vec<Binary> = price_data
        .binary
        .data
        .iter()
        .map(|d| Binary::new(hex::decode(d).unwrap()))
        .collect();

    // We send an update to the oracle contract (no update for now)
    let update_fee: Coin = clone_testing.query(
        &pyth_sdk_cw::QueryMsg::GetUpdateFee {
            vaas: update_data.clone(),
        },
        &pyth_addr,
    )?;
    clone_testing.add_balance(&clone_testing.sender, vec![update_fee.clone()])?;
    clone_testing.execute(
        &pyth_sdk_cw::ExecuteMsg::UpdatePriceFeeds {
            data: update_data.clone(),
        },
        &[update_fee],
        &pyth_addr,
    )?;

    let abstr_deployment = AbstractClient::new(clone_testing.clone())?;
    let abstract_admin = abstr_deployment.name_service().ownership()?.owner.unwrap();
    let abstr_deployment = abstr_deployment.call_as(&Addr::unchecked(abstract_admin));

    let tester = PythOracleTester {
        chain: clone_testing,
        current_oracle_price_data: price_data,
    };
    OracleTester::new(abstr_deployment, tester)
}

#[test]
fn test_price_query() -> anyhow::Result<()> {
    let oracle_tester = setup_clone_testing()?;
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
