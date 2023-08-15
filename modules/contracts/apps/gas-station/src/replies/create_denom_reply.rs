use cosmwasm_std::{ensure_eq, DepsMut, Env, Reply, Response};
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgCreateDenomResponse;

use abstract_sdk::{AbstractResponse, Execution, TokenFactoryInterface};

use crate::contract::{AppResult, GasStationApp};
use crate::error::AppError;
use crate::state::{GAS_PUMPS, PENDING_PUMP};

pub fn create_denom_reply(deps: DepsMut, env: Env, app: GasStationApp, reply: Reply) -> AppResult {
    // TODO: how can we do something with the SDK to make this easier?
    // I want to save the denom automatically.
    // Can I setup the reply handler using the app?
    let MsgCreateDenomResponse { new_token_denom } = reply.result.try_into()?;
    let (grade, pump) = PENDING_PUMP.load(deps.storage)?;
    ensure_eq!(
        pump.denom,
        new_token_denom,
        AppError::PendingGasPumpDoesNotMatchCreatedGasPump {
            pending: grade.clone(),
            created: new_token_denom.clone()
        }
    );

    // set beforesend listener to this contract
    // this will trigger sudo endpoint before any bank send
    // which makes blacklisting / freezing possible
    let factory = app.token_factory(deps.as_ref(), &grade, None);
    let before_send_hook = factory.set_before_send_hook(env.contract.address)?;
    let before_send_hook = app
        .executor(deps.as_ref())
        .execute(vec![before_send_hook])?;

    GAS_PUMPS.save(deps.storage, grade.clone(), &pump)?;

    Ok(app.custom_tag_response(
        Response::new().add_message(before_send_hook),
        "create_denom",
        vec![("grade", "grade")],
    ))
}
