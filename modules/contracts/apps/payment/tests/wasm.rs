use abstract_testing::OWNER;
use payment_app::contract::APP_ID;
use payment_app::PaymentAppInterface;

use cw_orch::prelude::*;

#[test]
fn successful_wasm() {
    // Create a sender
    let sender = Addr::unchecked(OWNER);
    // Create the mock
    let mock = Mock::new(&sender);

    // Construct the counter interface
    let contract = PaymentAppInterface::new(APP_ID, mock);

    contract.wasm();
}
