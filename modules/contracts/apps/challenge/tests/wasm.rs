use abstract_testing::OWNER;
use challenge_app::contract::CHALLENGE_APP_ID;
use challenge_app::ChallengeApp;

use cw_orch::prelude::*;

#[test]
fn successful_wasm() {
    // Create a sender
    let sender = Addr::unchecked(OWNER);
    // Create the mock
    let mock = Mock::new(&sender);

    // Construct the counter interface
    let contract = ChallengeApp::new(CHALLENGE_APP_ID, mock);

    contract.wasm();
    let _ = contract.upload();
}
