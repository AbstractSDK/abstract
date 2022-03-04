use crate::{
    contract::{ContractInstance, Interface},
    error::TerraRustScriptError,
    sender::{GroupConfig, Sender},
};
use cosmwasm_std::Empty;

pub type Template = ContractInstance<Empty, Empty, Empty, Empty>;

impl Template {
    pub fn new(group_config: GroupConfig) -> Template {
        let instance = ContractInstance {
            interface: Interface::default(),
            group_config,
            name: "template".to_string(),
        };
        instance.check_scaffold().unwrap();
        instance
    }
}
