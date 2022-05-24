use crate::dapp_base::common::MEMORY_CONTRACT;
use abstract_os::modules::dapp_base::msg::BaseInstantiateMsg;

#[allow(dead_code)]
pub(crate) fn instantiate_msg() -> BaseInstantiateMsg {
    BaseInstantiateMsg {
        memory_addr: MEMORY_CONTRACT.to_string(),
    }
}
