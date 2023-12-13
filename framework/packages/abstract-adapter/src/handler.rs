use crate::{state::ContractError, AdapterContract};
use abstract_sdk::{
    base::{AbstractContract, Handler},
    features::{CustomData, CustomEvents, ExecutionStack},
};
use cosmwasm_std::Empty;

impl<Error: ContractError, InitMsg, ExecMsg, QueryMsg, ReceiveMsg, SudoMsg> Handler
    for AdapterContract<Error, InitMsg, ExecMsg, QueryMsg, ReceiveMsg, SudoMsg>
{
    type Error = Error;
    type CustomInitMsg = InitMsg;
    type CustomExecMsg = ExecMsg;
    type CustomQueryMsg = QueryMsg;
    type CustomMigrateMsg = Empty;
    type ReceiveMsg = ReceiveMsg;
    type SudoMsg = SudoMsg;

    fn contract(&self) -> &AbstractContract<Self, Error> {
        &self.contract
    }
}

/// The execution stack for the app contract
impl<
        Error: ContractError,
        CustomInitMsg: 'static,
        CustomExecMsg: 'static,
        CustomQueryMsg: 'static,
        ReceiveMsg: 'static,
        SudoMsg: 'static,
    > ExecutionStack
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
    fn stack_mut(&mut self) -> &mut abstract_sdk::features::Executables {
        &mut self.response.executables
    }
}
/// The events for all executable environments
impl<
        Error: ContractError,
        CustomInitMsg: 'static,
        CustomExecMsg: 'static,
        CustomQueryMsg: 'static,
        ReceiveMsg: 'static,
        SudoMsg: 'static,
    > CustomEvents
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
    fn add_event<A: Into<cosmwasm_std::Attribute>>(
        &mut self,
        event_name: &str,
        attributes: impl IntoIterator<Item = A>,
    ) {
        self.response.add_event(event_name, attributes)
    }

    fn events(&self) -> Vec<cosmwasm_std::Event> {
        self.response.events()
    }

    fn add_attributes<A: Into<cosmwasm_std::Attribute>>(
        &mut self,
        attributes: impl IntoIterator<Item = A>,
    ) {
        self.response.add_attributes(attributes)
    }

    fn attributes(&self) -> Vec<cosmwasm_std::Attribute> {
        self.response.attributes()
    }
}
impl<
        Error: ContractError,
        CustomInitMsg: 'static,
        CustomExecMsg: 'static,
        CustomQueryMsg: 'static,
        ReceiveMsg: 'static,
        SudoMsg: 'static,
    > CustomData
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
    fn data(&self) -> Option<cosmwasm_std::Binary> {
        self.response.data()
    }

    fn set_data(&mut self, data: impl Into<cosmwasm_std::Binary>) {
        self.response.set_data(data)
    }
}
