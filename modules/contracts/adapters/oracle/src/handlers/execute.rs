use crate::{
    contract::{OracleAdapter, OracleResult},
    msg::{OracleAction, OracleExecuteMsg},
    state::Oracle,
    OracleError,
};
use abstract_core::objects::namespace::{Namespace, ABSTRACT_NAMESPACE};
use abstract_sdk::{features::AbstractNameService, ModuleRegistryInterface};
use cosmwasm_std::{ensure_eq, DepsMut, Env, MessageInfo, Response};

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    adapter: OracleAdapter,
    msg: OracleExecuteMsg,
) -> OracleResult {
    let (oracle, action) = match msg {
        OracleExecuteMsg::Admin(oracle_configuration) => {
            // Only namespace owner (abstract) can change recipient address
            let namespace = adapter
                .module_registry(deps.as_ref())?
                .query_namespace(Namespace::new(ABSTRACT_NAMESPACE)?)?;

            // unwrap namespace, since it's unlikely to have unclaimed abstract namespace
            let namespace_info = namespace.unwrap();
            ensure_eq!(
                namespace_info.account_base,
                adapter.target_account.clone().unwrap(),
                OracleError::Unauthorized {}
            );
            let oracle = Oracle::default();
            (oracle, oracle_configuration)
        }
        OracleExecuteMsg::Account(oracle_configuration) => (
            Oracle::new(adapter.target()?.as_str()),
            oracle_configuration,
        ),
    };
    match action {
        OracleAction::UpdateConfig { external_age_max } => {
            oracle.update_config(deps, external_age_max)?;
        }
        OracleAction::UpdateAssets { to_add, to_remove } => {
            let ans = adapter.ans_host(deps.as_ref())?;
            oracle.update_assets(deps, &ans, to_add, to_remove)?;
        }
    }
    Ok(Response::default())
}
