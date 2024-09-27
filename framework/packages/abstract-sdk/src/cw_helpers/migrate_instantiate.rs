use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

/// Instantiate account with migration, used to migration from blob smart contract
pub fn migrate_instantiate<InitMsg, Err>(
    deps: DepsMut,
    env: Env,
    init_msg: InitMsg,
    init_fn: impl Fn(DepsMut, Env, MessageInfo, InitMsg) -> Result<Response, Err>,
) -> Result<Response, Err>
where
    Err: From<cosmwasm_std::StdError>,
{
    if cw2::CONTRACT.exists(deps.storage) {
        return Err(cosmwasm_std::StdError::generic_err(
            "Second instantiation attempt: cw2 is not clear during instantiation",
        )
        .into());
    }
    let contract_info = deps
        .querier
        .query_wasm_contract_info(&env.contract.address)?;
    // Only admin can call migrate on contract
    let sender = contract_info.admin.unwrap();
    let message_info = MessageInfo {
        sender,
        funds: vec![],
    };
    init_fn(deps, env, message_info, init_msg)
}
