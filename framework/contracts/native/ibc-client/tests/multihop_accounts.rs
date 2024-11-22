use abstract_interface::Abstract;
use abstract_std::{
    ibc::polytone_callbacks::CallbackMessage,
    ibc_client::{ExecuteMsgFns as _, IbcClientCallback},
    objects::{AccountId, TruncatedChainId},
    ABSTRACT_EVENT_TYPE,
};
use cosmwasm_std::{to_json_binary, StdResult, SubMsgResponse};
use cw_orch::{core::serde_json, mock::MockBech32, prelude::*, take_storage_snapshot};

type AResult = cw_orch::anyhow::Result<()>; // alias for Result<(), anyhow::Error>

#[test]
fn multihop_account_snapshot() -> AResult {
    let chain = MockBech32::new("mock");
    // Mock note, so it can take execute calls
    let note_code_id = chain
        .upload_custom(
            "note",
            Box::new(ContractWrapper::new(
                |_, _, _, _: serde_json::Value| StdResult::Ok(cosmwasm_std::Response::new()),
                |_, _, _, _: Empty| StdResult::Ok(cosmwasm_std::Response::new()),
                |_,
                 _,
                 _: cosmwasm_std::Empty|
                 -> Result<cosmwasm_std::Binary, cosmwasm_std::Never> {
                    unreachable!()
                },
            )),
        )?
        .uploaded_code_id()?;
    let note = chain
        .instantiate(note_code_id, &Empty {}, Some("note"), None, &[])?
        .instantiated_contract_address()?;

    // Make ibc-client trust our mock note for registering accounts
    let deployment = Abstract::new(chain.clone());
    deployment.ibc.client.upload()?;
    deployment
        .ibc
        .client
        .instantiate(&abstract_std::ibc_client::InstantiateMsg {}, None, &[])?;
    deployment.ibc.client.register_infrastructure(
        TruncatedChainId::from_chain_id("remote-1"),
        "host",
        note.clone(),
    )?;
    deployment
        .ibc
        .client
        .call_as(&note)
        .callback(CallbackMessage {
            initiator: deployment.ibc.client.address()?,
            initiator_msg: to_json_binary(&IbcClientCallback::WhoAmI {})?,
            result: abstract_std::ibc::polytone_callbacks::Callback::Execute(Ok(
                abstract_std::ibc::polytone_callbacks::ExecutionResponse {
                    executed_by: "host".to_owned(),
                    result: vec![],
                },
            )),
        })?;

    // register account
    deployment
        .ibc
        .client
        .call_as(&note)
        .callback(CallbackMessage {
            initiator: deployment.ibc.client.address()?,
            initiator_msg: to_json_binary(&IbcClientCallback::CreateAccount {
                account_id: AccountId::new(
                    42,
                    abstract_std::objects::AccountTrace::Remote(vec![
                        TruncatedChainId::from_chain_id("remote-1"),
                        TruncatedChainId::from_chain_id("remote-2"),
                    ]),
                )?,
            })?,
            result: abstract_std::ibc::polytone_callbacks::Callback::Execute(Ok(
                abstract_std::ibc::polytone_callbacks::ExecutionResponse {
                    executed_by: "host".to_owned(),
                    #[allow(deprecated)]
                    result: vec![SubMsgResponse {
                        events: vec![cosmwasm_std::Event::new(ABSTRACT_EVENT_TYPE)
                            .add_attribute("account_address", "remote_account")],
                        data: None,
                        msg_responses: vec![],
                    }],
                },
            )),
        })?;
    take_storage_snapshot!(chain, "multihop_account");
    Ok(())
}
