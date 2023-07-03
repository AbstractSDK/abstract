use abstract_dca_app::contract::DCA_APP_ID;
use abstract_dca_app::DCAApp;

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
    let contract = DCAApp::new(DCA_APP_ID, mock);

    contract.wasm();
}
