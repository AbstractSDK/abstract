use abstract_app::objects::namespace::Namespace;
use abstract_app::objects::AccountId;

use abstract_app::std::ABSTRACT_EVENT_TYPE;
use abstract_client::{
    AbstractClient, AbstractInterchainClient, Application, Environment, RemoteAccount,
};

use abstract_app::std::objects::account::AccountTrace;
use abstract_app::std::objects::TruncatedChainId;

use cw_orch::{anyhow, prelude::*};
use cw_orch_interchain::prelude::*;

use ping_pong::contract::APP_ID;
use ping_pong::msg::{AppInstantiateMsg, AppQueryMsg, GameStatusResponse};
use ping_pong::{AppExecuteMsgFns, AppInterface, AppQueryMsgFns};

const JUNO: &str = "juno-1";
const STARGAZE: &str = "stargaze-1";

#[allow(unused)]
struct PingPong<Env: IbcQueryHandler, IbcEnv: InterchainEnv<Env>> {
    abs_juno: AbstractClient<Env>,
    abs_stargaze: AbstractClient<Env>,
    app: Application<Env, AppInterface<Env>>,
    remote_account: RemoteAccount<Env, IbcEnv>,
    mock_interchain: IbcEnv,
}

impl PingPong<MockBech32, MockBech32InterchainEnv> {
    /// Set up the test environment with two Accounts that has the App installed
    fn setup() -> anyhow::Result<PingPong<MockBech32, MockBech32InterchainEnv>> {
        // Create a sender and mock env
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        let interchain_abstract = AbstractInterchainClient::deploy_on(&mock_interchain)?;

        let abs_juno = interchain_abstract.client(JUNO)?;
        let abs_stargaze = interchain_abstract.client(STARGAZE)?;

        let namespace = Namespace::from_id(APP_ID)?;
        // Publish and install on both chains
        let publisher_juno = abs_juno
            .fetch_or_build_account(namespace.clone(), |builder| {
                builder.namespace(namespace.clone())
            })?
            .publisher()?;
        publisher_juno.publish_app::<AppInterface<_>>()?;
        let app = publisher_juno
            .account()
            .install_app_with_dependencies::<AppInterface<_>>(
                &AppInstantiateMsg {},
                Empty {},
                &[],
            )?;

        let publisher_stargaze = abs_stargaze
            .fetch_or_build_account(namespace.clone(), |builder| builder.namespace(namespace))?
            .publisher()?;
        publisher_stargaze.publish_app::<AppInterface<_>>()?;

        let remote_account = app
            .account()
            .remote_account_builder(mock_interchain.clone(), &abs_stargaze)
            .install_app_with_dependencies::<AppInterface<Daemon>>(&AppInstantiateMsg {}, Empty {})?
            .build()?;
        Ok(PingPong {
            abs_juno,
            abs_stargaze,
            app,
            remote_account,
            mock_interchain,
        })
    }
}

pub(crate) fn set_to_win(chain: MockBech32) {
    let mut i = chain.block_info().unwrap();
    if i.height % 2 == 1 {
        i.height += 1;
        chain.app.borrow_mut().set_block(i);
    }
}

pub(crate) fn set_to_lose(chain: MockBech32) {
    let mut i = chain.block_info().unwrap();
    if i.height % 2 == 0 {
        i.height += 1;
        chain.app.borrow_mut().set_block(i);
    }
}

pub fn logger_test_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    logger_test_init();

    let env = PingPong::setup()?;
    let app1 = env.app;

    let mock_stargaze = env.abs_stargaze.environment();

    let game_status = app1.game_status()?;
    assert_eq!(game_status, GameStatusResponse { wins: 0, losses: 0 });

    let module_addrs = env
        .remote_account
        .module_addresses(vec![APP_ID.to_owned()])?;
    let wins: GameStatusResponse = mock_stargaze.query(
        &ping_pong::msg::QueryMsg::from(AppQueryMsg::GameStatus {}),
        &module_addrs.modules[0].1,
    )?;
    assert_eq!(wins, GameStatusResponse { wins: 0, losses: 0 });

    let app2 = env.remote_account.application::<AppInterface<_>>()?;

    let wins: GameStatusResponse = app2.game_status()?;

    assert_eq!(wins, GameStatusResponse { wins: 0, losses: 0 });
    Ok(())
}

#[test]
fn successful_ping_pong() -> anyhow::Result<()> {
    logger_test_init();

    // Create a sender and mock env
    let env = PingPong::setup()?;
    let app = env.app;
    let remote_app = env.remote_account.application::<AppInterface<_>>()?;

    // Ensure account created
    env.abs_stargaze.registry().account(AccountId::new(
        0,
        AccountTrace::Remote(vec![TruncatedChainId::from_chain_id(JUNO)]),
    )?)?;

    let game_status = app.game_status()?;
    assert_eq!(game_status, GameStatusResponse { losses: 0, wins: 0 });
    let game_status = remote_app.game_status()?;
    assert_eq!(game_status, GameStatusResponse { losses: 0, wins: 0 });

    // let stargaze win
    set_to_win(env.mock_interchain.get_chain(STARGAZE)?);
    set_to_lose(env.mock_interchain.get_chain(JUNO)?);

    // juno plays against stargaze
    let pp = app.ping_pong(TruncatedChainId::from_chain_id(STARGAZE))?;
    env.mock_interchain.await_and_check_packets(JUNO, pp)?;

    // stargaze wins, juno lost.
    let game_status = app.game_status()?;
    assert_eq!(game_status, GameStatusResponse { losses: 1, wins: 0 });

    // now let juno win
    set_to_lose(env.mock_interchain.get_chain(STARGAZE)?);
    set_to_win(env.mock_interchain.get_chain(JUNO)?);

    let pp = app.ping_pong(TruncatedChainId::from_chain_id(STARGAZE))?;
    env.mock_interchain.await_and_check_packets(JUNO, pp)?;

    let game_status = app.game_status()?;
    assert_eq!(game_status.wins, 1);
    assert_eq!(game_status.losses, 1);

    let wins: GameStatusResponse = remote_app.game_status()?;
    assert_eq!(wins.losses, 1);
    Ok(())
}

#[test]
fn successful_ping_pong_to_home_chain() -> anyhow::Result<()> {
    logger_test_init();

    // Create a sender and mock env
    let env = PingPong::setup()?;
    let app = env.app;
    let remote_app = env.remote_account.application::<AppInterface<_>>()?;

    // let stargaze win
    set_to_win(env.mock_interchain.get_chain(STARGAZE)?);
    set_to_lose(env.mock_interchain.get_chain(JUNO)?);

    // stargaze plays against stargaze
    // Note that `RemoteApplication` takes care of waiting for ibc
    remote_app.execute(
        &ping_pong::msg::AppExecuteMsg::PingPong {
            opponent_chain: TruncatedChainId::from_chain_id(JUNO),
        }
        .into(),
        vec![],
    )?;

    // stargaze wins, juno lost.
    let game_status = remote_app.game_status()?;
    assert_eq!(game_status, GameStatusResponse { losses: 0, wins: 1 });

    // now let juno win
    set_to_lose(env.mock_interchain.get_chain(STARGAZE)?);
    set_to_win(env.mock_interchain.get_chain(JUNO)?);

    remote_app.execute(
        &ping_pong::msg::AppExecuteMsg::PingPong {
            opponent_chain: TruncatedChainId::from_chain_id(JUNO),
        }
        .into(),
        vec![],
    )?;

    // juno won, stargaze lost.
    let game_status = remote_app.game_status()?;
    assert_eq!(game_status, GameStatusResponse { losses: 1, wins: 1 });

    let game_status = app.game_status()?;
    assert_eq!(game_status.losses, 1);

    Ok(())
}

#[test]
fn query_and_maybe_ping_pong() -> anyhow::Result<()> {
    // Create a sender and mock env
    let env = PingPong::setup()?;
    let app = env.app;

    // Set stargaze to win
    set_to_win(env.mock_interchain.get_chain(STARGAZE)?);
    set_to_lose(env.mock_interchain.get_chain(JUNO)?);

    let pp = app.query_and_maybe_ping_pong(TruncatedChainId::from_chain_id(STARGAZE))?;
    let response = env.mock_interchain.await_packets(JUNO, pp)?;
    let parsed_response = response.assert()?;

    // juno should query and not play, check events
    let abstract_action_events = parsed_response.event_attr_values(ABSTRACT_EVENT_TYPE, "action");
    assert!(abstract_action_events.contains(&String::from("dont_play")));

    // Check stats didn't change in any way
    let game_status = app.game_status()?;
    assert_eq!(game_status, GameStatusResponse { wins: 0, losses: 0 });

    // Set juno to win
    set_to_win(env.mock_interchain.get_chain(JUNO)?);
    set_to_lose(env.mock_interchain.get_chain(STARGAZE)?);

    let pp = app.query_and_maybe_ping_pong(TruncatedChainId::from_chain_id(STARGAZE))?;
    let response = env.mock_interchain.await_packets(JUNO, pp)?;
    let parsed_response = response.assert()?;

    // juno should query and play, check events
    let abstract_action_events = parsed_response.event_attr_values(ABSTRACT_EVENT_TYPE, "action");
    assert!(abstract_action_events.contains(&String::from("ping_pong")));

    // juno won as expected
    let game_status = app.game_status()?;
    assert_eq!(game_status, GameStatusResponse { wins: 1, losses: 0 });

    Ok(())
}
