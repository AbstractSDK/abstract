use crate::{
    contract::{ContractInstance, Interface},
    sender::{GroupConfig},
};
use cosmwasm_std::Empty;

use pandora_os::os_factory::msg::*;

pub trait OsFactory {
    fn new(group_config: GroupConfig) -> ContractInstance<InstantiateMsg, ExecuteMsg, QueryMsg, Empty>;
}

impl OsFactory for ContractInstance<InstantiateMsg, ExecuteMsg, QueryMsg, Empty> {
    fn new(group_config: GroupConfig) -> ContractInstance<InstantiateMsg, ExecuteMsg, QueryMsg, Empty> {
        let instance = ContractInstance {
            interface: Interface::default(),
            group_config,
            name: "os_factory".to_string(),
        };
        instance.check_scaffold().unwrap();
        instance
    }
}
