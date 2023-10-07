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
use betting_app::state::{AccountOdds, DEFAULT_RAKE_PERCENT, NewBet, RoundId, RoundInfo, RoundStatus};

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


    fn add_team_to_round(&self, round_id: RoundId, account_id: AccountId, odds: Decimal) -> AResult<()> {
        self.manual_add_teams_to_round(round_id, vec![AccountOdds {
            account_id,
            odds,
        }])?;

        Ok(())
    }

    fn create_x_accounts(&self, x: usize) -> AResult<Vec<AccountId>> {
        let mut ids = vec![];

        for i in 0..x {
            let account = self.abstr.account_factory.create_default_account(GovernanceDetails::Monarchy {
                monarch: self.admin_account_addr().unwrap().into_string(),
            }).unwrap();
            let account_id = account.id().unwrap();
            ids.push(account_id);
        }

        Ok(ids)
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
                odds: Decimal::from_str("1").unwrap(),
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
        self.bet.call_as(&self.admin_account_addr()?).create_round(AssetEntry::new(BET_TOKEN_ANS_ID), "test".to_string(), "test".to_string())?;

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

    fn bet_on_round_as(&self, sender: Addr, round_id: RoundId, account_id: AccountId, amount: u128) -> AResult<()> {
        let bet = NewBet {
            round_id,
            account_id,
            asset: AnsAsset::new(BET_TOKEN_ANS_ID, amount),
        };
        self.bet.call_as(&sender).place_bet(bet, &coins(amount, BET_TOKEN_DENOM))?;

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

    let RoundResponse {
        id, name,
        description,
        teams,
        status,
        bet_count, total_bet,
    }  = rounds.rounds[0].clone();

    assert_that!(id).is_equal_to(0);
    assert_that!(name).is_equal_to("test".to_string());
    assert_that!(description).is_equal_to("test".to_string());
    assert_that!(teams).is_empty();
    assert_that!(status).is_equal_to(RoundStatus::Open);
    assert_that!(total_bet).is_equal_to(AnsAsset::new(BET_TOKEN_ANS_ID.to_string(), 0u128));
    assert_that!(bet_count).is_equal_to(0);


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

    let odds_list = env.bet.list_odds(round_id)?;
    println!("{:?}", odds_list);

    let better = Addr::unchecked("account");
    let bet_amount = 100;


    env.env.set_balance(&better, coins(bet_amount, BET_TOKEN_DENOM))?;

    let betting_on = AccountId::local(2);

    env.bet_on_round_as(better, round_id, betting_on, bet_amount)?;

    let odds_list = env.bet.list_odds(round_id)?;
    println!("{:?}", odds_list);

    Ok(())
}

#[test]
fn test_create_round_with_two_teams() -> AResult {
    let env = BetEnv::setup(None)?;

    let round_id= env.create_test_round()?;

    // create 2 accounts
    let mut new_acc_ids = env.create_x_accounts(2)?;

    let team_1 = new_acc_ids.swap_remove(0);
    let team_2 = new_acc_ids.swap_remove(0);


    env.add_team_to_round(round_id, team_1.clone(), Decimal::from_str("1").unwrap())?;
    env.add_team_to_round(round_id, team_2.clone(), Decimal::from_str("1").unwrap())?;

    let odds_list = env.bet.list_odds(round_id)?;
    println!("initial odds with house edge: {:?}", odds_list.odds);

    let better = Addr::unchecked("account");
    let bet_amount = 100000000;

    env.env.set_balance(&better, coins(bet_amount * 5999, BET_TOKEN_DENOM))?;


    env.bet_on_round_as(better.clone(), round_id, team_1, bet_amount)?;
    println!("odds_list 2: {:?}", odds_list.odds);

    env.bet_on_round_as(better, round_id, team_2, bet_amount / 2)?;

    let odds_list = env.bet.list_odds(round_id)?;
    println!("odds_list 3: {:?}", odds_list.odds);

    Ok(())
}


#[test]
fn test_create_round_with_three_teams() -> AResult {
    let env = BetEnv::setup(None)?;

    let round_id= env.create_test_round()?;

    // create 3 accounts
    let mut new_acc_ids = env.create_x_accounts(3)?;

    let team_1 = new_acc_ids.get(0).unwrap().clone();
    let team_2 =  new_acc_ids.get(1).unwrap().clone();
    let team_3 =  new_acc_ids.get(2).unwrap().clone();


    env.add_team_to_round(round_id, team_1.clone(), Decimal::from_str("1").unwrap())?;
    env.add_team_to_round(round_id, team_2.clone(), Decimal::from_str("1").unwrap())?;
    env.add_team_to_round(round_id, team_3.clone(), Decimal::from_str("1").unwrap())?;

    let odds_list = env.bet.list_odds(round_id)?;
    println!("initial odds: {:?}", odds_list.odds);

    let better = Addr::unchecked("account");
    env.env.set_balance(&better, coins(5005050 * 5999, BET_TOKEN_DENOM))?;

    let bet_amount = 125000000;
    env.bet_on_round_as(better.clone(), round_id, team_1, bet_amount)?;
    let odds_list = env.bet.list_odds(round_id)?;
    println!("odds_list 1: {:?}", odds_list.odds);

    let bet_amount = 175000000;
    env.bet_on_round_as(better.clone(), round_id, team_2, bet_amount)?;
    let odds_list = env.bet.list_odds(round_id)?;
    println!("odds_list 2: {:?}", odds_list.odds);

    let bet_amount = 45000000;
    env.bet_on_round_as(better.clone(), round_id, team_3, bet_amount)?;
    let odds_list = env.bet.list_odds(round_id)?;
    println!("odds_list 3: {:?}", odds_list.odds);

    Ok(())
}

/// Loser: 125
/// Winner: 200
/// Loser: 75
/// Expected payout: 200 * 1.8 = 360
#[test]
fn test_create_round_with_three_teams_and_claim() -> AResult {
    let env = BetEnv::setup(None)?;

    let round_id= env.create_test_round()?;

    // create 3 accounts
    let mut new_acc_ids = env.create_x_accounts(3)?;

    let team_1 = new_acc_ids.get(0).unwrap().clone();
    let team_2 =  new_acc_ids.get(1).unwrap().clone();
    let team_3 =  new_acc_ids.get(2).unwrap().clone();


    env.add_team_to_round(round_id, team_1.clone(), Decimal::from_str("1").unwrap())?;
    env.add_team_to_round(round_id, team_2.clone(), Decimal::from_str("1").unwrap())?;
    env.add_team_to_round(round_id, team_3.clone(), Decimal::from_str("1").unwrap())?;

    let odds_list = env.bet.list_odds(round_id)?;
    println!("initial odds: {:?}", odds_list.odds);

    let loser = Addr::unchecked("loser");
    let winner = Addr::unchecked("winner");
    env.env.set_balance(&loser, coins(200000000, BET_TOKEN_DENOM))?;
    env.env.set_balance(&winner, coins(200000000, BET_TOKEN_DENOM))?;

    let bet_amount = 125000000;
    env.bet_on_round_as(loser.clone(), round_id, team_1, bet_amount)?;
    let odds_list = env.bet.list_odds(round_id)?;
    println!("odds_list 1: {:?}", odds_list.odds);

    let bet_amount = 200000000;
    env.bet_on_round_as(winner.clone(), round_id, team_2.clone(), bet_amount)?;
    let odds_list = env.bet.list_odds(round_id)?;
    println!("odds_list 2: {:?}", odds_list.odds);

    let bet_amount = 75000000;
    env.bet_on_round_as(loser.clone(), round_id, team_3, bet_amount)?;
    let odds_list = env.bet.list_odds(round_id)?;
    println!("odds_list 3: {:?}", odds_list.odds);

    let odds_for_potential_winning_team = env.bet.odds(round_id, team_2.clone())?.odds;
    assert_that!(odds_for_potential_winning_team).is_equal_to(Decimal::from_str("1.8").unwrap());

    // set the winner
    env.bet.call_as(&env.admin_account_addr()?).close_round(round_id, Some(team_2))?;

    env.bet.distribute_winnings(round_id)?;
    let loser_balance = env.env.query_balance(&loser, BET_TOKEN_DENOM)?;
    assert_that!(loser_balance.u128()).is_equal_to(0);

    let winner_balance = env.env.query_balance(&winner, BET_TOKEN_DENOM)?;
    assert_that!(winner_balance.u128()).is_equal_to(360000000);

    Ok(())
}

/// Loser: 125
/// Winner 1: 200
/// Winner 2: 200
/// Loser: 75
/// Expected payout: 200 * 1.35 = 270
#[test]
fn test_create_round_with_three_teams_and_claim_multiple_winners() -> AResult {
    let env = BetEnv::setup(None)?;

    let round_id= env.create_test_round()?;

    // create 3 accounts
    let mut new_acc_ids = env.create_x_accounts(3)?;

    let team_1 = new_acc_ids.get(0).unwrap().clone();
    let team_2 =  new_acc_ids.get(1).unwrap().clone();
    let team_3 =  new_acc_ids.get(2).unwrap().clone();


    env.add_team_to_round(round_id, team_1.clone(), Decimal::from_str("1").unwrap())?;
    env.add_team_to_round(round_id, team_2.clone(), Decimal::from_str("1").unwrap())?;
    env.add_team_to_round(round_id, team_3.clone(), Decimal::from_str("1").unwrap())?;

    let odds_list = env.bet.list_odds(round_id)?;
    println!("initial odds: {:?}", odds_list.odds);

    let loser = Addr::unchecked("loser");
    let winner_1 = Addr::unchecked("winner_1");
    let winner_2 = Addr::unchecked("winner_2");
    env.env.set_balance(&loser, coins(200000000, BET_TOKEN_DENOM))?;
    env.env.set_balance(&winner_1, coins(200000000, BET_TOKEN_DENOM))?;
    env.env.set_balance(&winner_2, coins(200000000, BET_TOKEN_DENOM))?;

    let bet_amount = 125000000;
    env.bet_on_round_as(loser.clone(), round_id, team_1, bet_amount)?;
    let odds_list = env.bet.list_odds(round_id)?;
    println!("odds_list 1: {:?}", odds_list.odds);

    let bet_amount = 200000000;
    env.bet_on_round_as(winner_1.clone(), round_id, team_2.clone(), bet_amount)?;
    let odds_list = env.bet.list_odds(round_id)?;
    println!("odds_list 2: {:?}", odds_list.odds);

    let bet_amount = 200000000;
    env.bet_on_round_as(winner_2.clone(), round_id, team_2.clone(), bet_amount)?;
    let odds_list = env.bet.list_odds(round_id)?;
    println!("odds_list 2: {:?}", odds_list.odds);

    let bet_amount = 75000000;
    env.bet_on_round_as(loser.clone(), round_id, team_3, bet_amount)?;
    let odds_list = env.bet.list_odds(round_id)?;
    println!("odds_list 3: {:?}", odds_list.odds);

    let odds_for_potential_winning_team = env.bet.odds(round_id, team_2.clone())?.odds;
    assert_that!(odds_for_potential_winning_team).is_equal_to(Decimal::from_str("1.35").unwrap());

    // set the winner
    env.bet.call_as(&env.admin_account_addr()?).close_round(round_id, Some(team_2.clone()))?;


    let round = env.bet.round(round_id)?;
    assert_that!(round.status).is_equal_to(RoundStatus::Closed { winning_team: Some(team_2) });


    // distribute the winnings
    env.bet.distribute_winnings(round_id)?;
    let loser_balance = env.env.query_balance(&loser, BET_TOKEN_DENOM)?;
    assert_that!(loser_balance.u128()).is_equal_to(0);

    let winner_1_balance = env.env.query_balance(&winner_1, BET_TOKEN_DENOM)?;
    assert_that!(winner_1_balance.u128()).is_equal_to(270000000);

    let winner_2_balance = env.env.query_balance(&winner_2, BET_TOKEN_DENOM)?;
    assert_that!(winner_2_balance.u128()).is_equal_to(270000000);

    let round = env.bet.round(round_id)?;
    assert_that!(round.status).is_equal_to(RoundStatus::RewardsDistributed {});


    Ok(())
}

#[test]
fn test_draw() -> AResult {
    let env = BetEnv::setup(None)?;

    let round_id= env.create_test_round()?;

    // create 3 accounts
    let mut new_acc_ids = env.create_x_accounts(3)?;

    let team_1 = new_acc_ids.get(0).unwrap().clone();
    let team_2 =  new_acc_ids.get(1).unwrap().clone();
    let team_3 =  new_acc_ids.get(2).unwrap().clone();


    env.add_team_to_round(round_id, team_1.clone(), Decimal::from_str("1").unwrap())?;
    env.add_team_to_round(round_id, team_2.clone(), Decimal::from_str("1").unwrap())?;
    env.add_team_to_round(round_id, team_3.clone(), Decimal::from_str("1").unwrap())?;

    let odds_list = env.bet.list_odds(round_id)?;
    println!("initial odds: {:?}", odds_list.odds);

    let draw_1 = Addr::unchecked("dd");
    let draw_2 = Addr::unchecked("ddd");
    env.env.set_balance(&draw_1, coins(125000000, BET_TOKEN_DENOM))?;
    env.env.set_balance(&draw_2, coins(200000000, BET_TOKEN_DENOM))?;

    let bet_amount = 125000000;
    env.bet_on_round_as(draw_1.clone(), round_id, team_1, bet_amount)?;

    let bet_amount = 200000000;
    env.bet_on_round_as(draw_2.clone(), round_id, team_2.clone(), bet_amount)?;

    // set the winner
    env.bet.call_as(&env.admin_account_addr()?).close_round(round_id, None)?;

    let draw_1_balance = env.env.query_balance(&draw_1, BET_TOKEN_DENOM)?;
    assert_that!(draw_1_balance.u128()).is_equal_to(0);
    let draw_2_balance = env.env.query_balance(&draw_2, BET_TOKEN_DENOM)?;
    assert_that!(draw_2_balance.u128()).is_equal_to(0);

    env.bet.distribute_winnings(round_id)?;

    let draw_1_balance = env.env.query_balance(&draw_1, BET_TOKEN_DENOM)?;
    assert_that!(draw_1_balance.u128()).is_equal_to(125000000);
    let draw_2_balance = env.env.query_balance(&draw_2, BET_TOKEN_DENOM)?;
    assert_that!(draw_2_balance.u128()).is_equal_to(200000000);


    Ok(())
}

