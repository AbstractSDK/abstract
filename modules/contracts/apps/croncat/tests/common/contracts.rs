use cosmwasm_std::{Addr, Empty};
use croncat_app::msg::{ActiveTasksByCreatorResponse, ActiveTasksResponse};
use cw_multi_test::{Contract, ContractWrapper};

pub(crate) use croncat_integration_testing::contracts::{
    croncat_agents_contract, croncat_factory_contract, croncat_manager_contract,
    croncat_tasks_contract,
};

pub(crate) fn cw20_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

pub(crate) trait TasksResponseCaster {
    type Item;

    fn unchecked(self) -> Vec<Self::Item>;
    fn checked(self) -> (Vec<Self::Item>, Vec<Self::Item>);
}

impl TasksResponseCaster for ActiveTasksResponse {
    type Item = (Addr, String);

    fn unchecked(self) -> Vec<Self::Item> {
        match self {
            ActiveTasksResponse::Unchecked { tasks } => tasks,
            _ => panic!(),
        }
    }

    fn checked(self) -> (Vec<Self::Item>, Vec<Self::Item>) {
        match self {
            ActiveTasksResponse::Checked {
                scheduled_tasks,
                removed_tasks,
            } => (scheduled_tasks, removed_tasks),
            _ => panic!(),
        }
    }
}

impl TasksResponseCaster for ActiveTasksByCreatorResponse {
    type Item = String;

    fn unchecked(self) -> Vec<Self::Item> {
        match self {
            ActiveTasksByCreatorResponse::Unchecked { tasks } => tasks,
            _ => panic!(),
        }
    }

    fn checked(self) -> (Vec<Self::Item>, Vec<Self::Item>) {
        match self {
            ActiveTasksByCreatorResponse::Checked {
                scheduled_tasks,
                removed_tasks,
            } => (scheduled_tasks, removed_tasks),
            _ => panic!(),
        }
    }
}
