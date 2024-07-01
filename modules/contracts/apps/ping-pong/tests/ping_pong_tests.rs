use abstract_app::abstract_interface::VCQueryFns;
use abstract_app::objects::namespace::Namespace;
use abstract_app::objects::AccountId;

use abstract_client::{AbstractClient, Application, Environment, RemoteAccount};

use abstract_app::std::objects::account::AccountTrace;
use abstract_app::std::objects::chain_name::ChainName;

use cw_orch::{anyhow, prelude::*};
use cw_orch_interchain::prelude::*;

use ping_pong::contract::APP_ID;
use ping_pong::msg::{AppInstantiateMsg, AppQueryMsg, GameStatusResponse};
use ping_pong::{AppExecuteMsgFns, AppInterface, AppQueryMsgFns};

const JUNO: &str = "juno-1";
const STARGAZE: &str = "stargaze-1";

#[allow(unused)]
struct PingPong<'a, Env: IbcQueryHandler, IbcEnv: InterchainEnv<Env>> {
    abs_juno: AbstractClient<Env>,
    abs_stargaze: AbstractClient<Env>,
    app: Application<Env, AppInterface<Env>>,
    remote_account: RemoteAccount<'a, Env, IbcEnv>,
}

impl<'a> PingPong<'a, MockBech32, MockBech32InterchainEnv> {
    /// Set up the test environment with two Accounts that has the App installed
    fn setup(
        mock_interchain: &'a MockBech32InterchainEnv,
    ) -> anyhow::Result<PingPong<'a, MockBech32, MockBech32InterchainEnv>> {
        let mock_juno = mock_interchain.chain(JUNO).unwrap();
        let mock_stargaze = mock_interchain.chain(STARGAZE).unwrap();

        let abs_juno = AbstractClient::builder(mock_juno.clone()).build()?;
        let abs_stargaze = AbstractClient::builder(mock_stargaze.clone()).build()?;

        abs_juno.connect_to(&abs_stargaze, mock_interchain)?;

        let namespace = Namespace::from_id(APP_ID)?;
        // Publish and install on both chains
        let publisher_juno = abs_juno.publisher_builder(namespace.clone()).build()?;
        publisher_juno.publish_app::<AppInterface<_>>()?;
        let app = publisher_juno
            .account()
            .install_app_with_dependencies::<AppInterface<_>>(
                &AppInstantiateMsg {},
                Empty {},
                &[],
            )?;

        let publisher_stargaze = abs_stargaze.publisher_builder(namespace).build()?;
        publisher_stargaze.publish_app::<AppInterface<_>>()?;

        publisher_juno.account().set_ibc_status(true)?;
        let remote_account = app
            .account()
            .remote_account_builder(mock_interchain, &abs_stargaze)
            .install_app_with_dependencies::<AppInterface<Daemon>>(&AppInstantiateMsg {}, Empty {})?
            .build()?;

        Ok(PingPong {
            abs_juno,
            abs_stargaze,
            app,
            remote_account,
        })
    }
}

pub(crate) fn set_to_win(chain: MockBech32) {
    let mut i = chain.block_info().unwrap();
    if i.height % 2 == 0 {
        ()
    } else {
        i.height += 1;
        chain.app.borrow_mut().set_block(i);
    }
}

pub(crate) fn set_to_lose(chain: MockBech32) {
    let mut i = chain.block_info().unwrap();
    if i.height % 2 == 1 {
        ()
    } else {
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

    // Create a sender and mock env
    let mock_interchain =
        MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);
    let env = PingPong::setup(&mock_interchain)?;
    let app1 = env.app;

    let mock_stargaze = env.abs_stargaze.environment();

    let wins = app1.game_status()?;
    assert_eq!(wins, GameStatusResponse { wins: 0, losses: 0 });

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
    let mock_interchain =
        MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);
    let env = PingPong::setup(&mock_interchain)?;
    let app = env.app;
    let remote_app = env.remote_account.application::<AppInterface<_>>()?;

    // Ensure account created
    env.abs_stargaze
        .version_control()
        .account_base(AccountId::new(
            1,
            AccountTrace::Remote(vec![ChainName::from_chain_id(JUNO)]),
        )?)?;

    let wins = app.game_status()?;
    assert_eq!(wins, GameStatusResponse { losses: 0, wins: 0 });
    let wins = remote_app.game_status()?;
    assert_eq!(wins, GameStatusResponse { losses: 0, wins: 0 });

    // let stargaze win
    set_to_win(mock_interchain.chain(STARGAZE)?);
    set_to_lose(mock_interchain.chain(JUNO)?);

    // juno plays against stargaze
    let pp = app.ping_pong(ChainName::from_chain_id(STARGAZE))?;
    mock_interchain.check_ibc(JUNO, pp)?.into_result()?;

    // stargaze wins, juno lost.
    let wins = app.game_status()?;
    assert_eq!(wins, GameStatusResponse { losses: 1, wins: 0 });

    // now let juno win
    set_to_lose(mock_interchain.chain(STARGAZE)?);
    set_to_win(mock_interchain.chain(JUNO)?);

    let pp = app.ping_pong(ChainName::from_chain_id(STARGAZE))?;
    mock_interchain.check_ibc(JUNO, pp)?.into_result()?;

    let wins = app.game_status()?;
    assert_eq!(wins.wins, 1);
    assert_eq!(wins.losses, 1);

    let wins: GameStatusResponse = remote_app.game_status()?;
    assert_eq!(wins.losses, 1);
    Ok(())
}

#[test]
fn successful_ping_pong_to_home_chain() -> anyhow::Result<()> {
    logger_test_init();

    // Create a sender and mock env
    let mock_interchain =
        MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);
    let env = PingPong::setup(&mock_interchain)?;
    let app = env.app;
    let remote_app = env.remote_account.application::<AppInterface<_>>()?;

    // let stargaze win
    set_to_win(mock_interchain.chain(STARGAZE)?);
    set_to_lose(mock_interchain.chain(JUNO)?);

    // stargaze plays against stargaze
    let pp = remote_app.execute(
        &ping_pong::msg::AppExecuteMsg::PingPong {
            opponent_chain: ChainName::from_chain_id(STARGAZE),
        }
        .into(),
    )?;

    // stargaze wins, juno lost.
    let wins = remote_app.game_status()?;
    assert_eq!(wins, GameStatusResponse { losses: 0, wins: 1 });

    // // now let juno win
    // set_to_lose(mock_interchain.chain(STARGAZE)?);
    // set_to_win(mock_interchain.chain(JUNO)?);

    // let pp = app.ping_pong(ChainName::from_chain_id(STARGAZE))?;
    // mock_interchain.check_ibc(JUNO, pp)?.into_result()?;

    // let wins = app.game_status()?;
    // assert_eq!(wins.wins, 1);
    // assert_eq!(wins.losses, 1);

    // let wins: GameStatusResponse = remote_app.game_status()?;
    // assert_eq!(wins.losses, 1);
    Ok(())
}

// #[test]
// fn successful_remote_ping_pong() -> anyhow::Result<()> {
//     logger_test_init();

//     // Create a sender and mock env
//     let mock_interchain =
//         MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);
//     let env = PingPong::setup(&mock_interchain)?;
//     let app = env.remote_account.application::<AppInterface<_>>()?;

//     let remote_ping_pong_response = app.execute(
//         &ping_pong::msg::AppExecuteMsg::PingPong {
//             opponent_chain: ChainName::from_chain_id(JUNO),
//         }
//         .into(),
//     )?;

//     let wins_left_events = remote_ping_pong_response
//         .events()
//         .into_iter()
//         .map(|ev| {
//             ev.attributes
//                 .into_iter()
//                 .filter(|attr| attr.key == "wins_left")
//                 .collect::<Vec<_>>()
//         })
//         .flatten()
//         .collect::<Vec<_>>();
//     assert_eq!(
//         wins_left_events,
//         vec![
//             Attribute::new("wins_left", "4"),
//             Attribute::new("wins_left", "3"),
//             Attribute::new("wins_left", "2"),
//             Attribute::new("wins_left", "1")
//         ]
//     );

//     let ping_ponged = remote_ping_pong_response
//         .events()
//         .into_iter()
//         .find_map(|ev| {
//             ev.attributes
//                 .iter()
//                 .find(|attr| attr.value == "ping_ponged")
//                 .cloned()
//         });
//     assert!(ping_ponged.is_some());

//     let wins: GameStatusResponse = app.query(&ping_pong::msg::AppQueryMsg::Pongs {}.into())?;
//     assert_eq!(wins.wins, 0);
//     let previous_ping_pong: GameStatusResponse =
//         app.game_status()?;
//     assert_eq!(
//         previous_ping_pong,
//         GameStatusResponse {
//             wins: 0
//         }
//     );

//     Ok(())
// }

// #[test]
// fn rematch() -> anyhow::Result<()> {
//     logger_test_init();

//     // Create a sender and mock env
//     let mock_interchain =
//         MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);
//     let env = PingPong::setup(&mock_interchain)?;
//     let app = env.app;
//     let remote_app = env.remote_account.application::<AppInterface<_>>()?;

//     remote_app.execute(
//         &ping_pong::msg::AppExecuteMsg::PingPong {
//             opponent_chain: ChainName::from_chain_id(JUNO),
//         }
//         .into(),
//     )?;

//     let rematch = app.rematch(env.remote_account.id(), ChainName::from_chain_id(STARGAZE))?;
//     let ibc_wait_response = mock_interchain.wait_ibc(JUNO, rematch)?;
//     ibc_wait_response.into_result()?;

//     let wins_left_events = ibc_wait_response
//         .events()
//         .into_iter()
//         .map(|ev| {
//             ev.attributes
//                 .into_iter()
//                 .filter(|attr| attr.key == "wins_left")
//                 .collect::<Vec<_>>()
//         })
//         .flatten()
//         .collect::<Vec<_>>();
//     assert_eq!(
//         wins_left_events,
//         vec![
//             Attribute::new("wins_left", "4"),
//             Attribute::new("wins_left", "3"),
//             Attribute::new("wins_left", "2"),
//             Attribute::new("wins_left", "1")
//         ]
//     );

//     let ping_ponged = ibc_wait_response.events().into_iter().find_map(|ev| {
//         ev.attributes
//             .iter()
//             .find(|attr| attr.value == "ping_ponged")
//             .cloned()
//     });
//     assert!(ping_ponged.is_some());

//     let wins = app.game_status()?;
//     assert_eq!(wins.wins, 0);
//     let previous_ping_pong = app.game_status()?;
//     assert_eq!(
//         previous_ping_pong,
//         GameStatusResponse {
//             wins
//         }
//     );

//     // Remote
//     let wins: GameStatusResponse = remote_app.query(&ping_pong::msg::AppQueryMsg::GameStatus {  } {}.into())?;
//     assert_eq!(wins.wins, 0);
//     let previous_ping_pong: GameStatusResponse =
//         remote_app.query(&ping_pong::msg::AppQueryMsg::GameStatus {}.into())?;
//     assert_eq!(
//         previous_ping_pong,
//         GameStatusResponse {
//             wins: 0
//         }
//     );

//     Ok(())
// }
