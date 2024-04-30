use abstract_app::abstract_testing::OWNER;
use cw_orch::prelude::*;
use dca_app::{contract::DCA_APP_ID, DCA};

#[test]
fn successful_wasm() {
    // Create a sender
    let sender = Addr::unchecked(OWNER);
    // Create the mock
    let mock = Mock::new(sender);

    // Construct the counter interface
    let contract = DCA::new(DCA_APP_ID, mock);

    contract.wasm();
}
