use croncat_app::contract::CRONCAT_ID;
use croncat_app::Croncat;

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
    let contract = Croncat::new(CRONCAT_ID, mock);

    contract.wasm();
}
