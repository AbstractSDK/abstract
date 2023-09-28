use cosmwasm_std::{Addr, Empty};
use croncat_app::msg::{ActiveTasksByCreatorResponse, ActiveTasksResponse};
use cw_orch::mock::cw_multi_test::{Contract, ContractWrapper};

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

pub fn croncat_factory_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_factory::contract::execute,
        croncat_factory::contract::instantiate,
        croncat_factory::contract::query,
    )
    .with_reply(croncat_factory::contract::reply);
    Box::new(contract)
}

pub fn croncat_manager_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_manager::contract::execute,
        croncat_manager::contract::instantiate,
        croncat_manager::contract::query,
    )
    .with_reply(croncat_manager::contract::reply);
    Box::new(contract)
}

pub fn croncat_tasks_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_tasks::contract::execute,
        croncat_tasks::contract::instantiate,
        croncat_tasks::contract::query,
    );
    Box::new(contract)
}

pub fn croncat_agents_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_agents::contract::execute,
        croncat_agents::contract::instantiate,
        croncat_agents::contract::query,
    );
    Box::new(contract)
}
