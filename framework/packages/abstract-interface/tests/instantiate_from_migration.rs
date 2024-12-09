use abstract_interface::{AnsHost, ModuleFactory, Registry};
use abstract_std::{ANS_HOST, MODULE_FACTORY, REGISTRY};
use cosmwasm_schema::serde::Serialize;
use cw_blob::interface::CwBlob;
use cw_orch::{anyhow, mock::MockBase, prelude::*};

#[cosmwasm_schema::cw_serde]
pub enum MigrateMsg<I> {
    Instantiate(I),
}

pub fn instantiate_from_blob_same_result<T, I, A, S>(
    contract: T,
    instantiate_msg: I,
) -> anyhow::Result<()>
where
    T: ContractInstance<MockBase<A, S>> + Uploadable,
    I: Serialize + std::fmt::Debug,
    A: cosmwasm_std::Api,
    S: StateInterface,
{
    let chain = contract.environment().clone();

    let contract_code_id = contract.upload()?.uploaded_code_id()?;
    let contract_address = chain
        .instantiate(
            contract_code_id,
            &instantiate_msg,
            Some("label"),
            Some(chain.sender()),
            &[],
        )?
        .instantiated_contract_address()?;

    let blob = CwBlob::new("cw-blob", chain.clone());
    let blob_code_id = blob.upload()?.uploaded_code_id()?;
    let blob_address = chain
        .instantiate(
            blob_code_id,
            &Empty {},
            Some("label"),
            Some(chain.sender()),
            &[],
        )?
        .instantiated_contract_address()?;
    chain.migrate(
        &MigrateMsg::Instantiate(instantiate_msg),
        contract_code_id,
        &blob_address,
    )?;

    let dump = chain.app.borrow().dump_wasm_raw(&contract_address);
    let blob_dump = chain.app.borrow().dump_wasm_raw(&blob_address);

    assert_eq!(dump, blob_dump);
    Ok(())
}

#[test]
fn ans_host() {
    let chain = MockBech32::new("mock");
    let contract = AnsHost::new(ANS_HOST, chain.clone());
    instantiate_from_blob_same_result(
        contract,
        abstract_std::ans_host::InstantiateMsg {
            admin: chain.sender().to_string(),
        },
    )
    .unwrap();
}

#[test]
fn registry() {
    let chain = MockBech32::new("mock");
    let contract = Registry::new(REGISTRY, chain.clone());
    instantiate_from_blob_same_result(
        contract,
        abstract_std::registry::InstantiateMsg {
            admin: chain.sender().to_string(),
            security_enabled: None,
            namespace_registration_fee: None,
        },
    )
    .unwrap();
}

#[test]
fn module_factory() {
    let chain = MockBech32::new("mock");
    let contract = ModuleFactory::new(MODULE_FACTORY, chain.clone());
    instantiate_from_blob_same_result(
        contract,
        abstract_std::module_factory::InstantiateMsg {
            admin: chain.sender().to_string(),
        },
    )
    .unwrap();
}
