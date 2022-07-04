use crate::dapp_base::common::{MEMORY_CONTRACT, TRADER_CONTRACT, TREASURY_CONTRACT};
use abstract_os::proxy::dapp_base::BaseInstantiateMsg;

#[allow(dead_code)]
pub(crate) fn instantiate_msg() -> BaseInstantiateMsg {
    BaseInstantiateMsg {
        memory_addr: MEMORY_CONTRACT.to_string(),
        proxy_address: TREASURY_CONTRACT.to_string(),
        trader: TRADER_CONTRACT.to_string(),
    }
}
