use abstract_sdk::os::objects::module::Module;
use abstract_sdk::os::objects::module::ModuleInfo;
use abstract_sdk::os::objects::module::ModuleVersion;
use abstract_sdk::os::objects::module_reference::ModuleReference;
use abstract_sdk::os::version_control::state::MODULE_LIBRARY;
use abstract_sdk::os::version_control::ModuleResponse;
use abstract_sdk::os::version_control::ModulesResponse;
use abstract_sdk::os::version_control::OsCoreResponse;
use cosmwasm_std::Order;
use cosmwasm_std::StdError;
use cw_storage_plus::Bound;

use crate::error::VCError;
use abstract_sdk::os::version_control::state::OS_ADDRESSES;
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
                msg: VCError::ModuleNotInstalled(module.clone()).to_string(),
            })?
            .clone();
        module.version = ModuleVersion::Version(latest_version);
        Ok(id)
    };

    match maybe_module {
        Err(_) => Err(StdError::generic_err(
            VCError::ModuleNotInstalled(module).to_string(),
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

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{DepsMut, StdError};

    use abstract_os::version_control::*;

    use crate::contract;
    use crate::contract::VCResult;
    use speculoos::prelude::*;

    use super::*;

    type VersionControlTestResult = Result<(), VCError>;

    const TEST_ADMIN: &str = "testadmin";

    /// Initialize the version_control with admin as creator and factory
    fn mock_init(mut deps: DepsMut) -> VCResult {
        let info = mock_info(TEST_ADMIN, &[]);
        contract::instantiate(deps.branch(), mock_env(), info, InstantiateMsg {})
    }

    fn execute_helper(deps: DepsMut, msg: ExecuteMsg) -> VCResult {
        contract::execute(deps, mock_env(), mock_info(TEST_ADMIN, &[]), msg)
    }

    fn query_helper(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
        contract::query(deps, mock_env(), msg)
    }

    mod module {
        use super::*;
        use abstract_os::objects::module::ModuleVersion::Latest;

        use cosmwasm_std::from_binary;

        fn add_module(deps: DepsMut, new_module_info: ModuleInfo) {
            let add_msg = ExecuteMsg::AddModules {
                modules: vec![(new_module_info, ModuleReference::App(0))],
            };

            let res = execute_helper(deps, add_msg);
            assert_that!(&res).is_ok();
        }

        #[test]
        fn get_module() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let new_module_info =
                ModuleInfo::from_id("test:module", ModuleVersion::Version("0.1.2".into())).unwrap();

            let ModuleInfo { name, provider, .. } = new_module_info.clone();

            add_module(deps.as_mut(), new_module_info.clone());

            let query_msg = QueryMsg::Module {
                module: ModuleInfo {
                    provider,
                    name,
                    version: Latest {},
                },
            };

            let module: ModuleResponse = from_binary(&query_helper(deps.as_ref(), query_msg)?)?;
            assert_that!(module.module.info).is_equal_to(&new_module_info);
            Ok(())
        }

        #[test]
        fn none_when_no_matching_version() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let new_module_info =
                ModuleInfo::from_id("test:module", ModuleVersion::Version("0.1.2".into())).unwrap();

            let ModuleInfo { name, provider, .. } = new_module_info.clone();

            add_module(deps.as_mut(), new_module_info);

            let query_msg = QueryMsg::Module {
                module: ModuleInfo {
                    provider,
                    name,
                    version: ModuleVersion::Version("024209.902.902".to_string()),
                },
            };

            let res = query_helper(deps.as_ref(), query_msg);
            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, StdError::GenericErr { .. }));
            Ok(())
        }

        #[test]
        fn get_latest_when_multiple_registered() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let module_id = "test:module";
            let oldest_version =
                ModuleInfo::from_id(module_id, ModuleVersion::Version("0.1.2".into())).unwrap();

            add_module(deps.as_mut(), oldest_version);
            let newest_version =
                ModuleInfo::from_id(module_id, ModuleVersion::Version("100.1.2".into())).unwrap();
            add_module(deps.as_mut(), newest_version.clone());

            let another_version =
                ModuleInfo::from_id(module_id, ModuleVersion::Version("1.1.2".into())).unwrap();
            add_module(deps.as_mut(), another_version);

            let query_msg = QueryMsg::Module {
                module: ModuleInfo {
                    provider: "test".to_string(),
                    name: "module".to_string(),
                    version: Latest {},
                },
            };

            let module: ModuleResponse = from_binary(&query_helper(deps.as_ref(), query_msg)?)?;
            assert_that!(module.module.info).is_equal_to(&newest_version);
            Ok(())
        }
    }
}
