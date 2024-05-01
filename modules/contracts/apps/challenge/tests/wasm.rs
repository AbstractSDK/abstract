use challenge_app::{contract::CHALLENGE_APP_ID, Challenge};
use cw_orch::{daemon::networks::OSMOSIS_1, prelude::*};

#[test]
fn successful_wasm() {
    // Create the mock
    let mock = MockBech32::new("mock");

    // Construct the counter interface
    let contract = Challenge::new(CHALLENGE_APP_ID, mock);

    Challenge::<MockBech32>::wasm(&OSMOSIS_1.into());
    let _ = contract.upload();
}
