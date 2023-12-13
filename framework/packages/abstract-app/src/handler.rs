use crate::{state::ContractError, AbstractContract, AppContract, Handler};

impl<Error: ContractError, InitMsg, ExecMsg, QueryMsg, MigrateMsg, ReceiveMsg, SudoMsg> Handler
    for AppContract<Error, InitMsg, ExecMsg, QueryMsg, MigrateMsg, ReceiveMsg, SudoMsg>
{
    type Error = Error;
    type CustomInitMsg = InitMsg;
    type CustomExecMsg = ExecMsg;
    type CustomQueryMsg = QueryMsg;
    type CustomMigrateMsg = MigrateMsg;
    type ReceiveMsg = ReceiveMsg;
    type SudoMsg = SudoMsg;

    fn contract(&self) -> &AbstractContract<Self, Error> {
        &self.contract
    }
}

use abstract_sdk::features::{CustomData, CustomEvents, ExecutionStack};

/// The execution stack for the app contract
/// TODO : We need to make sure that for queries, this is not used
impl<
        Error: ContractError,
        CustomInitMsg: 'static,
        CustomExecMsg: 'static,
        CustomQueryMsg: 'static,
        CustomMigrateMsg: 'static,
        ReceiveMsg: 'static,
        SudoMsg: 'static,
    > ExecutionStack
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
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
        CustomMigrateMsg: 'static,
        ReceiveMsg: 'static,
        SudoMsg: 'static,
    > CustomEvents
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
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
        CustomMigrateMsg: 'static,
        ReceiveMsg: 'static,
        SudoMsg: 'static,
    > CustomData
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    fn data(&self) -> Option<cosmwasm_std::Binary> {
        self.response.data()
    }

    fn set_data(&mut self, data: impl Into<cosmwasm_std::Binary>) {
        self.response.set_data(data)
    }
}
