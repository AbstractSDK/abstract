use abstract_app::abstract_testing::OWNER;
use abstract_app::objects::namespace::Namespace;

use abstract_client::AbstractClient;
use abstract_client::Application;

use app::{
    contract::APP_ID,
    error::AppError,
    msg::{AppInstantiateMsg, ConfigResponse, CountResponse},
    *,
};
use cw_controllers::AdminError;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};

use cosmwasm_std::{coins, Addr};

/// Set up the test environment with an Account that has the App installed
#[allow(clippy::type_complexity)]
fn setup(
    count: i32,
) -> anyhow::Result<(AbstractClient<Mock>, Application<Mock, AppInterface<Mock>>)> {
    // Create a sender
    let sender = Addr::unchecked(OWNER);
    let namespace = Namespace::from_id(APP_ID)?;

    // You can set up Abstract with a builder.
    let client = AbstractClient::builder(Mock::new(&sender)).build()?;
    // The client supports setting balances for addresses and configuring ANS.
    client.set_balance(&sender, &coins(123, "ucosm"))?;

    // Build a Publisher Account
    let publisher = client.publisher_builder(namespace).build()?;

    publisher.publish_app::<AppInterface<_>>()?;

    let app = publisher
        .account()
        .install_app::<AppInterface<_>>(&AppInstantiateMsg { count }, &[])?;

    Ok((client, app))
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    let (_, app) = setup(0)?;

    let config = app.config()?;
    assert_eq!(config, ConfigResponse {});
    Ok(())
}

#[test]
fn successful_increment() -> anyhow::Result<()> {
    let (_, app) = setup(0)?;

    app.increment()?;
    let count: CountResponse = app.count()?;
    assert_eq!(count.count, 1);
    Ok(())
}

#[test]
fn successful_reset() -> anyhow::Result<()> {
    let (_, app) = setup(0)?;

    app.reset(42)?;
    let count: CountResponse = app.count()?;
    assert_eq!(count.count, 42);
    Ok(())
}

#[test]
fn failed_reset() -> anyhow::Result<()> {
    let (_, app) = setup(0)?;

    let err: AppError = app
        .call_as(&Addr::unchecked("NotAdmin"))
        .reset(9)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, AppError::Admin(AdminError::NotAdmin {}));
    Ok(())
}
