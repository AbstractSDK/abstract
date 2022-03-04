use cosmwasm_std::Empty;
use pandora_os::memory::msg::*;
use terra_rust_script::{
    contract::{ContractInstance, Interface},
    sender::GroupConfig,
};

pub struct Memory(ContractInstance<InstantiateMsg, ExecuteMsg, QueryMsg, Empty>);

impl Memory  
{
    pub fn new(
        group_config: GroupConfig,
    ) -> ContractInstance<InstantiateMsg, ExecuteMsg, QueryMsg, Empty> {
        let instance = ContractInstance {
            interface: Interface::default(),
            group_config,
            name: "memory".to_string(),
        };
        instance.check_scaffold().unwrap();
        instance
    }
}
