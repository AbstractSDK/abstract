// Use prelude to get all the necessary imports
use crate::msg::QueryMsg;
use abstract_challenge_app::{
    contract::{AppResult, CHALLENGE_APP_ID, CHALLENGE_APP_VERSION},
    msg::{
        AppInstantiateMsg, ChallengeQueryMsg, ChallengeResponse, ConfigResponse, FriendResponse,
        InstantiateMsg,
    },
    state::{ChallengeEntry, Friend},
    *,
};
use abstract_core::{
    app::BaseInstantiateMsg,
    objects::{
        gov_type::GovernanceDetails,
        module::{ModuleInfo, ModuleVersion},
        namespace::Namespace,
        AssetEntry,
    },
};
use abstract_dex_adapter::msg::OfferAsset;
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, VCExecFns, *};
use cosmwasm_std::{coin, Uint128};
use cw_asset::AssetInfo;
use cw_orch::{anyhow, deploy::Deploy, prelude::*};
use semver::Version;
use wyndex_bundle::{WynDex, EUR, USD, WYNDEX as WYNDEX_WITHOUT_CHAIN};

// consts for testing
const ADMIN: &str = "admin";
const DENOM: &str = "TOKEN";

#[allow(unused)]
struct DeployedApps {
    challenge_app: ChallengeApp<Mock>,
}

#[allow(unused)]
struct CronCatAddrs {
    factory: Addr,
    manager: Addr,
    tasks: Addr,
    agents: Addr,
}

#[allow(clippy::type_complexity)]
fn setup() -> anyhow::Result<(Mock, AbstractAccount<Mock>, Abstract<Mock>, DeployedApps)> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);
    let mut challenge_app = ChallengeApp::new(CHALLENGE_APP_ID, mock.clone());
    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), sender.to_string())?;

    challenge_app.deploy(CHALLENGE_APP_VERSION.parse()?)?;

    let module_info = ModuleInfo::from_id(
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

    let config: ConfigResponse = apps.challenge_app.config()?;
    assert_eq!(
        config,
        ConfigResponse {
            native_asset: AssetEntry::new("denom"),
            forfeit_amount: Uint128::new(42),
        }
    );
    Ok(())
}

#[test]
fn test_should_create_challenge() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;
    let challenge = ChallengeEntry {
        name: "test".to_string(),
        source_asset: OfferAsset::new("denom", Uint128::new(100)),
    };

    apps.challenge_app.create_challenge(challenge.clone())?;

    let challenge_query = QueryMsg::from(ChallengeQueryMsg::Challenge {
        challenge_id: "challenge_1".to_string(),
    });

    let created = apps
        .challenge_app
        .query::<ChallengeResponse>(&challenge_query)?;

    assert_eq!(created.challenge.unwrap(), challenge);
    Ok(())
}

#[test]
fn test_should_update_challenge() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;
    let challenge = ChallengeEntry {
        name: "test".to_string(),
        source_asset: OfferAsset::new("denom", Uint128::new(100)),
    };

    let created = apps.challenge_app.create_challenge(challenge.clone())?;
    let query = QueryMsg::from(ChallengeQueryMsg::Challenge {
        challenge_id: "challenge_1".to_string(),
    });

    let created = apps.challenge_app.query::<ChallengeResponse>(&query)?;

    let to_update = ChallengeEntry {
        name: "update-test".to_string(),
        source_asset: OfferAsset::new("denom", Uint128::new(100)),
    };

    apps.challenge_app
        .update_challenge(to_update.clone(), "challenge_1".to_string())?;

    let res = apps.challenge_app.query::<ChallengeResponse>(&query)?;

    assert_eq!(res.challenge.unwrap(), to_update);
    Ok(())
}

#[test]
fn test_should_cancel_challenge() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;
    let challenge = ChallengeEntry {
        name: "test".to_string(),
        source_asset: OfferAsset::new("denom", Uint128::new(100)),
    };

    let created = apps.challenge_app.create_challenge(challenge.clone())?;
    let query = QueryMsg::from(ChallengeQueryMsg::Challenge {
        challenge_id: "challenge_1".to_string(),
    });

    let created = apps.challenge_app.query::<ChallengeResponse>(&query)?;

    apps.challenge_app
        .cancel_challenge("challenge_1".to_string())?;

    let res = apps.challenge_app.query::<ChallengeResponse>(&query)?;

    assert_eq!(res.challenge, None);
    Ok(())
}

#[test]
fn test_should_add_friend_for_challenge() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;
    let challenge = ChallengeEntry {
        name: "test".to_string(),
        source_asset: OfferAsset::new("denom", Uint128::new(100)),
    };

    let created = apps.challenge_app.create_challenge(challenge.clone())?;

    let created = apps
        .challenge_app
        .query::<ChallengeResponse>(&QueryMsg::from(ChallengeQueryMsg::Challenge {
            challenge_id: "challenge_1".to_string(),
        }))?;

    apps.challenge_app.add_friend_for_challenge(
        "challenge_1".to_string(),
        "0x123".to_string(),
        "Alice".to_string(),
    )?;

    let added =
        apps.challenge_app
            .query::<FriendResponse>(&QueryMsg::from(ChallengeQueryMsg::Friend {
                challenge_id: "challenge_1".to_string(),
                friend_address: "0x123".to_string(),
            }))?;

    assert_eq!(
        added.friend.unwrap(),
        Friend {
            address: "0x123".to_string(),
            name: "Alice".to_string(),
        }
    );
    Ok(())
}

#[test]
fn test_should_add_friends_for_challenge() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;
    let challenge = ChallengeEntry {
        name: "test".to_string(),
        source_asset: OfferAsset::new("denom", Uint128::new(100)),
    };

    let created = apps.challenge_app.create_challenge(challenge.clone())?;

    let created = apps
        .challenge_app
        .query::<ChallengeResponse>(&QueryMsg::from(ChallengeQueryMsg::Challenge {
            challenge_id: "challenge_1".to_string(),
        }))?;

    let friends = vec![
        Friend {
            address: "0x123".to_string(),
            name: "Alice".to_string(),
        },
        Friend {
            address: "0x456".to_string(),
            name: "Bob".to_string(),
        },
        Friend {
            address: "0x789".to_string(),
            name: "Charlie".to_string(),
        },
    ];

    apps.challenge_app
        .add_friends_for_challenge("challenge_1".to_string(), friends.clone())?;

    for friend in friends {
        let added = apps.challenge_app.query::<FriendResponse>(&QueryMsg::from(
            ChallengeQueryMsg::Friend {
                challenge_id: "challenge_1".to_string(),
                friend_address: friend.address.clone(),
            },
        ))?;

        assert_eq!(added.friend.unwrap(), friend);
    }
    Ok(())
}

#[test]
fn test_should_remove_friend_from_challenge() -> anyhow::Result<()> {
    let (mock, _account, _abstr, apps) = setup()?;
    let challenge = ChallengeEntry {
        name: "test".to_string(),
        source_asset: OfferAsset::new("denom", Uint128::new(100)),
    };

    let created = apps.challenge_app.create_challenge(challenge.clone())?;

    let created = apps
        .challenge_app
        .query::<ChallengeResponse>(&QueryMsg::from(ChallengeQueryMsg::Challenge {
            challenge_id: "challenge_1".to_string(),
        }))?;

    apps.challenge_app.add_friend_for_challenge(
        "challenge_1".to_string(),
        "0x123".to_string(),
        "Alice".to_string(),
    )?;

    let friend_query = QueryMsg::from(ChallengeQueryMsg::Friend {
        challenge_id: "challenge_1".to_string(),
        friend_address: "0x123".to_string(),
    });

    let added = apps.challenge_app.query::<FriendResponse>(&friend_query)?;

    assert_eq!(
        added.friend.unwrap(),
        Friend {
            address: "0x123".to_string(),
            name: "Alice".to_string(),
        }
    );

    apps.challenge_app
        .remove_friend_for_challenge("challenge_1".to_string(), "0x123".to_string())?;

    let removed = apps.challenge_app.query::<FriendResponse>(&friend_query)?;
    assert_eq!(removed.friend, None);

    Ok(())
}
