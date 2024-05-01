use cw_orch::{daemon::networks::OSMOSIS_1, prelude::*};
use payment_app::PaymentAppInterface;

#[test]
fn successful_wasm() {
    PaymentAppInterface::<MockBech32>::wasm(&OSMOSIS_1.into());
}
