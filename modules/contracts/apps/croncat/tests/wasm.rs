use abstract_app::abstract_testing::OWNER;
use croncat_app::{contract::CRONCAT_ID, Croncat};
use cw_orch::prelude::*;

#[test]
fn successful_wasm() {
    // Create a sender
    let sender = Addr::unchecked(OWNER);
    // Create the mock
    let mock = Mock::new(sender);

    // Construct the counter interface
    let contract = Croncat::new(CRONCAT_ID, mock);

    contract.wasm();
}
