use std::collections::BTreeMap;

use abstract_sdk::feature_objects::VersionControlContract;
use abstract_std::{
    account::{
        state::{AccountInfo, ACCOUNT_ID, ACCOUNT_MODULES, CONFIG, INFO},
        state::{Config, SUB_ACCOUNTS, SUSPENSION_STATUS},
        AccountModuleInfo, ConfigResponse, InfoResponse, ModuleAddressesResponse,
        ModuleInfosResponse, ModuleVersionsResponse, SubAccountIdsResponse,
    },
    objects::{
        gov_type::TopLevelOwnerResponse,
        module::{self, ModuleInfo},
        ownership::nested_admin::query_top_level_owner_addr,
    },
};
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, Order, StdError, StdResult};
use cw2::ContractVersion;
use cw_storage_plus::Bound;

const DEFAULT_LIMIT: u8 = 5;
const MAX_LIMIT: u8 = 10;

pub fn handle_module_address_query(deps: Deps, ids: Vec<String>) -> StdResult<Binary> {
    let contracts = query_module_addresses(deps, ids)?;
    let vector = contracts.into_iter().collect();
    to_json_binary(&ModuleAddressesResponse { modules: vector })
}

pub fn handle_module_versions_query(deps: Deps, ids: Vec<String>) -> StdResult<Binary> {
    let response = query_module_versions(deps, ids)?;
    let versions = response.into_values().collect();
    to_json_binary(&ModuleVersionsResponse { versions })
}

pub fn handle_account_info_query(deps: Deps) -> StdResult<Binary> {
    let info: AccountInfo = INFO.load(deps.storage)?;
    to_json_binary(&InfoResponse { info })
}

pub fn handle_config_query(deps: Deps) -> StdResult<Binary> {
    let account_id = ACCOUNT_ID.load(deps.storage)?;
    let Config {
        version_control_address,
        module_factory_address,
        ..
    } = CONFIG.load(deps.storage)?;
    let is_suspended = SUSPENSION_STATUS.load(deps.storage)?;
    to_json_binary(&ConfigResponse {
        account_id,
        is_suspended,
        version_control_address,
        module_factory_address,
        modules: ACCOUNT_MODULES
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<Result<_, _>>()?,
    })
}

pub fn handle_module_info_query(
    deps: Deps,
    last_module_id: Option<String>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_module_id.as_deref().map(Bound::exclusive);

    let res: Result<Vec<(String, Addr)>, _> = ACCOUNT_MODULES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    let ids_and_addr = res?;

    let config = CONFIG.load(deps.storage)?;
    let version_control = VersionControlContract::new(config.version_control_address);

    let mut resp_vec: Vec<AccountModuleInfo> = vec![];
    for (id, address) in ids_and_addr.into_iter() {
        let version = query_module_version(deps, address.clone(), &version_control)?;
        resp_vec.push(AccountModuleInfo {
            id,
            version,
            address,
        })
    }

    to_json_binary(&ModuleInfosResponse {
        module_infos: resp_vec,
    })
}

pub fn handle_sub_accounts_query(
    deps: Deps,
    last_account_id: Option<u32>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_account_id.map(Bound::exclusive);

    let res = SUB_ACCOUNTS
        .keys(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<u32>>>()?;

    to_json_binary(&SubAccountIdsResponse { sub_accounts: res })
}

pub fn handle_top_level_owner_query(deps: Deps, env: Env) -> StdResult<Binary> {
    let addr = query_top_level_owner_addr(&deps.querier, env.contract.address)?;

    to_json_binary(&TopLevelOwnerResponse { address: addr })
}

/// RawQuery the version of an enabled module
pub fn query_module_version(
    deps: Deps,
    module_addr: Addr,
    version_control: &VersionControlContract,
) -> StdResult<ContractVersion> {
    if let Ok(info) = cw2::query_contract_info(&deps.querier, module_addr.to_string()) {
        // Check if it's abstract format and return now
        if ModuleInfo::from_id(
            &info.contract,
            module::ModuleVersion::Version(info.version.clone()),
        )
        .is_ok()
        {
            return Ok(info);
        }
    }
    // Right now we have either
    // - failed cw2 query
    // - the query succeeded but the cw2 name doesn't adhere to our formatting standards
    //
    // Which means this contract is a standalone or service contract. Hence we need to get its information from VersionControl.
    let module_info = match version_control.query_service_info_raw(&module_addr, &deps.querier) {
        // We got service
        Ok(module_info) => module_info,
        // Didn't got service, let's try to get standalone
        Err(_) => {
            let code_id = deps
                .querier
                .query_wasm_contract_info(module_addr.to_string())?
                .code_id;
            version_control
                .query_standalone_info_raw(code_id, &deps.querier)
                .map_err(|e| StdError::generic_err(e.to_string()))?
        }
    };
    let version =
        ContractVersion::try_from(module_info).map_err(|e| StdError::generic_err(e.to_string()))?;
    Ok(version)
}

/// RawQuery the module versions of the modules part of the Account
/// Errors if not present
pub fn query_module_versions(
    deps: Deps,
    module_names: Vec<String>,
) -> StdResult<BTreeMap<String, ContractVersion>> {
    let addresses: BTreeMap<String, Addr> = query_module_addresses(deps, module_names)?;
    let mut module_versions: BTreeMap<String, ContractVersion> = BTreeMap::new();

    let config = CONFIG.load(deps.storage)?;
    let version_control = VersionControlContract::new(config.version_control_address);
    for (name, address) in addresses.into_iter() {
        let result = query_module_version(deps, address, &version_control)?;
        module_versions.insert(name, result);
    }
    Ok(module_versions)
}

/// RawQuery module addresses from manager
/// Errors if not present
pub fn query_module_addresses(
    deps: Deps,
    module_names: Vec<String>,
) -> StdResult<BTreeMap<String, Addr>> {
    let mut modules: BTreeMap<String, Addr> = BTreeMap::new();

    // Query over
    for module in module_names {
        // Add to map if present, skip otherwise. Allows version control to check what modules are present.
        if let Some(address) = ACCOUNT_MODULES.may_load(deps.storage, &module)? {
            modules.insert(module, address);
        }
    }
    Ok(modules)
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;

    use crate::{
        contract::{execute, instantiate, query, AccountResult},
        test_common::{execute_as_admin, mock_init},
    };
    use abstract_std::{
        account::{ExecuteMsg, InstantiateMsg, InternalConfigAction},
        objects::AccountId,
    };
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, mock_env, MockApi},
        OwnedDeps,
    };

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    #[test]
    fn query_config() {
        let mut deps = mock_dependencies();
        deps.querier = MockAnsHost::new(deps.api).with_defaults().to_querier();
        mock_init(&mut deps);
        let abstr = AbstractMockAddrs::new(deps.api);

        execute_as_admin(
            &mut deps,
            ExecuteMsg::UpdateInternalConfig(
                to_json_binary(&InternalConfigAction::UpdateModuleAddresses {
                    to_add: Some(vec![(
                        TEST_MODULE_ID.into(),
                        abstr.module_address.to_string(),
                    )]),
                    to_remove: None,
                })
                .unwrap(),
            ),
        )
        .unwrap();

        let config: ConfigResponse = from_json(
            query(
                deps.as_ref(),
                mock_env(),
                abstract_std::account::QueryMsg::Config {},
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            config,
            ConfigResponse {
                modules: vec![(TEST_MODULE_ID.to_string(), abstr.module_address)],
                account_id: AccountId::local(1),
                is_suspended: false,
                version_control_address: abstr.version_control,
                module_factory_address: abstr.module_factory
            }
        );
    }
}
