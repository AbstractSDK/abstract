use cw_orch::{daemon::networks::OSMOSIS_1, prelude::*};
use dca_app::DCA;

#[test]
fn successful_wasm() {
    DCA::<MockBech32>::wasm(&OSMOSIS_1.into());
}
