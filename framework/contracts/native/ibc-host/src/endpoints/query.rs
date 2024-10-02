use std::str::FromStr;

use abstract_sdk::{
    feature_objects::{AnsHost, VersionControlContract},
    std::ibc_host::QueryMsg,
};
use abstract_std::{
    ibc_host::{state::CHAIN_PROXIES, ClientProxiesResponse, ClientProxyResponse, ConfigResponse},
    objects::{module_factory::ModuleFactoryContract, TruncatedChainId},
};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env};
use cw_storage_plus::Bound;

use crate::{contract::HostResult, HostError};

use super::packet;

pub fn query(deps: Deps, env: Env, query: QueryMsg) -> HostResult<Binary> {
    match query {
        QueryMsg::Config {} => to_json_binary(&config(deps, &env)?),
        QueryMsg::ClientProxies { start_after, limit } => {
            to_json_binary(&registered_chains(deps, start_after, limit)?)
        }
        QueryMsg::ClientProxy { chain } => to_json_binary(&associated_client(deps, chain)?),
        QueryMsg::Ownership {} => to_json_binary(&cw_ownable::get_ownership(deps.storage)?),
        QueryMsg::ModuleQuery { target_module, msg } => {
            return packet::handle_host_module_query(deps, env, target_module, msg);
        }
    }
    .map_err(Into::into)
}

fn config(deps: Deps, env: &Env) -> HostResult<ConfigResponse> {
    Ok(ConfigResponse {
        ans_host_address: AnsHost::new(deps.api, env)?.address,
        module_factory_address: ModuleFactoryContract::new(deps.api, env)?.address,
        version_control_address: VersionControlContract::new(deps.api, env)?.address,
    })
}

fn registered_chains(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> HostResult<ClientProxiesResponse> {
    let start = start_after.map(TruncatedChainId::from_string).transpose()?;

    let chains = cw_paginate::paginate_map(
        &CHAIN_PROXIES,
        deps.storage,
        start.as_ref().map(Bound::exclusive),
        limit,
        |name, proxy| Ok::<_, HostError>((name, proxy)),
    )?;

    Ok(ClientProxiesResponse { chains })
}

fn associated_client(deps: Deps, chain: String) -> HostResult<ClientProxyResponse> {
    let proxy = CHAIN_PROXIES.load(deps.storage, &TruncatedChainId::from_str(&chain)?)?;
    Ok(ClientProxyResponse { proxy })
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]

    #[test]
    fn test_registered_client() {
        use abstract_std::ibc_host::{ClientProxyResponse, InstantiateMsg, QueryMsg};
        use cosmwasm_std::{from_json, testing::*};

        use crate::contract::{execute, instantiate, query};
        // Instantiate
        let mut deps = mock_dependencies();
        let info = message_info(&deps.api.addr_make("admin"), &[]);
        instantiate(deps.as_mut(), mock_env_validated(deps.api), info.clone(), InstantiateMsg {}).unwrap();

        // Register
        let proxy = deps.api.addr_make("juno-proxy");
        execute(
            deps.as_mut(),
            mock_env_validated(deps.api),
            info,
            abstract_std::ibc_host::ExecuteMsg::RegisterChainProxy {
                chain: "juno".parse().unwrap(),
                proxy: proxy.to_string(),
            },
        )
        .unwrap();

        // Query
        let client_name = query(
            deps.as_ref(),
            mock_env_validated(deps.api),
            QueryMsg::ClientProxy {
                chain: "juno".to_string(),
            },
        )
        .unwrap();
        let queried_client_name: ClientProxyResponse = from_json(client_name).unwrap();
        assert_eq!(queried_client_name.proxy, proxy);
    }
}
