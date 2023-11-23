mod common;

use cosmwasm_std::Addr;
use cw_orch::daemon::networks::JUNO_1;
use cw_orch::prelude::*;
use cw_orch_fork_mock::ForkMock;
use std::path::PathBuf;
use std::str::FromStr;

use cw20::Cw20QueryMsg;

use cosmwasm_std::Empty;

const VERSION: &str = env!("CARGO_PKG_VERSION");
// owner of the abstract infra
const SENDER: &str = "juno1kjzpqv393k4g064xh04j4hwy5d0s03wfvqejga";

#[test]
fn migrate_infra_success() -> anyhow::Result<()>{
    use abstract_core::objects::AccountId;
    use abstract_interface::Abstract;

    env_logger::init();

    let sender = Addr::unchecked(SENDER);

    // Instantiation of the fork platform is a breeze.
    let mut app = ForkMock::new(JUNO_1);
    app.set_sender(sender.clone());

    let abstr_deployment = Abstract::load_from(app)?;

    let a = abstr_deployment.version_control.get_account(AccountId::local(0))?;
    eprint!("account: {:?}", a);
    // We assert the balance has changed when depositing some funds

    Ok(())
}