use abstract_interface::Abstract;
use cosmwasm_std::coins;
use cosmwasm_std::Empty;
use cw_orch::prelude::*;
use osmosis_pool::OsmosisPools;

#[test]
fn deploy() {
    let chain = OsmosisTestTube::new(coins(100_000_000_000_000, "uosmo"));
    Abstract::deploy_on(chain.clone(), chain.sender().to_string()).unwrap();
    OsmosisPools::deploy_on(chain.clone(), Empty {}).unwrap();
}
