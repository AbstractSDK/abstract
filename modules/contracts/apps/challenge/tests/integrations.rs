// Use prelude to get all the necessary imports
use crate::msg::QueryMsg;
use abstract_challenge_app::{
    contract::{CHALLENGE_APP_ID, CHALLENGE_APP_VERSION},
    msg::{
        AppInstantiateMsg, ChallengeQueryMsg, ChallengeResponse, CheckInResponse, FriendsResponse,
        InstantiateMsg, VotesResponse,
    },
    state::{ChallengeEntry, ChallengeEntryUpdate, Friend, Penalty, UpdateFriendsOpKind, Vote},
    *,
};
use abstract_core::{
    app::BaseInstantiateMsg,
    objects::{
        gov_type::GovernanceDetails,
        module::{ModuleInfo, ModuleVersion},
        AssetEntry,
    },
};
use abstract_dex_adapter::msg::OfferAsset;
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, *};
use cosmwasm_std::{coin, Uint128};
use cw_asset::AssetInfo;
use cw_orch::{anyhow, deploy::Deploy, prelude::*};
use lazy_static::lazy_static;

// consts for testing
const ADMIN: &str = "admin";
const DENOM: &str = "TOKEN";
lazy_static! {
    static ref CHALLENGE: ChallengeEntry = ChallengeEntry {
        name: "test".to_string(),
        collateral: Penalty::FixedAmount {
            asset: OfferAsset::new("denom", Uint128::new(100)),
        },
        description: "Test Challenge".to_string(),
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
    static ref FRIEND: Friend<String> = Friend {
        address: ALICE_ADDRESS.clone(),
        name: ALICE_NAME.clone(),
    };
    static ref VOTE: Vote<String> = Vote {
        voter: ALICE_ADDRESS.clone(),
        approval: Some(true),
    };
    static ref VOTES: Vec<Vote<String>> = vec![
        Vote {
            voter: ALICE_ADDRESS.clone(),
            approval: Some(true),
        },
        Vote {
            voter: BOB_ADDRESS.clone(),
            approval: Some(true),
        },
        Vote {
            voter: CHARLIE_ADDRESS.clone(),
            approval: Some(true),
        },
    ];
    static ref ONE_NO_VOTE: Vec<Vote<String>> = vec![
        Vote {
            voter: ALICE_ADDRESS.clone(),
            approval: Some(false),
        },
        Vote {
            voter: BOB_ADDRESS.clone(),
            approval: Some(true),
        },
        Vote {
            voter: CHARLIE_ADDRESS.clone(),
            approval: Some(true),
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

    challenge_app.deploy(CHALLENGE_APP_VERSION.parse()?)?;

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
    )?;

    let _ = account.install_module(
        CHALLENGE_APP_ID,
        &InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: abstr_deployment.ans_host.addr_str()?,
            },
            module: AppInstantiateMsg {
                native_asset: AssetEntry::new("denom"),
                forfeit_amount: Uint128::new(42),
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
    Ok((mock, account, abstr_deployment, deployed))
}

#[test]
fn successful_install() -> anyhow::Result<()> {
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
    apps.challenge_app.create_challenge(CHALLENGE.clone())?;

    let challenge_query = QueryMsg::from(ChallengeQueryMsg::Challenge { challenge_id: 1 });

    let created = apps
        .challenge_app
        .query::<ChallengeResponse>(&challenge_query)?;

    assert_eq!(created.challenge.unwrap(), CHALLENGE.clone());
    Ok(())
}

#[test]
fn test_should_update_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE.clone())?;
    let query = QueryMsg::from(ChallengeQueryMsg::Challenge { challenge_id: 1 });

    apps.challenge_app.query::<ChallengeResponse>(&query)?;

    let to_update = ChallengeEntryUpdate {
        name: Some("update-test".to_string()),
        collateral: Some(Penalty::FixedAmount {
            asset: OfferAsset::new("denom", Uint128::new(100)),
        }),
        description: Some("Updated Test Challenge".to_string()),
    };

    apps.challenge_app.update_challenge(to_update.clone(), 1)?;

    let res = apps.challenge_app.query::<ChallengeResponse>(&query)?;

    assert_eq!(
        res.challenge.as_ref().unwrap().name,
        to_update.name.unwrap()
    );
    assert_eq!(
        res.challenge.as_ref().unwrap().collateral,
        to_update.collateral.unwrap()
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

    apps.challenge_app.create_challenge(CHALLENGE.clone())?;
    let query = QueryMsg::from(ChallengeQueryMsg::Challenge { challenge_id: 1 });

    apps.challenge_app.query::<ChallengeResponse>(&query)?;

    apps.challenge_app.cancel_challenge(1)?;

    let res = apps.challenge_app.query::<ChallengeResponse>(&query)?;

    assert_eq!(res.challenge, None);
    Ok(())
}

#[test]
fn test_should_add_friend_for_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE.clone())?;

    apps.challenge_app
        .query::<ChallengeResponse>(&QueryMsg::from(ChallengeQueryMsg::Challenge {
            challenge_id: 1,
        }))?;

    apps.challenge_app.update_friends_for_challenge(
        1,
        vec![FRIEND.clone()],
        UpdateFriendsOpKind::Add,
    )?;

    let response = apps
        .challenge_app
        .query::<FriendsResponse>(&QueryMsg::from(ChallengeQueryMsg::Friends {
            challenge_id: 1,
        }))?;

    if let Some(friends) = &response.0 {
        for friend in friends.iter() {
            assert_eq!(friend.address, Addr::unchecked(ALICE_ADDRESS.clone()));
            assert_eq!(friend.name, ALICE_NAME.clone());
        }
    }
    Ok(())
}

#[test]
fn test_should_add_friends_for_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE.clone())?;

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

    if let Some(response) = response.0 {
        assert_eq!(
            response,
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
    } else {
        panic!("Friends not found");
    }

    Ok(())
}

#[test]
fn test_should_remove_friend_from_challenge() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE.clone())?;

    let created = apps
        .challenge_app
        .query::<ChallengeResponse>(&QueryMsg::from(ChallengeQueryMsg::Challenge {
            challenge_id: 1,
        }))?;

    assert_eq!(created.challenge.unwrap(), CHALLENGE.clone());

    // add friend
    apps.challenge_app.update_friends_for_challenge(
        1,
        vec![FRIEND.clone()],
        UpdateFriendsOpKind::Add,
    )?;

    let friend_query = QueryMsg::from(ChallengeQueryMsg::Friends { challenge_id: 1 });

    let response = apps.challenge_app.query::<FriendsResponse>(&friend_query)?;

    if let Some(friends) = &response.0 {
        for friend in friends.iter() {
            assert_eq!(friend.address, Addr::unchecked(ALICE_ADDRESS.clone()));
            assert_eq!(friend.name, ALICE_NAME.clone());
        }
    }

    // remove friend
    apps.challenge_app.update_friends_for_challenge(
        1,
        vec![FRIEND.clone()],
        UpdateFriendsOpKind::Remove,
    )?;

    let response = apps.challenge_app.query::<FriendsResponse>(&friend_query)?;
    if let Some(friends) = &response.0 {
        assert_eq!(friends.len(), 0);
    }

    Ok(())
}

#[test]
fn test_should_update_daily_check_in() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;

    apps.challenge_app.create_challenge(CHALLENGE.clone())?;
    let metadata = Some("some_metadata".to_string());
    apps.challenge_app.daily_check_in(1, metadata.clone())?;

    let checked_in = apps
        .challenge_app
        .query::<CheckInResponse>(&QueryMsg::from(ChallengeQueryMsg::CheckIn {
            challenge_id: 1,
        }))?;

    assert_eq!(checked_in.check_in.unwrap().metadata, metadata);
    Ok(())
}

#[test]
fn test_should_cast_vote() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps) = setup()?;

    apps.challenge_app.create_challenge(CHALLENGE.clone())?;
    apps.challenge_app.cast_vote(1, VOTE.clone())?;

    let response =
        apps.challenge_app
            .query::<VotesResponse>(&QueryMsg::from(ChallengeQueryMsg::Votes {
                challenge_id: 1,
            }))?;

    if let Some(votes) = &response.0 {
        for vote in votes.iter() {
            assert_eq!(vote.voter, Addr::unchecked(ALICE_ADDRESS.clone()));
            assert_eq!(vote.approval, Some(true));
        }
    }
    Ok(())
}

#[test]
fn test_should_not_charge_penalty_for_truthy_votes() -> anyhow::Result<()> {
    let (mock, account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE.clone())?;
    apps.challenge_app.update_friends_for_challenge(
        1,
        FRIENDS.clone(),
        UpdateFriendsOpKind::Add,
    )?;

    println!("updated friends");

    for vote in VOTES.clone() {
        apps.challenge_app.cast_vote(1, vote)?;
    }

    let votes =
        apps.challenge_app
            .query::<VotesResponse>(&QueryMsg::from(ChallengeQueryMsg::Votes {
                challenge_id: 1,
            }))?;

    if let Some(votes) = &votes.0 {
        for vote in votes.iter() {
            assert_eq!(vote.approval, Some(true));
        }
    }

    apps.challenge_app.count_votes(1)?;

    let balance = mock.query_balance(&account.proxy.address()?, DENOM)?;
    // if no one voted false, no penalty should be charged, so balance will be 50_000_000
    assert_eq!(balance, Uint128::new(50_000_000));
    Ok(())
}

#[test]
fn test_should_charge_penalty_for_false_votes() -> anyhow::Result<()> {
    let (mock, account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE.clone())?;
    apps.challenge_app.update_friends_for_challenge(
        1,
        FRIENDS.clone(),
        UpdateFriendsOpKind::Add,
    )?;

    for vote in ONE_NO_VOTE.clone() {
        apps.challenge_app.cast_vote(1, vote)?;
    }

    let votes =
        apps.challenge_app
            .query::<VotesResponse>(&QueryMsg::from(ChallengeQueryMsg::Votes {
                challenge_id: 1,
            }))?;

    // Only Alice voted false the rest voted true
    if let Some(votes) = &votes.0 {
        for vote in votes.iter() {
            if vote.voter == Addr::unchecked(ALICE_ADDRESS.clone()) {
                assert_eq!(vote.approval, Some(false));
            } else {
                assert_eq!(vote.approval, Some(true));
            }
        }
    }

    apps.challenge_app.count_votes(1)?;

    let balance = mock.query_balance(&account.proxy.address()?, DENOM)?;
    assert_eq!(balance, Uint128::new(49999901));
    Ok(())
}

#[test]
fn test_should_allow_admin_to_veto_vote() -> anyhow::Result<()> {
    let (mock, account, _abstr, apps) = setup()?;
    apps.challenge_app.create_challenge(CHALLENGE.clone())?;
    apps.challenge_app.update_friends_for_challenge(
        1,
        FRIENDS.clone(),
        UpdateFriendsOpKind::Add,
    )?;

    for vote in ONE_NO_VOTE.clone() {
        apps.challenge_app.cast_vote(1, vote)?;
    }

    let votes =
        apps.challenge_app
            .query::<VotesResponse>(&QueryMsg::from(ChallengeQueryMsg::Votes {
                challenge_id: 1,
            }))?;

    // Only Alice voted false the rest voted true
    if let Some(votes) = &votes.0 {
        for vote in votes.iter() {
            if vote.voter == Addr::unchecked(ALICE_ADDRESS.clone()) {
                assert_eq!(vote.approval, Some(false));
            } else {
                assert_eq!(vote.approval, Some(true));
            }
        }
    }

    let challenge_id = 1;
    apps.challenge_app
        .veto_vote(challenge_id, ALICE_ADDRESS.clone())?;

    apps.challenge_app.count_votes(1)?;
    let balance = mock.query_balance(&account.proxy.address()?, DENOM)?;
    // The false vote was vetoed by the admin, so no penalty should be charged, so balance will be 50_000_000
    assert_eq!(balance, Uint128::new(50_000_000));
    Ok(())
}
