// #[cfg(test)]
// mod test_utils;

use std::str::FromStr;

use abstract_core::{objects::AssetEntry, app::BaseInstantiateMsg, ans_host::ExecuteMsgFns as AnsExecuteMsgFns, objects::gov_type::GovernanceDetails, version_control::ExecuteMsgFns as VersionControlExecuteMsgFns, app};
use abstract_core::app::BaseExecuteMsg;
use abstract_core::objects::{AccountId, AnsAsset};
use abstract_core::version_control::AccountBase;
use abstract_interface::{
    Abstract, AbstractAccount, AppDeployer, DeployStrategy,
    ManagerQueryFns,
};
use abstract_sdk::core as abstract_core;
use abstract_testing::addresses::TEST_NAMESPACE;
use abstract_testing::prelude::TEST_ADMIN;
use cosmwasm_std::{Addr, coins, Decimal, Uint128};
use cw_asset::AssetInfoUnchecked;
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;

use betting_app::{
    contract::CONTRACT_VERSION,
    BET_APP_ID,
    contract::interface::BetApp,
    msg::{BetInstantiateMsg, InstantiateMsg},
    msg::BetQueryMsgFns
};

use speculoos::prelude::*;
use betting_app::msg::{BetExecuteMsg, BetExecuteMsgFns, RoundResponse};
use betting_app::state::{AccountOdds, DEFAULT_RAKE_PERCENT, NewBet, RoundId, RoundInfo};

type AResult<T = ()> = anyhow::Result<T>;

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
    pub bet: BetApp<Env>,
    pub abstr: Abstract<Env>,
    pub env: Env,
}

const ADMIN_ACCOUNT_SEQ: u32 = 1;

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
            bet: bet_app,
            abstr,
        })
    }

    fn account(&self, seq: u32) -> AResult<AbstractAccount<Mock>> {
        Ok(AbstractAccount::new(&self.abstr, Some(AccountId::local(seq.into()))))
    }

    fn admin_account(&self) ->AResult<AbstractAccount<Mock>> {
        self.account(ADMIN_ACCOUNT_SEQ)
    }

    fn admin_account_addr(&self) -> AResult<Addr> {
        Ok(self.admin_account()?.manager.address()?)
    }

    // Add teams to the round with 0 odds to start
    fn add_x_teams_to_round(&self, round_id: RoundId, x: usize) -> AResult<()> {
        let account_ids = (0..x).map(|_| {
            let account= self.abstr.account_factory.create_default_account(GovernanceDetails::Monarchy {
                monarch: self.admin_account_addr().unwrap().into_string(),
            }).unwrap();
            let account_id = account.id().unwrap();
            AccountOdds {
                account_id,
                odds: Uint128::from(x as u128),
            }
        }).collect::<Vec<AccountOdds>>();

        self.manual_add_teams_to_round(round_id, account_ids)?;

        Ok(())
    }

    fn manual_add_teams_to_round(&self, round_id: RoundId, teams: Vec<AccountOdds>) -> AResult<()> {
        self.bet.call_as(&self.admin_account_addr()?).update_accounts(round_id, teams, vec![])?;

        Ok(())
    }

    fn create_test_round(&self) -> AResult<RoundId> {
        self.bet.call_as(&self.admin_account_addr()?).create_round(RoundInfo {
            name: "test".to_string(),
            description: "test".to_string(),
            base_bet_token: AssetEntry::new(BET_TOKEN_ANS_ID),
        })?;

        let rounds = self.bet.list_rounds(None, None)?;

        let last_round = rounds.rounds.last().unwrap();

        Ok(last_round.id)
    }
    // admin execute on round
    fn execute_as_account_on_round(&self, account: AbstractAccount<Mock>, msg: BetExecuteMsg) -> AResult<()> {
        account.manager.execute_on_module(BET_APP_ID, app::ExecuteMsg::<_, Empty>::Module(msg))?;

        Ok(())
    }

    fn register_on_round(&self, account: AbstractAccount<Mock>, round_id: RoundId) -> AResult<()> {
        self.execute_as_account_on_round(account, BetExecuteMsg::Register { round_id })?;

        Ok(())
    }

    fn bet_on_round_as(&self, sender: Addr, account_id: AccountId, round_id: RoundId, amount: u128) -> AResult<()> {
        let bet = NewBet {
            round_id,
            account_id,
            asset: AnsAsset::new(BET_TOKEN_ANS_ID, amount),
        };
        self.bet.call_as(&sender).place_bets(vec![bet], &coins(amount, BET_TOKEN_DENOM))?;

        Ok(())
    }
}



#[test]
fn test_init_config() -> AResult {
    let test_env = BetEnv::setup(None)?;

    let config = BetQueryMsgFns::config(&test_env.bet)?;

    assert_that!(config.bet_asset).is_equal_to(AssetEntry::new(BET_TOKEN_ANS_ID));
    assert_that!(config.rake).is_equal_to(Decimal::percent(DEFAULT_RAKE_PERCENT));


    Ok(())
}

#[test]
fn test_create_round() -> AResult {
    let env = BetEnv::setup(None)?;

    env.create_test_round()?;

    let rounds = env.bet.list_rounds(None, None)?;

    assert_that!(rounds.rounds).has_length(1);

    let round  = rounds.rounds[0].clone();

    assert_that!(round.name).is_equal_to("test".to_string());
    assert_that!(round.description).is_equal_to("test".to_string());
    assert_that!(round.teams).is_empty();
    assert_that!(round.winning_team).is_none();
    assert_that!(round.total_bets).is_equal_to(0);


    Ok(())
}



#[test]
fn test_create_round_with_teams() -> AResult {
    let env = BetEnv::setup(None)?;

    let round_id= env.create_test_round()?;

    env.add_x_teams_to_round(round_id, 10)?;

    let round = env.bet.round(round_id)?;
    assert_that!(&round.teams.len()).is_equal_to(10);

    Ok(())
}

#[test]
fn test_create_round_with_mini_bets() -> AResult {
    let env = BetEnv::setup(None)?;

    let round_id= env.create_test_round()?;

    env.add_x_teams_to_round(round_id, 2)?;

    let bettor = Addr::unchecked("account");
    let bet_amount = 100;


    env.env.set_balance(&bettor, coins(bet_amount, BET_TOKEN_DENOM))?;

    let betting_on = AccountId::local(2);

    env.bet_on_round_as(bettor, betting_on, round_id, bet_amount)?;

    Ok(())
}
