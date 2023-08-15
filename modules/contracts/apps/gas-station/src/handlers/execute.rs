#![allow(clippy::too_many_arguments)]

use cosmwasm_std::{Coin, coins, CosmosMsg, DepsMut, Env, from_binary, MessageInfo, ReplyOn, Response, SubMsg};
use cw_asset::AssetInfoBase;
use osmosis_std::types::cosmos::bank::v1beta1::BankQuerier;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgCreateDenom;

use abstract_core::objects::AnsAsset;
use abstract_sdk::features::{AbstractNameService, AccountIdentification};
use abstract_sdk::{
    AbstractResponse, BasicAllowance, Execution, GrantInterface, Resolve, TokenFactoryInterface,
};
use cosmos_sdk_proto::traits::Message;

use crate::contract::{GasStationResult, GasStationApp};
use crate::error::GasStationError;
use crate::msg::GasStationExecuteMsg;
use crate::replies::CREATE_DENOM_REPLY_ID;
use crate::state::{GasPump, GAS_PUMPS, PENDING_PUMP};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: GasStationApp,
    msg: GasStationExecuteMsg,
) -> GasStationResult {
    match msg {
        GasStationExecuteMsg::CreateGasPump { grade, fuel_mix } => {
            create_gas_pump(deps, env, info, app, grade, fuel_mix)
        }
        GasStationExecuteMsg::DispenseGas { grade, recipient } => {
            dispense_gas(deps, env, info, app, grade, recipient)
        }
        _ => panic!(),
    }
}

/// Dispense a new gas token grade to the recipient.
/// This mints a new token, and grants the recipient a basic allowance for the fuel mix.
fn dispense_gas(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: GasStationApp,
    grade: String,
    recipient: String,
) -> GasStationResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let recipient = deps.api.addr_validate(&recipient)?;

    let pump = GAS_PUMPS.load(deps.storage, grade.clone())?;

    // check if recipient already has token?? or actually just re-up grant

    let allowance_msg = app.grant().basic(&app.proxy_address(deps.as_ref())?, &recipient, BasicAllowance {
        spend_limit: pump
            .fuel_mix
            .into_iter()
            .map(|asset| Coin {
                amount: asset.amount,
                denom: match asset.info {
                    AssetInfoBase::Native(denom) => denom,
                    _ => panic!(),
                },
            })
            .collect(),
        // TODO - allow for expiration
        expiration: None,
    })?;

    let allowance_msg = app.executor(deps.as_ref()).execute(vec![allowance_msg])?;

    Ok(app.custom_tag_response(Response::new().add_message(allowance_msg), "dispense_gas", vec![("recipient", recipient.as_str())]))
}

fn create_gas_pump(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: GasStationApp,
    grade: String,
    fuel_mix: Vec<AnsAsset>,
) -> GasStationResult {
    // Ensure the caller is an admin
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let ans = app.ans_host(deps.as_ref())?;

    let fuel_mix = fuel_mix.resolve(&deps.querier, &ans)?;

    // iterate and assert each native variant
    for asset in fuel_mix.iter() {
        match asset.info {
            AssetInfoBase::Native(_) => {}
            _ => return Err(GasStationError::OnlyNativeTokensCanBeUsedAsGas {}),
        }
    }

    // Check if the pump already exists
    if GAS_PUMPS.may_load(deps.storage, grade.clone())?.is_some() {
        return Err(GasStationError::GasPumpAlreadyExists {});
    }

    // Format the gas grade denom
    let grade_subdenom = format!("gaspump{}", grade);

    // Create or retrieve the token factory for the given gas grade
    let factory = app.token_factory(deps.as_ref(), &grade_subdenom, None)?;
    // let factory = app.token_factory(deps.as_ref(), &grade_subdenom, Some(env.contract.address.clone()))?;
    let denom = factory.denom()?;

    // Check if the denom already exists in the bank
    let bank_querier = BankQuerier::new(&deps.querier);
    if bank_querier.denom_metadata(denom.clone()).is_ok() {
        return Err(GasStationError::DenomAlreadyExists(denom));
    }

    // Create the grade denom (TODO: consider combining this in the token factory)
    let action = factory.create_denom()?;

    // let action = app.grant().basic(&env.contract.address, &app.proxy_address(deps.as_ref())?, BasicAllowance {
    //     spend_limit: coins(1000000000u128, "uosmo"),
    //     // TODO - allow for expiration
    //     expiration: None,
    // })?;

    let create_denom_msg = action.clone().messages().swap_remove(0);
    // The below message is wrapped in moduleAction...
    // let test_msg = app.executor(deps.as_ref()).execute(vec![action.clone()])?;
    //
    // // codespace: wasm, code: 5
    let denom_msg = app.executor(deps.as_ref()).execute_with_reply_and_data(
        create_denom_msg,
        ReplyOn::Always,
        CREATE_DENOM_REPLY_ID,
    )?;
    //
    // // codespace: wasm, code: 10.... now codespace: sdk, code: 4
    // let denom_msg = app.executor(deps.as_ref()).execute_with_reply(
    //     vec![action.clone()],
    //     ReplyOn::Always,
    //     CREATE_DENOM_REPLY_ID,
    // )?;
    // //
    // // let action2 = factory.create_denom()?;
    // //
    // // let action_msg = action2.messages().swap_remove(0);
    // // let value: MsgCreateDenom = match action_msg {
    // //     CosmosMsg::Stargate { value, .. } => {
    // //         let coded: Vec<u8> = from_binary(&value).unwrap();
    // //         let decoded: MsgCreateDenom = MsgCreateDenom::decode(&coded[..]).unwrap();
    // //         decoded
    // //     },
    // //     _ => panic!(),
    // // };
    // //
    // // panic!("Action: {:?}, denom_msg: {:?}, value: {:?}", action, denom_msg, value);
    // //

    // Save the pending pump
    PENDING_PUMP.save(deps.storage, &(grade.clone(), GasPump { denom, fuel_mix }))?;

    // Return a response with custom tags
    Ok(app.custom_tag_response(
        // Response::new().add_message(create_denom_msg),
        // Response::new().add_submessage(SubMsg::reply_always(create_denom_msg, CREATE_DENOM_REPLY_ID)),
        Response::new().add_submessage(denom_msg),
        "create_gas_pump",
        vec![("grade", grade)],
    ))
}
