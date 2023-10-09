use crate::msg::QueryMsg;
use abstract_core::{
    app::BaseInstantiateMsg,
    objects::{
        gov_type::GovernanceDetails,
        module::{ModuleInfo, ModuleVersion},
        voting::{Threshold, Vote, VoteConfig, VoteOutcome, VoteStatus},
        AssetEntry,
    },
};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, *};
use challenge_app::{
    contract::{CHALLENGE_APP_ID, CHALLENGE_APP_VERSION},
    msg::{
        ChallengeEntryResponse, ChallengeInstantiateMsg, ChallengeQueryMsg, ChallengeRequest,
        ChallengeResponse, ChallengesResponse, FriendsResponse, InstantiateMsg, VoteResponse,
    },
    state::{AdminStrikes, ChallengeEntryUpdate, StrikeStrategy, UpdateFriendsOpKind},
    *,
};
use cosmwasm_std::{coin, Uint128};
use cw_asset::AssetInfo;
use cw_orch::{anyhow, deploy::Deploy, prelude::*};
use lazy_static::lazy_static;

const ADMIN: &str = "admin";
const DENOM: &str = "TOKEN";
const FIRST_CHALLENGE_ID: u64 = 1;

const INITIAL_BALANCE: u128 = 50_000_000;

lazy_static! {
    static ref CHALLENGE_REQ: ChallengeRequest = ChallengeRequest {
        name: "test".to_string(),
        strike_asset: AssetEntry::new("denom"),
        strike_strategy: StrikeStrategy::Split(Uint128::new(30_000_000)),
        description: "Test Challenge".to_string(),
        duration: cw_utils::HOUR,
        strikes_limit: None,
        init_friends: FRIENDS.clone()
    };
    static ref ALICE_ADDRESS: String = "alice".to_string();
    static ref BOB_ADDRESS: String = "bob".to_string();
    static ref CHARLIE_ADDRESS: String = "charlie".to_string();
    static ref FRIENDS: Vec<String> = vec![
        ALICE_ADDRESS.clone(),
        BOB_ADDRESS.clone(),
        CHARLIE_ADDRESS.clone()
    ];
}

#[allow(unused)]
struct DeployedApps {
    challenge_app: ChallengeApp<Mock>,
}

#[allow(clippy::type_complexity)]
fn setup() -> anyhow::Result<(Mock, AbstractAccount<Mock>, Abstract<Mock>, DeployedApps)> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);
    mock.set_balance(&sender, vec![coin(INITIAL_BALANCE, DENOM)])?;

    let mut challenge_app = ChallengeApp::new(CHALLENGE_APP_ID, mock.clone());
    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), sender.to_string())?;

    challenge_app.deploy(CHALLENGE_APP_VERSION.parse()?, DeployStrategy::Try)?;

    let _module_info = ModuleInfo::from_id(
        CHALLENGE_APP_ID,
        ModuleVersion::Version(CHALLENGE_APP_VERSION.to_string()),
    )?;

    abstr_deployment.ans_host.execute(
        &abstract_core::ans_host::ExecuteMsg::UpdateAssetAddresses {
            to_add: vec![("denom".to_owned(), AssetInfo::native(DENOM).into())],
            to_remove: vec![],
        },
        None,
    )?;

    let account_details: AccountDetails = AccountDetails {
        name: "test".to_string(),
        description: None,
        link: None,
        namespace: None,
        base_asset: None,
        install_modules: vec![],
    };

    let account = abstr_deployment.account_factory.create_new_account(
        account_details,
        GovernanceDetails::Monarchy {
            monarch: ADMIN.to_string(),
        },
        None,
    )?;

    let _ = account.install_module(
        CHALLENGE_APP_ID,
        &InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: abstr_deployment.ans_host.addr_str()?,
                version_control_address: abstr_deployment.version_control.addr_str()?,
            },
            module: ChallengeInstantiateMsg {
                vote_config: VoteConfig {
                    threshold: Threshold::Majority {},
                    veto_duration: None,
                },
            },
        },
        None,
    )?;

    let module_addr = account
        .manager
        .module_info(CHALLENGE_APP_ID)?
        .unwrap()
        .address;

    challenge_app.set_address(&module_addr);
    challenge_app.set_sender(&account.manager.address()?);
    mock.set_balance(
        &account.proxy.address()?,
        vec![coin(50_000_000, DENOM), coin(10_000, "eur")],
    )?;

    let deployed = DeployedApps { challenge_app };
    mock.wait_blocks(1000)?;
    Ok((mock, account, abstr_deployment, deployed))
}

#[test]
fn test_should_successful_install() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;

    let query_res = QueryMsg::from(ChallengeQueryMsg::Challenge {
        challenge_id: FIRST_CHALLENGE_ID,
    });
    assert_eq!(
        apps.challenge_app.query::<ChallengeResponse>(&query_res)?,
        ChallengeResponse { challenge: None }
    );

    Ok(())
}

#[test]
fn test_should_create_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    let challenge_req = CHALLENGE_REQ.clone();
    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;

    let challenge_query = QueryMsg::from(ChallengeQueryMsg::Challenge {
        challenge_id: FIRST_CHALLENGE_ID,
    });

    let created_challenge = apps
        .challenge_app
        .query::<ChallengeResponse>(&challenge_query)?
        .challenge
        .unwrap();

    let expected_response = ChallengeEntryResponse {
        challenge_id: FIRST_CHALLENGE_ID,
        name: challenge_req.name,
        strike_asset: challenge_req.strike_asset,
        strike_strategy: challenge_req.strike_strategy,
        description: challenge_req.description,
        end: challenge_req.duration.after(&_mock.block_info()?),
        status: VoteStatus::Active,
        admin_strikes: AdminStrikes {
            num_strikes: 0,
            limit: challenge_req.strikes_limit.unwrap_or(1),
        },
    };
    assert_eq!(created_challenge, expected_response);
    Ok(())
}

#[test]
fn test_update_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;

    let new_name = "update-test".to_string();
    let new_description = "Updated Test Challenge".to_string();

    let to_update = ChallengeEntryUpdate {
        name: Some(new_name.clone()),
        description: Some(new_description.clone()),
    };

    apps.challenge_app
        .update_challenge(to_update.clone(), FIRST_CHALLENGE_ID)?;
    let res: ChallengeResponse =
        apps.challenge_app
            .query(&QueryMsg::from(ChallengeQueryMsg::Challenge {
                challenge_id: FIRST_CHALLENGE_ID,
            }))?;
    let challenge = res.challenge.unwrap();

    assert_eq!(challenge.name, new_name);
    assert_eq!(challenge.description, new_description,);
    Ok(())
}

#[test]
fn test_cancel_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;

    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;

    apps.challenge_app.cancel_challenge(FIRST_CHALLENGE_ID)?;

    let res: ChallengeResponse =
        apps.challenge_app
            .query(&QueryMsg::from(ChallengeQueryMsg::Challenge {
                challenge_id: FIRST_CHALLENGE_ID,
            }))?;
    let challenge = res.challenge.unwrap();

    assert_eq!(
        challenge.status,
        VoteStatus::Finished(VoteOutcome::Canceled)
    );
    Ok(())
}

#[test]
fn test_add_single_friend_for_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;

    let new_friend = "new_friend".to_string();

    apps.challenge_app.update_friends_for_challenge(
        FIRST_CHALLENGE_ID,
        vec![new_friend.clone()],
        UpdateFriendsOpKind::Add {},
    )?;

    let response: FriendsResponse =
        apps.challenge_app
            .query(&QueryMsg::from(ChallengeQueryMsg::Friends {
                challenge_id: FIRST_CHALLENGE_ID,
            }))?;
    let mut friends = response.friends;
    friends.sort();

    let mut expected_friends: Vec<Addr> = FRIENDS.iter().map(Addr::unchecked).collect();
    expected_friends.push(Addr::unchecked(new_friend));
    expected_friends.sort();

    assert_eq!(friends, expected_friends);

    Ok(())
}

#[test]
fn test_add_friends_for_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    let challenge_req_without_friends = ChallengeRequest {
        init_friends: vec![],
        ..CHALLENGE_REQ.clone()
    };

    apps.challenge_app
        .create_challenge(challenge_req_without_friends)?;

    apps.challenge_app.update_friends_for_challenge(
        FIRST_CHALLENGE_ID,
        FRIENDS.clone(),
        UpdateFriendsOpKind::Add {},
    )?;

    let response: FriendsResponse =
        apps.challenge_app
            .query(&QueryMsg::from(ChallengeQueryMsg::Friends {
                challenge_id: FIRST_CHALLENGE_ID,
            }))?;
    let mut friends = response.friends;
    friends.sort();

    let expected_friends: Vec<Addr> = FRIENDS.iter().map(Addr::unchecked).collect();

    assert_eq!(friends, expected_friends);

    Ok(())
}

#[test]
fn test_remove_friend_from_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;

    // remove friend
    apps.challenge_app.update_friends_for_challenge(
        1,
        vec![ALICE_ADDRESS.clone()],
        UpdateFriendsOpKind::Remove {},
    )?;

    let friends_query = QueryMsg::from(ChallengeQueryMsg::Friends {
        challenge_id: FIRST_CHALLENGE_ID,
    });

    let response: FriendsResponse = apps.challenge_app.query(&friends_query)?;
    let mut friends = response.friends;
    friends.sort();

    let mut expected_friends = FRIENDS.clone();
    expected_friends.retain(|s| s != ALICE_ADDRESS.as_str());
    expected_friends.sort();

    assert_eq!(friends, expected_friends);

    Ok(())
}

#[test]
fn test_cast_vote() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;

    let vote = Vote {
        vote: true,
        memo: Some("some memo".to_owned()),
    };
    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;
    apps.challenge_app
        .call_as(&Addr::unchecked(ALICE_ADDRESS.clone()))
        .cast_vote(FIRST_CHALLENGE_ID, vote.clone())?;

    let response: VoteResponse =
        apps.challenge_app
            .query(&QueryMsg::from(ChallengeQueryMsg::Vote {
                voter_addr: ALICE_ADDRESS.clone(),
                challenge_id: FIRST_CHALLENGE_ID,
                previous_vote_index: None,
            }))?;

    assert_eq!(response.vote, Some(vote));
    Ok(())
}

#[test]
fn test_not_charge_penalty_for_voting_false() -> anyhow::Result<()> {
    let (mock, account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;

    let votes = vec![
        (
            Addr::unchecked(ALICE_ADDRESS.clone()),
            Vote {
                vote: false,
                memo: None,
            },
        ),
        (
            Addr::unchecked(BOB_ADDRESS.clone()),
            Vote {
                vote: false,
                memo: None,
            },
        ),
        (
            Addr::unchecked(CHARLIE_ADDRESS.clone()),
            Vote {
                vote: false,
                memo: None,
            },
        ),
    ];
    run_challenge_vote_sequence(&mock, &apps, votes)?;

    let balance = mock.query_balance(&account.proxy.address()?, DENOM)?;
    // if no one voted true, no penalty should be charged, so balance will be 50_000_000
    assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
    Ok(())
}

#[test]
fn test_charge_penalty_for_voting_true() -> anyhow::Result<()> {
    let (mock, account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;

    let votes = vec![
        (
            Addr::unchecked(ALICE_ADDRESS.clone()),
            Vote {
                vote: true,
                memo: None,
            },
        ),
        (
            Addr::unchecked(BOB_ADDRESS.clone()),
            Vote {
                vote: true,
                memo: None,
            },
        ),
        (
            Addr::unchecked(CHARLIE_ADDRESS.clone()),
            Vote {
                vote: true,
                memo: None,
            },
        ),
    ];
    run_challenge_vote_sequence(&mock, &apps, votes)?;

    let balance = mock.query_balance(&account.proxy.address()?, DENOM)?;
    // Initial balance - strike
    assert_eq!(balance, Uint128::new(INITIAL_BALANCE - 30_000_000));
    Ok(())
}

#[test]
fn test_query_challenges_within_range() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    for _ in 0..10 {
        apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;
    }

    let response: ChallengesResponse =
        apps.challenge_app
            .query(&QueryMsg::from(ChallengeQueryMsg::Challenges {
                start_after: None,
                limit: Some(5),
            }))?;

    assert_eq!(response.challenges.len(), 5);
    Ok(())
}

#[test]
fn test_should_query_challenges_within_different_range() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    for _ in 0..10 {
        apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;
    }

    let response: ChallengesResponse =
        apps.challenge_app
            .query(&QueryMsg::from(ChallengeQueryMsg::Challenges {
                start_after: Some(7),
                limit: Some(8),
            }))?;

    // 10 challenges exist, but we start after 7 and limit to 8,
    // so we should get 3 challenges
    assert_eq!(response.challenges.len(), 3);
    Ok(())
}

fn run_challenge_vote_sequence(
    _mock: &Mock,
    apps: &DeployedApps,
    votes: Vec<(Addr, Vote)>,
) -> anyhow::Result<()> {
    for (signer, vote) in votes {
        apps.challenge_app
            .call_as(&signer)
            .cast_vote(FIRST_CHALLENGE_ID, vote)?;
    }
    apps.challenge_app.count_votes(FIRST_CHALLENGE_ID)?;
    Ok(())
}
