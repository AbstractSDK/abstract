use app::contract::APP_ID;
use app::AppInterface;

use cw_orch::prelude::*;

// consts for testing
const ADMIN: &str = "admin";

#[test]
fn successful_wasm() {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Construct the counter interface
    let contract = AppInterface::new(APP_ID, mock);
    // Panics if no path to a .wasm file is found
    contract.wasm();
}
