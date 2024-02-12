use abstract_testing::OWNER;
use calendar_app::{contract::APP_ID, CalendarAppInterface};
use cw_orch::prelude::*;

#[test]
fn successful_wasm() {
    // Create the mock
    let mock = MockBech32::new("mock");

    // Construct the counter interface
    let contract = CalendarAppInterface::new(APP_ID, mock);
    // Panics if no path to a .wasm file is found
    contract.wasm();
}
