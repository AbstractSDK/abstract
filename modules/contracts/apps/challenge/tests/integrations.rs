use crate::msg::QueryMsg;
use abstract_core::{
    app::BaseInstantiateMsg,
    objects::{
        gov_type::GovernanceDetails,
        module::{ModuleInfo, ModuleVersion},
    },
};
use abstract_dex_adapter::msg::OfferAsset;
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, *};
use challenge_app::{
    contract::{CHALLENGE_APP_ID, CHALLENGE_APP_VERSION},
    msg::{
        ChallengeQueryMsg, ChallengeRequest, ChallengeResponse, ChallengesResponse,
        CheckInsResponse, DurationChoice, FriendsResponse, InstantiateMsg, VoteResponse,
    },
    state::{
        ChallengeEntryUpdate, ChallengeStatus, CheckInStatus, Friend, UpdateFriendsOpKind, Vote,
        DAY,
    },
    *,
};
use cosmwasm_std::{coin, Uint128};
use cw_asset::AssetInfo;
use cw_orch::{anyhow, deploy::Deploy, prelude::*};
use lazy_static::lazy_static;

// consts for testing
const ADMIN: &str = "admin";
const DENOM: &str = "TOKEN";
const CHALLENGE_ID: u64 = 1;

lazy_static! {
    static ref CHALLENGE_REQ: ChallengeRequest = ChallengeRequest {
        name: "test".to_string(),
        collateral: OfferAsset::new("denom", Uint128::new(100_000_000_000)),
        description: "Test Challenge".to_string(),
        end: DurationChoice::OneHundredYears,
    };
    static ref ALICE_ADDRESS: String = "alice0x".to_string();
    static ref BOB_ADDRESS: String = "bob0x".to_string();
    static ref CHARLIE_ADDRESS: String = "charlie0x".to_string();
    static ref ALICE_NAME: String = "Alice".to_string();
    static ref BOB_NAME: String = "Bob".to_string();
    static ref CHARLIE_NAME: String = "Charlie".to_string();
    static ref FRIENDS: Vec<Friend<String>> = vec![
        Friend {
            address: ALICE_ADDRESS.clone(),
            name: ALICE_NAME.clone(),
        },
        Friend {
            address: BOB_ADDRESS.clone(),
            name: BOB_NAME.clone(),
        },
        Friend {
            address: CHARLIE_ADDRESS.clone(),
            name: CHARLIE_NAME.clone(),
        },
    ];
    static ref ALICE: Friend<String> = Friend {
        address: ALICE_ADDRESS.clone(),
        name: ALICE_NAME.clone(),
    };
    static ref BOB: Friend<String> = Friend {
        address: BOB_ADDRESS.clone(),
        name: BOB_NAME.clone(),
    };
    static ref ALICE_VOTE: Vote<String> = Vote {
        voter: ALICE_ADDRESS.clone(),
        approval: Some(true),
        for_check_in: None,
    };
    static ref VOTES: Vec<Vote<String>> = vec![
        Vote {
            voter: ALICE_ADDRESS.clone(),
            approval: Some(true),
            for_check_in: None,
        },
        Vote {
            voter: BOB_ADDRESS.clone(),
            approval: Some(true),
            for_check_in: None,
        },
        Vote {
            voter: CHARLIE_ADDRESS.clone(),
            approval: Some(true),
            for_check_in: None,
        },
    ];
    static ref ALICE_NO_VOTE: Vote<String> = Vote {
        voter: ALICE_ADDRESS.clone(),
        approval: Some(false),
        for_check_in: None,
    };
    static ref ONE_NO_VOTE: Vec<Vote<String>> = vec![
        ALICE_NO_VOTE.clone(),
        Vote {
            voter: BOB_ADDRESS.clone(),
            approval: Some(true),
            for_check_in: None,
        },
        Vote {
            voter: CHARLIE_ADDRESS.clone(),
            approval: Some(true),
            for_check_in: None,
        },
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
    mock.set_balance(&sender, vec![coin(50_000_000, DENOM)])?;

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
            module: Empty {},
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

    let query_res = QueryMsg::from(ChallengeQueryMsg::Challenge { challenge_id: 1 });
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

    let challenge_query = QueryMsg::from(ChallengeQueryMsg::Challenge { challenge_id: 1 });

    let created = apps
        .challenge_app
        .query::<ChallengeResponse>(&challenge_query)?;

    assert_eq!(created.challenge.as_ref().unwrap().name, "test".to_string());
    assert_eq!(
        created.challenge.as_ref().unwrap().collateral,
        challenge_req.collateral
    );
    assert_eq!(
        created.challenge.as_ref().unwrap().description,
        challenge_req.description
    );
    assert_eq!(
        created.challenge.as_ref().unwrap().status,
        ChallengeStatus::Active
    );
    Ok(())
}

#[test]
fn test_should_update_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;

    let query = QueryMsg::from(ChallengeQueryMsg::Challenge { challenge_id: 1 });
    apps.challenge_app.query::<ChallengeResponse>(&query)?;

    let to_update = ChallengeEntryUpdate {
        name: Some("update-test".to_string()),
        description: Some("Updated Test Challenge".to_string()),
    };

    apps.challenge_app.update_challenge(to_update.clone(), 1)?;
    let res = apps.challenge_app.query::<ChallengeResponse>(&query)?;

    assert_eq!(
        res.challenge.as_ref().unwrap().name,
        to_update.name.unwrap()
    );
    assert_eq!(
        res.challenge.as_ref().unwrap().description,
        to_update.description.unwrap(),
    );
    Ok(())
}

#[test]
fn test_should_cancel_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;

    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;
    let query = QueryMsg::from(ChallengeQueryMsg::Challenge { challenge_id: 1 });

    apps.challenge_app.query::<ChallengeResponse>(&query)?;

    apps.challenge_app.cancel_challenge(1)?;

    let res = apps.challenge_app.query::<ChallengeResponse>(&query)?;

    assert_eq!(res.challenge.unwrap().status, ChallengeStatus::Cancelled);
    Ok(())
}

#[test]
fn test_should_add_single_friend_for_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;

    apps.challenge_app
        .query::<ChallengeResponse>(&QueryMsg::from(ChallengeQueryMsg::Challenge {
            challenge_id: 1,
        }))?;

    apps.challenge_app.update_friends_for_challenge(
        1,
        vec![ALICE.clone()],
        UpdateFriendsOpKind::Add,
    )?;

    let response = apps
        .challenge_app
        .query::<FriendsResponse>(&QueryMsg::from(ChallengeQueryMsg::Friends {
            challenge_id: 1,
        }))?;

    for friend in response.0.iter() {
        assert_eq!(friend.address, Addr::unchecked(ALICE_ADDRESS.clone()));
        assert_eq!(friend.name, ALICE_NAME.clone());
    }
    Ok(())
}

#[test]
fn test_should_add_friends_for_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;

    apps.challenge_app
        .query::<ChallengeResponse>(&QueryMsg::from(ChallengeQueryMsg::Challenge {
            challenge_id: 1,
        }))?;

    apps.challenge_app.update_friends_for_challenge(
        1,
        FRIENDS.clone(),
        UpdateFriendsOpKind::Add,
    )?;

    let response = apps
        .challenge_app
        .query::<FriendsResponse>(&QueryMsg::from(ChallengeQueryMsg::Friends {
            challenge_id: 1,
        }))?;

    assert_eq!(
        response.0,
        vec![
            Friend {
                address: Addr::unchecked(ALICE_ADDRESS.clone()),
                name: "Alice".to_string(),
            },
            Friend {
                address: Addr::unchecked(BOB_ADDRESS.clone()),
                name: "Bob".to_string(),
            },
            Friend {
                address: Addr::unchecked(CHARLIE_ADDRESS.clone()),
                name: "Charlie".to_string(),
            }
        ]
    );

    Ok(())
}

#[test]
fn test_should_remove_friend_from_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;

    let created = apps
        .challenge_app
        .query::<ChallengeResponse>(&QueryMsg::from(ChallengeQueryMsg::Challenge {
            challenge_id: 1,
        }))?;

    let challenge = CHALLENGE_REQ.clone();
    assert_eq!(created.challenge.as_ref().unwrap().name, challenge.name);
    assert_eq!(
        created.challenge.as_ref().unwrap().collateral,
        challenge.collateral
    );
    assert_eq!(
        created.challenge.as_ref().unwrap().description,
        challenge.description
    );
    assert_eq!(
        created.challenge.as_ref().unwrap().status,
        ChallengeStatus::Active
    );

    // add friend
    apps.challenge_app.update_friends_for_challenge(
        1,
        vec![ALICE.clone()],
        UpdateFriendsOpKind::Add,
    )?;

    let friend_query = QueryMsg::from(ChallengeQueryMsg::Friends { challenge_id: 1 });

    let response = apps.challenge_app.query::<FriendsResponse>(&friend_query)?;

    for friend in response.0.iter() {
        assert_eq!(friend.address, Addr::unchecked(ALICE_ADDRESS.clone()));
        assert_eq!(friend.name, ALICE_NAME.clone());
    }

    // remove friend
    apps.challenge_app.update_friends_for_challenge(
        1,
        vec![ALICE.clone()],
        UpdateFriendsOpKind::Remove,
    )?;

    let response = apps.challenge_app.query::<FriendsResponse>(&friend_query)?;
    assert_eq!(response.0.len(), 0);

    Ok(())
}

#[test]
fn test_should_cast_vote() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;

    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;
    apps.challenge_app.daily_check_in(CHALLENGE_ID, None)?;
    apps.challenge_app.update_friends_for_challenge(
        CHALLENGE_ID,
        vec![ALICE.clone(), BOB.clone()],
        UpdateFriendsOpKind::Add,
    )?;

    let response = apps
        .challenge_app
        .query::<CheckInsResponse>(&QueryMsg::from(ChallengeQueryMsg::CheckIns {
            challenge_id: CHALLENGE_ID,
        }))?;

    assert_eq!(
        response.0.last().unwrap().status,
        CheckInStatus::CheckedInNotYetVoted
    );

    apps.challenge_app
        .cast_vote(CHALLENGE_ID, ALICE_VOTE.clone())?;

    let response = apps
        .challenge_app
        .query::<CheckInsResponse>(&QueryMsg::from(ChallengeQueryMsg::CheckIns {
            challenge_id: CHALLENGE_ID,
        }))?;

    let response =
        apps.challenge_app
            .query::<VoteResponse>(&QueryMsg::from(ChallengeQueryMsg::Vote {
                last_check_in: response.0.last().unwrap().last.nanos(),
                voter_addr: ALICE_ADDRESS.clone(),
                challenge_id: CHALLENGE_ID,
            }))?;

    assert_eq!(response.vote.unwrap().approval, Some(true));
    Ok(())
}

#[test]
fn test_should_not_charge_penalty_for_truthy_votes() -> anyhow::Result<()> {
    let (mock, account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;
    apps.challenge_app.update_friends_for_challenge(
        CHALLENGE_ID,
        FRIENDS.clone(),
        UpdateFriendsOpKind::Add,
    )?;

    run_challenge_vote_sequence(&mock, &apps, VOTES.clone())?;

    let response = apps
        .challenge_app
        .query::<CheckInsResponse>(&QueryMsg::from(ChallengeQueryMsg::CheckIns {
            challenge_id: CHALLENGE_ID,
        }))?;

    let vote =
        apps.challenge_app
            .query::<VoteResponse>(&QueryMsg::from(ChallengeQueryMsg::Vote {
                last_check_in: response.0.last().unwrap().last.nanos(),
                voter_addr: ALICE_ADDRESS.clone(),
                challenge_id: CHALLENGE_ID,
            }))?;

    assert_eq!(vote.vote.unwrap().approval, Some(true));

    let balance = mock.query_balance(&account.proxy.address()?, DENOM)?;
    // if no one voted false, no penalty should be charged, so balance will be 50_000_000
    assert_eq!(balance, Uint128::new(50_000_000));
    Ok(())
}

#[test]
fn test_should_charge_penalty_for_false_votes() -> anyhow::Result<()> {
    let (mock, account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;
    let response = apps
        .challenge_app
        .query::<ChallengeResponse>(&QueryMsg::from(ChallengeQueryMsg::Challenge {
            challenge_id: CHALLENGE_ID,
        }))?;

    let challenge = CHALLENGE_REQ.clone();
    assert_eq!(response.challenge.as_ref().unwrap().name, challenge.name);
    assert_eq!(
        response.challenge.as_ref().unwrap().collateral,
        challenge.collateral
    );
    assert_eq!(
        response.challenge.as_ref().unwrap().description,
        challenge.description
    );
    assert_eq!(
        response.challenge.as_ref().unwrap().status,
        ChallengeStatus::Active
    );

    apps.challenge_app.update_friends_for_challenge(
        CHALLENGE_ID,
        FRIENDS.clone(),
        UpdateFriendsOpKind::Add,
    )?;

    println!("Running challenge vote sequence");
    run_challenge_vote_sequence(&mock, &apps, ONE_NO_VOTE.clone())?;
    let check_ins_res = apps
        .challenge_app
        .query::<CheckInsResponse>(&QueryMsg::from(ChallengeQueryMsg::CheckIns {
            challenge_id: CHALLENGE_ID,
        }))?;

    let response =
        apps.challenge_app
            .query::<VoteResponse>(&QueryMsg::from(ChallengeQueryMsg::Vote {
                last_check_in: check_ins_res.0.last().unwrap().last.nanos(),
                voter_addr: ALICE_ADDRESS.clone(),
                challenge_id: CHALLENGE_ID,
            }))?;
    assert_eq!(response.vote.unwrap().approval, Some(false));

    let response =
        apps.challenge_app
            .query::<VoteResponse>(&QueryMsg::from(ChallengeQueryMsg::Vote {
                last_check_in: check_ins_res.0.last().unwrap().last.nanos(),
                voter_addr: BOB_ADDRESS.clone(),
                challenge_id: CHALLENGE_ID,
            }))?;
    assert_eq!(response.vote.unwrap().approval, Some(true));

    let response =
        apps.challenge_app
            .query::<VoteResponse>(&QueryMsg::from(ChallengeQueryMsg::Vote {
                last_check_in: check_ins_res.0.last().unwrap().last.nanos(),
                voter_addr: CHARLIE_ADDRESS.clone(),
                challenge_id: CHALLENGE_ID,
            }))?;
    assert_eq!(response.vote.unwrap().approval, Some(true));

    let balance = mock.query_balance(&account.proxy.address()?, DENOM)?;
    assert_eq!(balance, Uint128::new(44537609));
    Ok(())
}

#[test]
fn test_should_query_challenges_within_range() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    for _ in 0..10 {
        apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;
    }

    let response = apps
        .challenge_app
        .query::<ChallengesResponse>(&QueryMsg::from(ChallengeQueryMsg::Challenges {
            start_after: 0,
            limit: 5,
        }))?;

    assert_eq!(response.0.len(), 5);
    Ok(())
}

#[test]
fn test_should_query_challenges_within_different_range() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    for _ in 0..10 {
        apps.challenge_app.create_challenge(CHALLENGE_REQ.clone())?;
    }

    let response = apps
        .challenge_app
        .query::<ChallengesResponse>(&QueryMsg::from(ChallengeQueryMsg::Challenges {
            start_after: 5,
            limit: 8,
        }))?;

    // 10 challenges exist, but we start after 5 and limit to 8,
    // so we should get 3 challenges
    assert_eq!(response.0.len(), 3);
    Ok(())
}

fn run_challenge_vote_sequence(
    mock: &Mock,
    apps: &DeployedApps,
    votes: Vec<Vote<String>>,
) -> anyhow::Result<()> {
    for _ in 0..3 {
        mock.wait_seconds(DAY - 100)?; // this ensure we don't miss the check in
        apps.challenge_app.daily_check_in(1, None)?;
    }

    for vote in votes.clone() {
        apps.challenge_app.cast_vote(1, vote)?;
    }
    Ok(())
}
