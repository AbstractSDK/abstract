use std::collections::HashSet;

use cosmwasm_std::{Addr, Coin, DepsMut, Env, MessageInfo, Response, Timestamp};
use cw_asset::AssetInfoBase;

use abstract_core::objects::AnsAsset;
use abstract_sdk::{features::AbstractNameService, AbstractResponse, Execution, GrantInterface, Resolve, TransferInterface};

use crate::state::{GasPass, GradeName};
use crate::{
    contract::{GasStationApp, GasStationResult},
    error::GasStationError,
    msg::GasStationExecuteMsg,
    state::{Grade, GAS_PASSES, GRADES, GRADE_TO_USERS},
};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: GasStationApp,
    msg: GasStationExecuteMsg,
) -> GasStationResult {
    // Ensure the caller is an admin
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    match msg {
        GasStationExecuteMsg::CreateGrade { grade, fuel_mix } => {
            create_grade(deps, env, info, app, grade, fuel_mix)
        }
        GasStationExecuteMsg::ActivateGasPass {
            grade,
            recipient,
            expiration,
            bypass_pass_check,
        } => activate_gas_pass(
            deps,
            env,
            info,
            app,
            grade,
            recipient,
            expiration,
            bypass_pass_check,
        ),
        GasStationExecuteMsg::DeactivateGasPass { holder } => {
            deactivate_gas_pass(deps, env, info, app, holder)
        }
    }
}

fn create_grade(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: GasStationApp,
    grade: GradeName,
    fuel_mix: Vec<AnsAsset>,
) -> GasStationResult {
    let ans = app.ans_host(deps.as_ref())?;

    let resolved_mix = fuel_mix.resolve(&deps.querier, &ans)?;

    // iterate and assert each native variant
    for asset in resolved_mix.iter() {
        match asset.info {
            AssetInfoBase::Native(_) => {}
            _ => return Err(GasStationError::OnlyNativeTokensCanBeUsedAsGas {}),
        }
    }

    // Save the new grade
    GRADES.update(
        deps.storage,
        grade.clone(),
        |x| -> GasStationResult<Grade> {
            match x {
                Some(_) => Err(GasStationError::GradeAlreadyExists(grade.clone())),
                None => Ok(Grade {
                    fuel_mix: resolved_mix
                        .into_iter()
                        .map(|asset| Coin {
                            amount: asset.amount,
                            denom: match asset.info {
                                AssetInfoBase::Native(denom) => denom,
                                _ => panic!(),
                            },
                        })
                        .collect(),
                }),
            }
        },
    )?;

    // Return a response with custom tags
    Ok(app.custom_tag_response(Response::new(), "create_grade", vec![("grade", grade)]))
}

/// Dispense a new gas token grade to the recipient.
/// This mints a new token, and grants the recipient a basic allowance for the fuel mix.
fn activate_gas_pass(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: GasStationApp,
    grade: GradeName,
    recipient: String,
    expiration: Option<Timestamp>,
    bypass_pass_check: bool,
) -> GasStationResult {
    let recipient = deps.api.addr_validate(&recipient)?;

    let pump = GRADES.load(deps.storage, grade.clone())?;

    // check if recipient already has token?? or actually just re-up grant
    let allowance_msg = app.fee_granter(deps.as_ref(), None).grant_basic_allowance(
        &recipient,
        pump.fuel_mix,
        expiration,
    )?;

    let allowance_msg = app
        .executor(deps.as_ref())
        .execute(vec![allowance_msg.into()])?;

    GAS_PASSES.update(deps.storage, &recipient, |x| -> GasStationResult<GasPass> {
        let pass = GasPass {
            grade: grade.clone(),
            expiration,
        };
        match x {
            Some(_) => {
                if bypass_pass_check {
                    Ok(pass)
                } else {
                    Err(GasStationError::GasPassAlreadyExists(recipient.to_string()))
                }
            }
            None => Ok(pass),
        }
    })?;

    GRADE_TO_USERS.update(
        deps.storage,
        &grade,
        |x| -> GasStationResult<HashSet<Addr>> {
            let mut set = x.unwrap_or_else(|| HashSet::new());
            set.insert(recipient.clone());
            Ok(set)
        },
    )?;

    Ok(app.custom_tag_response(
        Response::new().add_message(allowance_msg),
        "activate_gas_pass",
        vec![("recipient", recipient.as_str()), ("grade", grade.as_str())],
    ))
}

fn deactivate_gas_pass(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: GasStationApp,
    holder: String,
) -> GasStationResult {
    let holder = deps.api.addr_validate(&holder)?;

    // Revoke all allowances
    let allowance_msg = app
        .fee_granter(deps.as_ref(), None)
        .revoke_allowance(&holder)?;
    let allowance_msg = app
        .executor(deps.as_ref())
        .execute(vec![allowance_msg.into()])?;

    // Allow for the case where the user has no grade for traditional authz
    let maybe_grade = GAS_PASSES.may_load(deps.storage, &holder)?;

    if let Some(pass) = maybe_grade {
        GAS_PASSES.remove(deps.storage, &holder);
        GRADE_TO_USERS.update(
            deps.storage,
            &pass.grade,
            |x| -> GasStationResult<HashSet<Addr>> {
                match x {
                    Some(mut set) => {
                        set.remove(&holder);
                        Ok(set)
                    }
                    // We shouldn't get here, but if we do, just return an empty set
                    None => Ok(HashSet::new()),
                }
            },
        )?;
    }

    Ok(app.custom_tag_response(
        Response::new().add_message(allowance_msg),
        "deactivate_gas_pass",
        vec![("holder", holder.as_str())],
    ))
}
