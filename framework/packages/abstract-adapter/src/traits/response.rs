use abstract_sdk::features::{
    CustomData, CustomEvents, DepsAccess, DepsMutAccess, ExecutionStack, HasExecutableEnv,
};

use crate::{state::ContractError, AdapterContract};

/// The execution stack for the app contract
/// TODO : We need to make sure that for queries, this is not used
impl<
        'app,
        T: DepsMutAccess,
        Error: ContractError,
        CustomInitMsg: 'static,
        CustomExecMsg: 'static,
        CustomQueryMsg: 'static,
        ReceiveMsg: 'static,
        SudoMsg: 'static,
    > ExecutionStack
    for AdapterContract<
        'app,
        T,
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    fn stack_mut(&mut self) -> &mut abstract_sdk::features::Executables {
        todo!()
    }
}
/// The events for all executable environments
impl<
        'app,
        T: DepsAccess,
        Error: ContractError,
        CustomInitMsg: 'static,
        CustomExecMsg: 'static,
        CustomQueryMsg: 'static,
        ReceiveMsg: 'static,
        SudoMsg: 'static,
    > CustomEvents
    for AdapterContract<
        'app,
        T,
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    fn add_event<A: Into<cosmwasm_std::Attribute>>(
        &mut self,
        event_name: &str,
        attributes: impl IntoIterator<Item = A>,
    ) {
        todo!()
    }

    fn events(&self) -> Vec<cosmwasm_std::Event> {
        todo!()
    }

    fn add_attributes<A: Into<cosmwasm_std::Attribute>>(
        &mut self,
        attributes: impl IntoIterator<Item = A>,
    ) {
        todo!()
    }

    fn attributes(&self) -> Vec<cosmwasm_std::Attribute> {
        todo!()
    }
}
impl<
        'app,
        T: DepsAccess,
        Error: ContractError,
        CustomInitMsg: 'static,
        CustomExecMsg: 'static,
        CustomQueryMsg: 'static,
        ReceiveMsg: 'static,
        SudoMsg: 'static,
    > CustomData
    for AdapterContract<
        'app,
        T,
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    fn data(&self) -> Option<cosmwasm_std::Binary> {
        todo!()
    }

    fn set_data(&mut self, data: impl Into<cosmwasm_std::Binary>) {
        todo!()
    }
}
