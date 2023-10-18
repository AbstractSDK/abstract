use abstract_payment_app::contract::APP_ID;
use abstract_payment_app::PaymentApp;

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
    let contract = PaymentApp::new(APP_ID, mock);

    contract.wasm();
}
