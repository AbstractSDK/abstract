// #[cfg(test)]
// mod test_utils;

use std::str::FromStr;

use abstract_core::objects::AssetEntry;
use abstract_core::ans_host::ExecuteMsgFns as AnsExecuteMsgFns;
use abstract_core::app::BaseInstantiateMsg;
use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_core::version_control::ExecuteMsgFns as VersionControlExecuteMsgFns;
use abstract_interface::{
    Abstract, AbstractAccount, AppDeployer, DeployStrategy,
    ManagerQueryFns,
};
use abstract_sdk::core as abstract_core;
use abstract_testing::addresses::TEST_NAMESPACE;
use abstract_testing::prelude::TEST_ADMIN;
use cosmwasm_std::Addr;
use cw_asset::AssetInfoUnchecked;
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;

use betting_app::BET_APP_ID;
use betting_app::contract::CONTRACT_VERSION;
use betting_app::contract::interface::BetApp;
use betting_app::msg::{BetInstantiateMsg, InstantiateMsg};

type AResult = anyhow::Result<()>;

const ETF_MANAGER: &str = "etf_manager";
const ETF_TOKEN: &str = "etf_token";

// Returns an account with the necessary setup
fn setup_new_account<Env: CwEnv>(
    abstr_deployment: &Abstract<Env>,
    namespace: impl ToString,
) -> anyhow::Result<AbstractAccount<Env>> {
    // TODO: might need to move this
    let signing_account = abstr_deployment.account_factory.get_chain().sender();

    // Create a new account to install the app onto
    let account = abstr_deployment
        .account_factory
        .create_default_account(GovernanceDetails::Monarchy {
            monarch: signing_account.into_string(),
        })
        .unwrap();

    // claim the namespace so app can be deployed
    abstr_deployment
        .version_control
        .claim_namespace(account.id().unwrap(), namespace.to_string())
        .unwrap();

    // register base asset!
    // account.proxy.call_as(&abstr_deployment.account_factory.get_chain().sender).update_assets(vec![(AssetEntry::from(ISSUE_ASSET), UncheckedPriceSource::None)], vec![]).unwrap();

    Ok(account)
}

const BET_TOKEN_ANS_ID: &str = "testing>bet";
const BET_TOKEN_DENOM: &str = "factory/xxx/betting";

fn setup_default_assets<Env: CwEnv>(abstr: &Abstract<Env>) {
    // register juno as an asset
    abstr
        .ans_host
        .update_asset_addresses(
            vec![(
                BET_TOKEN_ANS_ID.to_string(),
                AssetInfoUnchecked::from_str(&format!("native:{}", BET_TOKEN_DENOM)).unwrap(),
            )],
            vec![],
        )
        .unwrap();
}

pub struct BetEnv<Env: CwEnv> {
    pub account: AbstractAccount<Env>,
    pub etf: BetApp<Env>,
    pub abstr: Abstract<Env>,
    pub env: Env,
}

impl BetEnv<Mock> {
    fn setup(initial_balance: Option<Vec<Coin>>) -> anyhow::Result<Self> {
        let owner = Addr::unchecked(TEST_ADMIN);

        // create testing environment
        let mock = Mock::new(&owner);

        let abstr = Abstract::deploy_on(mock.clone(), mock.sender().to_string()).unwrap();
        let bet_app = BetApp::new(BET_APP_ID, mock.clone());

        bet_app.deploy(CONTRACT_VERSION.parse().unwrap(), DeployStrategy::Force)?;

        let account = setup_new_account(&abstr, TEST_NAMESPACE)?;
        setup_default_assets(&abstr);

        account.install_module(
            BET_APP_ID,
            &InstantiateMsg {
                base: BaseInstantiateMsg {
                    ans_host_address: abstr.ans_host.addr_str()?,
                    version_control_address: abstr.version_control.addr_str()?,
                },
                module: BetInstantiateMsg {
                    rake: None,
                    bet_asset: AssetEntry::new(BET_TOKEN_ANS_ID),
                },
            },
            None,
        )?;

        let modules = account.manager.module_infos(None, None)?;
        bet_app.set_address(&modules.module_infos[0].address);

        Ok(Self {
            env: mock,
            account,
            etf: bet_app,
            abstr,
        })
    }
}


#[test]
fn test_init() -> AResult {
    let test_env = BetEnv::setup(None)?;

    Ok(())
}
