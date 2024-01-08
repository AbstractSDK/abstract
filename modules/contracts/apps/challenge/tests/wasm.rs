use abstract_app::abstract_testing::OWNER;
use challenge_app::contract::CHALLENGE_APP_ID;
use challenge_app::Challenge;

use cw_orch::prelude::*;

#[test]
fn successful_wasm() {
    // Create a sender
    let sender = Addr::unchecked(OWNER);
    // Create the mock
    let mock = Mock::new(&sender);

    // Construct the counter interface
    let contract = Challenge::new(CHALLENGE_APP_ID, mock);

    contract.wasm();
    let _ = contract.upload();
}
