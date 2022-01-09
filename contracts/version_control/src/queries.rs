use cw_storage_plus::Prefix;
use cw_storage_plus::Bound;
use cosmwasm_std::Addr;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult, Order};
use manager::state::OS_MODULES;
use dao_os::version_control::msg::{QueryMsg,EnabledModulesResponse};

pub fn query_enabled_modules(deps: Deps, env: Env, manager_addr: Addr) -> StdResult<Binary> {
    let modules: Vec<Vec<u8>> = OS_MODULES.keys(deps.storage, None, None, Order::Ascending).collect();
    
    let module_names: Vec<String> = modules.into_iter()
     .map(|module| String::from_utf8(module).unwrap())
     .collect();
 
    // for module in modules.into_iter() {
    //     module_names.push(String::from_utf8(module)?);
    // };
    to_binary(&EnabledModulesResponse {
        modules: module_names
    })
}