mod common;

use abstract_interface::Abstract;
use cosmwasm_std::Addr;
use cw_orch::daemon::networks::JUNO_1;
use cw_orch::prelude::*;
use cw_orch_fork_mock::ForkMock;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::runtime::Runtime;

use cw20::Cw20QueryMsg;

use cosmwasm_std::Empty;

const VERSION: &str = env!("CARGO_PKG_VERSION");
// owner of the abstract infra
const SENDER: &str = "juno1kjzpqv393k4g064xh04j4hwy5d0s03wfvqejga";

#[test]
fn migrate_infra_success() -> anyhow::Result<()> {
    use abstract_core::objects::AccountId;
    use abstract_interface::Abstract;
    let runtime = Runtime::new().unwrap();
    env_logger::init();

    let sender = Addr::unchecked(SENDER);

    // Instantiation of the fork platform is a breeze.
    let mut app = ForkMock::new(&runtime, JUNO_1)?;
    app.set_sender(sender.clone());

    let abstr_deployment = Abstract::load_from(app)?;
    
    assert_eq!(abstr_deployment.version_control.code_id()?, 3692);
    abstr_deployment.migrate_if_needed()?;
    assert_eq!(abstr_deployment.version_control.code_id()?, 5000003);
    abstr_deployment.migrate_if_needed()?;
    assert_eq!(abstr_deployment.version_control.code_id()?, 5000003);
    Ok(())
}
