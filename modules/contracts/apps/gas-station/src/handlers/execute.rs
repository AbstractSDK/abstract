use cosmos_sdk_proto::cosmos::auth::v1beta1::QueryAccountRequest;
use cosmos_sdk_proto::prost::Message;
use std::collections::HashSet;

use cosmwasm_std::{
    coins, Addr, Binary, Coin, DepsMut, Empty, Env, MessageInfo, QueryRequest, Response, Timestamp,
};
use cw_asset::AssetInfoBase;
use osmosis_std::types::cosmos::auth::v1beta1::QueryAccountResponse;

use abstract_core::objects::AnsAsset;
use abstract_sdk::{
    features::AbstractNameService, AbstractResponse, Execution, GrantInterface, Resolve,
    TransferInterface,
};

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
            create_if_missing,
        } => activate_gas_pass(
            deps,
            env,
            info,
            app,
            grade,
            recipient,
            expiration,
            bypass_pass_check,
            create_if_missing,
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
#[allow(clippy::too_many_arguments)]
fn activate_gas_pass(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: GasStationApp,
    grade_name: GradeName,
    recipient: String,
    expiration: Option<Timestamp>,
    bypass_pass_check: Option<bool>,
    create_if_missing: Option<bool>,
) -> GasStationResult {
    let recipient = deps.api.addr_validate(&recipient)?;

    let grade = GRADES.load(deps.storage, grade_name.clone())?;
    let mut account_actions = vec![];

    // query the account and if it errors, send the minimum amount of tokens to it.
    if query_account(&deps, &recipient).is_err() && create_if_missing.is_some_and(|c| c) {
        let denom = grade
            .fuel_mix
            .clone()
            .first()
            .map(|x| x.denom.clone())
            .unwrap();
        // transfer the minimum of the grade fuel mix to the recipient

        account_actions.push(
            app.bank(deps.as_ref())
                .transfer(coins(1, denom), &recipient)?,
        );
    }

    // check if recipient already has token?? or actually just re-up grant
    let allowance_msg = app.fee_granter(deps.as_ref(), None)?.grant_basic_allowance(
        &recipient,
        grade.fuel_mix,
        expiration,
    );
    account_actions.push(allowance_msg.into());

    let account_msg = app.executor(deps.as_ref()).execute(account_actions)?;

    GAS_PASSES.update(deps.storage, &recipient, |x| -> GasStationResult<GasPass> {
        let pass = GasPass {
            grade: grade_name.clone(),
            expiration,
        };
        match x {
            Some(_) => {
                // Allow the user to overwrite their gas pass if they want to.
                if let Some(true) = bypass_pass_check {
                    // todo - if the pass is the same, we need to revoke the old one
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
        &grade_name,
        |x| -> GasStationResult<HashSet<Addr>> {
            let mut set = x.unwrap_or_default();
            set.insert(recipient.clone());
            Ok(set)
        },
    )?;

    Ok(app.custom_tag_response(
        Response::new().add_message(account_msg),
        "activate_gas_pass",
        vec![
            ("recipient", recipient.as_str()),
            ("grade", grade_name.as_str()),
        ],
    ))
}

fn query_account(
    deps: &DepsMut,
    recipient: &Addr,
) -> Result<QueryAccountResponse, GasStationError> {
    let base_account_query_request = QueryAccountRequest {
        address: recipient.to_string(),
    };
    let base_account_query: QueryRequest<Empty> = QueryRequest::Stargate {
        // auth Base AccountQueryAccountResponse
        path: "/cosmos.auth.v1beta1.Query/Account".to_string(),
        data: Binary(base_account_query_request.encode_to_vec()),
    };
    let response: QueryAccountResponse = deps.querier.query(&base_account_query)?;
    Ok(response)
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
        .fee_granter(deps.as_ref(), None)?
        .revoke_allowance(&holder);
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
