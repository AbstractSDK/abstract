#![allow(clippy::too_many_arguments)]

use cosmwasm_std::{Coin, DepsMut, Env, MessageInfo, ReplyOn, Response};
use cw_asset::AssetInfoBase;
use osmosis_std::types::cosmos::bank::v1beta1::BankQuerier;

use abstract_core::objects::AnsAsset;
use abstract_sdk::{AbstractResponse, BasicAllowance, Execution, GrantInterface, Resolve, TokenFactoryInterface};
use abstract_sdk::features::AbstractNameService;

use crate::contract::{AppResult, GasStationApp};
use crate::error::AppError;
use crate::msg::GasStationExecuteMsg;
use crate::replies::CREATE_DENOM_REPLY_ID;
use crate::state::{GAS_PUMPS, GasPump, PENDING_PUMP};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: GasStationApp,
    msg: GasStationExecuteMsg,
) -> AppResult {
    match msg {
        GasStationExecuteMsg::CreateGasPump {
            grade,
            fuel_mix
        } => create_gas_pump(
            deps,
            env,
            info,
            app,
            grade,
            fuel_mix),
        GasStationExecuteMsg::DispenseGas {
            grade,
            recipient,
        } => dispense_gas(
            deps,
            env,
            info,
            app,
            grade,
            recipient),
        _ => panic!()
    }
}

fn dispense_gas(deps: DepsMut, env: Env, info: MessageInfo, app: GasStationApp, grade: String, recipient: String) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let recipient = deps.api.addr_validate(&recipient)?;

    let pump = GAS_PUMPS.load(deps.storage, grade.clone())?;

    // check if recipient already has token?? or actually just re-up grant

    let allowance_msg = app.grant().allow_basic(BasicAllowance {
        spend_limit: pump.fuel_mix.into_iter().map(|asset| Coin {
            amount: asset.amount,
            denom: match asset.info {
                AssetInfoBase::Native(denom) => denom,
                _ => panic!()
            }
        }).collect(),
        // TODO
        expiration: None,
    })?;

    let allowance_msg = app.executor(deps.as_ref()).execute(vec![allowance_msg])?;


    Ok(app.tag_response(
        Response::new()
            .add_message(allowance_msg),
        "dispense_gas"
    ))
}

fn create_gas_pump(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: GasStationApp,
    grade: String,
    fuel_mix: Vec<AnsAsset>
) -> AppResult {
    // Ensure the caller is an admin
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let ans = app.ans_host(deps.as_ref())?;

    let fuel_mix = fuel_mix.resolve(&deps.querier, &ans)?;

    // iterate and assert each native variant
    for asset in fuel_mix {
        match asset.info {
            AssetInfoBase::Native(_) => {}
            _ => return Err(AppError::OnlyNativeTokensCanBeUsedAsGas {})
        }
    }

    // Check if the pump already exists
    if GAS_PUMPS.may_load(deps.storage, grade.clone())?.is_some() {
        return Err(AppError::GasPumpAlreadyExists {});
    }

    // Format the gas grade denom
    let grade_subdenom = format!("{}_gas_pump", grade);

    // Create or retrieve the token factory for the given gas grade
    let factory = app.token_factory(deps.as_ref(), &grade_subdenom, None);
    let denom = factory.denom()?;

    // Check if the denom already exists in the bank
    let bank_querier = BankQuerier::new(&deps.querier);
    if bank_querier.denom_metadata(denom.clone()).is_ok() {
        return Err(AppError::DenomAlreadyExists(denom));
    }

    // Create the grade denom (TODO: consider combining this in the token factory)
    let action = factory.create_denom()?;
    let denom_msg = app.executor(deps.as_ref())
        .execute_with_reply(vec![action], ReplyOn::Always, CREATE_DENOM_REPLY_ID)?;

    // Save the pending pump
    PENDING_PUMP.save(deps.storage, &(grade, GasPump { denom, fuel_mix }))?;

    // Return a response with custom tags
    Ok(app.custom_tag_response(
        Response::new().add_submessage(denom_msg),
        "create_gas_pump",
        vec![("grade", grade)]
    ))
}

