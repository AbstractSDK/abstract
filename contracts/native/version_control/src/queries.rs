use abstract_os::objects::module::Module;
use abstract_os::objects::module::ModuleInfo;
use abstract_os::objects::module::ModuleVersion;
use abstract_os::objects::module_reference::ModuleReference;
use abstract_os::version_control::state::MODULE_LIBRARY;
use abstract_os::version_control::ModuleResponse;
use abstract_os::version_control::ModulesResponse;
use abstract_os::version_control::OsCoreResponse;
use cosmwasm_std::Order;
use cosmwasm_std::StdError;
use cw_storage_plus::Bound;

use crate::error::VCError;
use abstract_os::version_control::state::OS_ADDRESSES;
use cosmwasm_std::{to_binary, Binary, Deps, StdResult};

const DEFAULT_LIMIT: u8 = 10;
const MAX_LIMIT: u8 = 20;

pub fn handle_os_address_query(deps: Deps, os_id: u32) -> StdResult<Binary> {
    let os_address = OS_ADDRESSES.load(deps.storage, os_id);
    match os_address {
        Err(_) => Err(StdError::generic_err(
            VCError::MissingOsId { id: os_id }.to_string(),
        )),
        Ok(core) => to_binary(&OsCoreResponse { os_core: core }),
    }
}

pub fn handle_module_query(deps: Deps, mut module: ModuleInfo) -> StdResult<Binary> {
    let maybe_module = if let ModuleVersion::Version(_) = module.version {
        MODULE_LIBRARY.load(deps.storage, module.clone())
    } else {
        // get latest
        let versions: StdResult<Vec<(String, ModuleReference)>> = MODULE_LIBRARY
            .prefix((module.provider.clone(), module.name.clone()))
            .range(deps.storage, None, None, Order::Descending)
            .take(1)
            .collect();
        let (latest_version, id) = versions?
            .first()
            .ok_or_else(|| StdError::GenericErr {
                msg: VCError::MissingModule(module.clone()).to_string(),
            })?
            .clone();
        module.version = ModuleVersion::Version(latest_version);
        Ok(id)
    };

    match maybe_module {
        Err(_) => Err(StdError::generic_err(
            VCError::MissingModule(module).to_string(),
        )),
        Ok(mod_ref) => to_binary(&ModuleResponse {
            module: Module {
                info: module,
                reference: mod_ref,
            },
        }),
    }
}

pub fn handle_modules_query(
    deps: Deps,
    page_token: Option<ModuleInfo>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound: Option<Bound<ModuleInfo>> = page_token.map(Bound::exclusive);

    let res: Result<Vec<(ModuleInfo, ModuleReference)>, _> = MODULE_LIBRARY
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    to_binary(&ModulesResponse { modules: res? })
}
