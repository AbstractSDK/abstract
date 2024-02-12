use abstract_app::abstract_testing::OWNER;
use challenge_app::{contract::CHALLENGE_APP_ID, Challenge};
use cw_orch::prelude::*;

#[test]
fn successful_wasm() {
    // Create the mock
    let mock = MockBech32::new("mock");

    // Construct the counter interface
    let contract = Challenge::new(CHALLENGE_APP_ID, mock);

    contract.wasm();
    let _ = contract.upload();
}
