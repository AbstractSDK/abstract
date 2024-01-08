use app::contract::APP_ID;
use app::AppInterface;

use abstract_app::abstract_testing::OWNER;
use cw_orch::prelude::*;

/// This is the raw way to access the cw-orchestrator logic.
/// I.e. this does not use the AbstractClient.
#[test]
fn successful_wasm() {
    // Create a sender
    let sender = Addr::unchecked(OWNER);
    // Create the mock
    let mock = Mock::new(&sender);

    // Construct the counter interface
    let contract = AppInterface::new(APP_ID, mock);
    // Panics if no path to a .wasm file is found
    contract.wasm();
}
