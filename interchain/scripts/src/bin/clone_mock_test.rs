use abstract_interface::Abstract;
use abstract_std::ibc_host::ExecuteMsgFns;
use abstract_std::ibc_host::HostAction;
use abstract_std::ibc_host::InternalAction;
use abstract_std::ibc_host::QueryMsgFns;
use abstract_std::objects::AccountId;
use cw_orch::daemon::networks::OSMOSIS_1;
use cw_orch::prelude::*;
use cw_orch_clone_testing::CloneTesting;

fn main() -> anyhow::Result<()> {
    let chain = OSMOSIS_1;
    let mut app = CloneTesting::new(chain)?;
    // Set the sender to the host proxy
    let abs = Abstract::load_from(app.clone())?;
    let proxy = abs.ibc.host.client_proxy("phoenix".to_string())?;

    app.set_sender(proxy.proxy.clone());
    let abs = Abstract::load_from(app.clone())?;

    env_logger::init();
    log::info!("Terra proxy {}", proxy.proxy);

    // Send a register message to the host
    abs.ibc.host.ibc_execute(
        "terra15zg7mvqxug2h4nv58u985kk89xaek49zu3cr8sylvq83ts44peaszjqsng".to_string(),
        AccountId::local(0),
        HostAction::Internal(InternalAction::Register {
            name: Some("Default Abstract Account".to_string()),
            description: None,
            link: None,
            namespace: None,
            install_modules: vec![],
        }),
    )?;

    Ok(())
}
