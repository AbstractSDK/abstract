use croncat_app::Croncat;
use cw_orch::{daemon::networks::OSMOSIS_1, prelude::*};

#[test]
fn successful_wasm() {
    Croncat::<MockBech32>::wasm(&OSMOSIS_1.into());
}
