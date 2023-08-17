use crate::{
    contract::HostResult,
    endpoints::reply::INIT_CALLBACK_ID,
    state::{CLIENT_PROXY, CONFIG, REGISTRATION_CACHE},
    HostError,
};
use abstract_core::{account_factory, objects::AccountId};
use abstract_sdk::core::abstract_ica::{IbcQueryResponse, StdAck};
use cosmwasm_std::{
    to_vec, wasm_execute, Binary, ContractResult, Deps, DepsMut, Empty, Env, IbcReceiveResponse,
    QuerierWrapper, QueryRequest, Response, StdError, SubMsg, SystemResult,
};

fn unparsed_query(
    querier: QuerierWrapper<'_, Empty>,
    request: &QueryRequest<Empty>,
) -> Result<Binary, HostError> {
    let raw = to_vec(request)?;
    match querier.raw_query(&raw) {
        SystemResult::Err(system_err) => {
            Err(StdError::generic_err(format!("Querier system error: {system_err}")).into())
        }
        SystemResult::Ok(ContractResult::Err(contract_err)) => {
            Err(StdError::generic_err(format!("Querier contract error: {contract_err}")).into())
        }
        SystemResult::Ok(ContractResult::Ok(value)) => Ok(value),
    }
}

// processes IBC query
pub fn receive_query(
    deps: Deps,
    msgs: Vec<QueryRequest<Empty>>,
) -> Result<IbcReceiveResponse, HostError> {
    let mut results = vec![];

    for query in msgs {
        let res = unparsed_query(deps.querier, &query)?;
        results.push(res);
    }
    let response = IbcQueryResponse { results };

    let acknowledgement = StdAck::success(response);
    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_attribute("action", "receive_ibc_query"))
}

// processes PacketMsg::Register variant
/// Creates and registers proxy for remote Account
#[allow(clippy::too_many_arguments)]
pub fn receive_register(
    deps: DepsMut,
    env: Env,
    account_id: AccountId,
    account_proxy_address: String,
    name: String,
    description: Option<String>,
    link: Option<String>,
) -> HostResult {
    let cfg = CONFIG.load(deps.storage)?;

    // verify that the origin last chain is the chain related to this channel, and that it is not `Local`
    account_id.trace().verify_remote()?;

    // create the message to instantiate the remote account
    let factory_msg = wasm_execute(
        cfg.account_factory,
        &account_factory::ExecuteMsg::CreateAccount {
            governance: abstract_core::objects::gov_type::GovernanceDetails::External {
                governance_address: env.contract.address.into_string(),
                governance_type: "abstract-ibc".into(), // at least 4 characters
            },
            name,
            description,
            link,
            // provide the origin chain id
            origin: Some(account_id.clone()),
        },
        vec![],
    )?;
    // wrap with a submsg
    let factory_msg = SubMsg::reply_on_success(factory_msg, INIT_CALLBACK_ID);

    // store the proxy address of the Account on the client chain.
    CLIENT_PROXY.save(deps.storage, &account_id, &account_proxy_address)?;
    // store the account info for the reply handler
    REGISTRATION_CACHE.save(deps.storage, &account_id.clone())?;

    Ok(Response::new()
        .add_submessage(factory_msg)
        .add_attribute("action", "register"))
}
