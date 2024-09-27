use abstract_app::{
    abstract_interface::*,
    std::objects::{
        gov_type::GovernanceDetails,
        module::{ModuleInfo, ModuleVersion},
        voting::{ProposalInfo, ProposalOutcome, ProposalStatus, Threshold, Vote, VoteConfig},
        AssetEntry,
    },
};
use challenge_app::{
    contract::{CHALLENGE_APP_ID, CHALLENGE_APP_VERSION},
    error::AppError,
    msg::{
        ChallengeEntryResponse, ChallengeInstantiateMsg, ChallengeQueryMsg, ChallengeRequest,
        ChallengeResponse, ChallengesResponse, Friend, FriendByAddr, FriendsResponse,
        ProposalsResponse, VoteResponse,
    },
    state::{AdminStrikes, ChallengeEntryUpdate, StrikeStrategy, UpdateFriendsOpKind},
    *,
};
use cosmwasm_std::{coin, Uint128, Uint64};
use cw_asset::AssetInfo;
use cw_orch::{anyhow, prelude::*};

use crate::msg::QueryMsg;

const DENOM: &str = "TOKEN";
const FIRST_CHALLENGE_ID: u64 = 1;

const INITIAL_BALANCE: u128 = 50_000_000;

fn alice_address(mock: &MockBech32) -> Addr {
    mock.addr_make("alice")
}

fn bob_address(mock: &MockBech32) -> Addr {
    mock.addr_make("bob")
}

fn charlie_address(mock: &MockBech32) -> Addr {
    mock.addr_make("charlie")
}

fn alice_friend(mock: &MockBech32) -> Friend<String> {
    Friend::Addr(FriendByAddr {
        address: alice_address(mock).to_string(),
        name: "alice_name".to_string(),
    })
}

fn bob_friend(mock: &MockBech32) -> Friend<String> {
    Friend::Addr(FriendByAddr {
        address: bob_address(mock).to_string(),
        name: "bob_name".to_string(),
    })
}

fn charlie_friend(mock: &MockBech32) -> Friend<String> {
    Friend::Addr(FriendByAddr {
        address: charlie_address(mock).to_string(),
        name: "charlie_name".to_string(),
    })
}

fn friends(mock: &MockBech32) -> Vec<Friend<String>> {
    vec![alice_friend(mock), bob_friend(mock), charlie_friend(mock)]
}

fn unchecked_friends(mock: &MockBech32) -> Vec<Friend<Addr>> {
    friends(mock)
        .clone()
        .into_iter()
        .map(|f| match f {
            Friend::Addr(FriendByAddr { address, name }) => Friend::Addr(FriendByAddr {
                address: Addr::unchecked(address),
                name,
            }),
            Friend::AbstractAccount(account_id) => Friend::AbstractAccount(account_id),
        })
        .collect()
}

fn challenge_req_func(mock: &MockBech32) -> ChallengeRequest {
    ChallengeRequest {
        name: "test".to_string(),
        strike_asset: AssetEntry::new("denom"),
        strike_strategy: StrikeStrategy::Split(Uint128::new(30_000_000)),
        description: Some("Test Challenge".to_string()),
        challenge_duration_seconds: Uint64::new(10_000),
        proposal_duration_seconds: Uint64::new(1_000),
        strikes_limit: None,
        init_friends: friends(mock).clone(),
    }
}

#[allow(unused)]
struct DeployedApps {
    challenge_app: Challenge<MockBech32>,
}

#[allow(clippy::type_complexity)]
fn setup() -> anyhow::Result<(
    MockBech32,
    AccountI<MockBech32>,
    Abstract<MockBech32>,
    DeployedApps,
)> {
    // Create the mock
    let mut mock = MockBech32::new("mock");
    let sender = Abstract::mock_admin(&mock);
    mock.set_sender(sender.clone());
    mock.set_balance(&sender, vec![coin(INITIAL_BALANCE, DENOM)])?;

    let mut challenge_app = Challenge::new(CHALLENGE_APP_ID, mock.clone());
    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), sender.clone())?;

    challenge_app.deploy(CHALLENGE_APP_VERSION.parse()?, DeployStrategy::Try)?;

    let _module_info = ModuleInfo::from_id(
        CHALLENGE_APP_ID,
        ModuleVersion::Version(CHALLENGE_APP_VERSION.to_string()),
    )?;

    abstr_deployment.ans_host.execute(
        &abstract_app::std::ans_host::ExecuteMsg::UpdateAssetAddresses {
            to_add: vec![("denom".to_owned(), AssetInfo::native(DENOM).into())],
            to_remove: vec![],
        },
        &[],
    )?;

    let account_details: AccountDetails = AccountDetails {
        name: "test".to_string(),
        description: None,
        link: None,
        namespace: None,
        install_modules: vec![],
        account_id: None,
    };

    let account = AccountI::create(
        &abstr_deployment,
        account_details,
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        &[],
    )?;

    let _ = account.install_app(
        &challenge_app,
        &ChallengeInstantiateMsg {
            vote_config: VoteConfig {
                threshold: Threshold::Majority {},
                veto_duration_seconds: None,
            },
        },
        &[],
    )?;

    challenge_app.set_sender(&account.address()?);
    mock.set_balance(
        &account.address()?,
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
    let (mock, _account, _abstr, apps) = setup()?;
    let challenge_req = challenge_req_func(&mock).clone();
    apps.challenge_app
        .create_challenge(challenge_req_func(&mock).clone())?;

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
        description: challenge_req.description.unwrap(),
        end_timestamp: mock.block_info()?.time.plus_seconds(10_000),
        proposal_duration_seconds: Uint64::new(1_000),
        admin_strikes: AdminStrikes {
            num_strikes: 0,
            limit: challenge_req.strikes_limit.unwrap_or(1),
        },
        active_proposal: None,
    };
    assert_eq!(created_challenge, expected_response);
    Ok(())
}

#[test]
fn test_update_challenge() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app
        .create_challenge(challenge_req_func(&mock).clone())?;

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
    let (mock, _account, _abstr, apps) = setup()?;

    // Challenge without active proposals
    apps.challenge_app
        .create_challenge(challenge_req_func(&mock).clone())?;
    apps.challenge_app.cancel_challenge(FIRST_CHALLENGE_ID)?;

    let res: ChallengeResponse =
        apps.challenge_app
            .query(&QueryMsg::from(ChallengeQueryMsg::Challenge {
                challenge_id: FIRST_CHALLENGE_ID,
            }))?;
    let challenge = res.challenge.unwrap();

    assert_eq!(challenge.end_timestamp, mock.block_info()?.time);

    // Challenge with active proposal
    apps.challenge_app
        .create_challenge(challenge_req_func(&mock).clone())?;
    apps.challenge_app
        .call_as(&alice_address(&mock))
        .cast_vote(
            FIRST_CHALLENGE_ID + 1,
            Vote {
                vote: true,
                memo: None,
            },
        )?;

    apps.challenge_app
        .cancel_challenge(FIRST_CHALLENGE_ID + 1)?;

    let res: ChallengeResponse =
        apps.challenge_app
            .query(&QueryMsg::from(ChallengeQueryMsg::Challenge {
                challenge_id: FIRST_CHALLENGE_ID + 1,
            }))?;
    let challenge = res.challenge.unwrap();
    assert_eq!(challenge.end_timestamp, mock.block_info()?.time);
    let proposals: ProposalsResponse =
        apps.challenge_app
            .proposals(FIRST_CHALLENGE_ID + 1, None, None)?;

    assert_eq!(
        proposals.proposals[0].1.status,
        ProposalStatus::Finished(ProposalOutcome::Canceled)
    );
    Ok(())
}

#[test]
fn test_add_single_friend_for_challenge() -> anyhow::Result<()> {
    let (mock, _account, abstr, apps) = setup()?;
    apps.challenge_app
        .create_challenge(challenge_req_func(&mock).clone())?;

    let new_account = AccountI::create_default_account(
        &abstr,
        GovernanceDetails::Monarchy {
            monarch: mock.sender_addr().to_string(),
        },
    )?;
    let new_friend: Friend<String> = Friend::AbstractAccount(new_account.id()?);

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
    let friends = response.friends;

    let mut expected_friends: Vec<Friend<Addr>> = unchecked_friends(&mock).clone();
    expected_friends.push(Friend::AbstractAccount(new_account.id()?));

    assert_eq!(friends, expected_friends);

    Ok(())
}

#[test]
fn test_add_friends_for_challenge() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;
    let challenge_req_without_friends = ChallengeRequest {
        init_friends: vec![],
        ..challenge_req_func(&mock).clone()
    };

    apps.challenge_app
        .create_challenge(challenge_req_without_friends)?;

    apps.challenge_app.update_friends_for_challenge(
        FIRST_CHALLENGE_ID,
        friends(&mock).clone(),
        UpdateFriendsOpKind::Add {},
    )?;

    let response: FriendsResponse =
        apps.challenge_app
            .query(&QueryMsg::from(ChallengeQueryMsg::Friends {
                challenge_id: FIRST_CHALLENGE_ID,
            }))?;
    let friends = response.friends;

    let expected_friends: Vec<Friend<Addr>> = unchecked_friends(&mock).clone();

    assert_eq!(friends, expected_friends);

    Ok(())
}

#[test]
fn test_remove_friend_from_challenge() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app
        .create_challenge(challenge_req_func(&mock).clone())?;

    // remove friend
    apps.challenge_app.update_friends_for_challenge(
        FIRST_CHALLENGE_ID,
        vec![alice_friend(&mock).clone()],
        UpdateFriendsOpKind::Remove {},
    )?;

    let friends_query = QueryMsg::from(ChallengeQueryMsg::Friends {
        challenge_id: FIRST_CHALLENGE_ID,
    });

    let response: FriendsResponse = apps.challenge_app.query(&friends_query)?;
    let friends = response.friends;

    let mut expected_friends = unchecked_friends(&mock).clone();
    expected_friends.retain(|s| match s {
        Friend::Addr(addr) => addr.address != alice_address(&mock).clone(),
        Friend::AbstractAccount(_) => todo!(),
    });

    assert_eq!(friends, expected_friends);

    Ok(())
}

#[test]
fn test_cast_vote() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;

    let vote = Vote {
        vote: true,
        memo: Some("some memo".to_owned()),
    };
    apps.challenge_app
        .create_challenge(challenge_req_func(&mock).clone())?;
    apps.challenge_app
        .call_as(&alice_address(&mock))
        .cast_vote(FIRST_CHALLENGE_ID, vote.clone())?;

    let response: VoteResponse =
        apps.challenge_app
            .query(&QueryMsg::from(ChallengeQueryMsg::Vote {
                voter_addr: alice_address(&mock).to_string(),
                challenge_id: FIRST_CHALLENGE_ID,
                proposal_id: None,
            }))?;

    assert_eq!(response.vote, Some(vote));
    Ok(())
}

#[test]
fn test_update_friends_during_proposal() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;
    apps.challenge_app
        .create_challenge(challenge_req_func(&mock).clone())?;

    // start proposal
    apps.challenge_app
        .call_as(&alice_address(&mock).clone())
        .cast_vote(
            FIRST_CHALLENGE_ID,
            Vote {
                vote: true,
                memo: None,
            },
        )?;

    let err: AppError = apps
        .challenge_app
        .update_friends_for_challenge(
            FIRST_CHALLENGE_ID,
            vec![alice_friend(&mock).clone()],
            UpdateFriendsOpKind::Remove {},
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        AppError::FriendsEditDuringProposal(mock.block_info()?.time.plus_seconds(1_000))
    );

    Ok(())
}

#[test]
fn test_not_charge_penalty_for_voting_false() -> anyhow::Result<()> {
    let (mock, account, _abstr, apps) = setup()?;
    apps.challenge_app
        .create_challenge(challenge_req_func(&mock).clone())?;

    // cast votes
    let votes = vec![
        (
            alice_address(&mock).clone(),
            Vote {
                vote: false,
                memo: None,
            },
        ),
        (
            bob_address(&mock).clone(),
            Vote {
                vote: false,
                memo: None,
            },
        ),
        (
            charlie_address(&mock).clone(),
            Vote {
                vote: false,
                memo: None,
            },
        ),
    ];
    run_challenge_vote_sequence(&mock, &apps, votes)?;

    let prev_votes_results = apps
        .challenge_app
        .proposals(FIRST_CHALLENGE_ID, None, None)?
        .proposals;
    let expected_end = mock.block_info()?.time;
    assert_eq!(
        prev_votes_results,
        vec![(
            1,
            ProposalInfo {
                total_voters: 3,
                votes_for: 0,
                votes_against: 3,
                status: ProposalStatus::Finished(ProposalOutcome::Failed),
                config: VoteConfig {
                    threshold: Threshold::Majority {},
                    veto_duration_seconds: None,
                },
                end_timestamp: expected_end,
            }
        )]
    );

    let balance = mock.query_balance(&account.address()?, DENOM)?;
    // if no one voted true, no penalty should be charged, so balance will be 50_000_000
    assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
    Ok(())
}

#[test]
fn test_charge_penalty_for_voting_true() -> anyhow::Result<()> {
    let (mock, account, _abstr, apps) = setup()?;
    apps.challenge_app
        .create_challenge(challenge_req_func(&mock).clone())?;

    let votes = vec![
        (
            alice_address(&mock).clone(),
            Vote {
                vote: true,
                memo: None,
            },
        ),
        (
            bob_address(&mock).clone(),
            Vote {
                vote: true,
                memo: None,
            },
        ),
        (
            charlie_address(&mock).clone(),
            Vote {
                vote: true,
                memo: None,
            },
        ),
    ];
    run_challenge_vote_sequence(&mock, &apps, votes)?;

    let balance = mock.query_balance(&account.address()?, DENOM)?;
    // Initial balance - strike
    assert_eq!(balance, Uint128::new(INITIAL_BALANCE - 30_000_000));
    Ok(())
}

#[test]
fn test_query_challenges_within_range() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;
    for _ in 0..10 {
        apps.challenge_app
            .create_challenge(challenge_req_func(&mock).clone())?;
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
fn test_query_challenges_within_different_range() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;
    for _ in 0..10 {
        apps.challenge_app
            .create_challenge(challenge_req_func(&mock).clone())?;
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

#[test]
fn test_vetoed() -> anyhow::Result<()> {
    let (mock, account, _abstr, apps) = setup()?;
    apps.challenge_app.update_config(VoteConfig {
        threshold: Threshold::Majority {},
        veto_duration_seconds: Some(Uint64::new(1_000)),
    })?;
    apps.challenge_app
        .create_challenge(challenge_req_func(&mock).clone())?;

    let votes = vec![
        (
            alice_address(&mock).clone(),
            Vote {
                vote: true,
                memo: None,
            },
        ),
        (
            bob_address(&mock).clone(),
            Vote {
                vote: true,
                memo: None,
            },
        ),
        (
            charlie_address(&mock).clone(),
            Vote {
                vote: true,
                memo: None,
            },
        ),
    ];
    for (signer, vote) in votes {
        apps.challenge_app
            .call_as(&signer)
            .cast_vote(FIRST_CHALLENGE_ID, vote)?;
    }
    mock.wait_seconds(1_000)?;
    apps.challenge_app.veto(FIRST_CHALLENGE_ID)?;
    let prev_proposals: ProposalsResponse =
        apps.challenge_app
            .proposals(FIRST_CHALLENGE_ID, None, None)?;
    let status = prev_proposals.proposals[0].1.status.clone();
    assert_eq!(status, ProposalStatus::Finished(ProposalOutcome::Vetoed));

    // balance unchanged
    let balance = mock.query_balance(&account.address()?, DENOM)?;
    assert_eq!(balance, Uint128::new(INITIAL_BALANCE));
    Ok(())
}

#[test]
fn test_veto_expired() -> anyhow::Result<()> {
    let (mock, account, _abstr, apps) = setup()?;
    apps.challenge_app.update_config(VoteConfig {
        threshold: Threshold::Majority {},
        veto_duration_seconds: Some(Uint64::new(1_000)),
    })?;
    apps.challenge_app
        .create_challenge(challenge_req_func(&mock).clone())?;

    let votes = vec![
        (
            alice_address(&mock).clone(),
            Vote {
                vote: true,
                memo: None,
            },
        ),
        (
            bob_address(&mock).clone(),
            Vote {
                vote: true,
                memo: None,
            },
        ),
        (
            charlie_address(&mock).clone(),
            Vote {
                vote: true,
                memo: None,
            },
        ),
    ];
    for (signer, vote) in votes {
        apps.challenge_app
            .call_as(&signer)
            .cast_vote(FIRST_CHALLENGE_ID, vote)?;
    }

    // wait time to expire veto
    mock.wait_seconds(2_000)?;
    apps.challenge_app
        .call_as(&alice_address(&mock).clone())
        .count_votes(FIRST_CHALLENGE_ID)?;

    let proposals: ProposalsResponse =
        apps.challenge_app
            .proposals(FIRST_CHALLENGE_ID, None, None)?;

    assert_eq!(
        proposals.proposals[0].1.status,
        ProposalStatus::Finished(ProposalOutcome::Passed)
    );

    // balance updated
    let balance = mock.query_balance(&account.address()?, DENOM)?;
    assert_eq!(balance, Uint128::new(INITIAL_BALANCE - 30_000_000));
    Ok(())
}

#[test]
fn test_duplicate_friends() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;
    // Duplicate initial friends
    let err: error::AppError = apps
        .challenge_app
        .create_challenge(ChallengeRequest {
            init_friends: vec![alice_friend(&mock).clone(), alice_friend(&mock).clone()],
            ..challenge_req_func(&mock).clone()
        })
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, error::AppError::DuplicateFriends {});

    // Add duplicate (Alice already exists)
    apps.challenge_app
        .create_challenge(challenge_req_func(&mock).clone())?;
    let err: error::AppError = apps
        .challenge_app
        .update_friends_for_challenge(
            FIRST_CHALLENGE_ID,
            vec![alice_friend(&mock).clone()],
            UpdateFriendsOpKind::Add {},
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, error::AppError::DuplicateFriends {});
    Ok(())
}

fn run_challenge_vote_sequence(
    mock: &MockBech32,
    apps: &DeployedApps,
    votes: Vec<(Addr, Vote)>,
) -> anyhow::Result<()> {
    for (signer, vote) in votes {
        apps.challenge_app
            .call_as(&signer)
            .cast_vote(FIRST_CHALLENGE_ID, vote)?;
    }
    mock.wait_seconds(1_000)?;
    apps.challenge_app.count_votes(FIRST_CHALLENGE_ID)?;
    Ok(())
}
