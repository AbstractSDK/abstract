use cosmwasm_std::{Addr, Deps, StdResult};

pub trait RegisterAccess: Sized {
    fn registry(&self, deps: Deps) -> StdResult<Addr>;
}

// / Query module information
// pub fn get_module(
//     querier: &QuerierWrapper,
//     module_info: ModuleInfo,
//     version_control_addr: &Addr,
// ) -> StdResult<Module> {
//     let resp: ModuleResponse = querier.query_wasm_smart(
//         version_control_addr,
//         &QueryMsg::Module {
//             module: module_info,
//         },
//     )?;
//     Ok(resp.module)
// }
