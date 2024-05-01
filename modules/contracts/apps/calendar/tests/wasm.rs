use calendar_app::CalendarAppInterface;
use cw_orch::{daemon::networks::OSMOSIS_1, prelude::*};

#[test]
fn successful_wasm() {
    CalendarAppInterface::<MockBech32>::wasm(&OSMOSIS_1.into());
}
