/// Macro to update the ownership of an Abstract contract.
///
/// ```rustignore
/// pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ContractResult {
///     match msg {
///         ...
///         ExecuteMsg::UpdateOwnership(action) => {
///             execute_update_ownership!(ContractResponse, deps, env, info, action)
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! execute_update_ownership {
    ($response_type:ident, $deps:expr, $env:expr, $info:expr, $action:expr) => {{
        let ownership = cw_ownable::update_ownership($deps, &$env.block, &$info.sender, $action)?;
        Ok($response_type::new(
            "update_ownership",
            ownership.into_attributes(),
        ))
    }};
}

/// Macro to query the ownership of a contract.
///
/// ```rustignore
/// pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
///     match msg {
///         ...
///         QueryMsg::Ownership {} => query_ownership!(deps),
///     }
/// }
/// ```
#[macro_export]
macro_rules! query_ownership {
    ($deps:expr) => {{
        cosmwasm_std::to_binary(&cw_ownable::get_ownership($deps.storage)?)
    }};
}

#[cfg(test)]
mod tests {
    use abstract_macros::abstract_response;
    use cosmwasm_schema::cw_serde;
    use cosmwasm_schema::QueryResponses;
    use cosmwasm_std::{
        from_binary,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Binary, StdError, StdResult,
    };

    use cw_ownable::{cw_ownable_execute, cw_ownable_query, Action, Ownership, OwnershipError};
    use thiserror::Error;

    const MOCK_CONTRACT: &str = "contract";

    #[abstract_response(MOCK_CONTRACT)]
    pub struct MockResponse;

    #[cw_ownable_execute]
    #[cw_serde]
    enum ExecuteMsg {}

    #[cw_ownable_query]
    #[cw_serde]
    #[derive(QueryResponses)]
    enum QueryMsg {}

    #[derive(Error, Debug, PartialEq)]
    pub enum MockError {
        #[error("{0}")]
        Std(#[from] StdError),
        #[error("{0}")]
        Ownership(#[from] OwnershipError),
    }

    const NEW_OWNER: &str = "new_owner";
    const OLD_OWNER: &str = "old_owner";

    #[test]
    fn test_update_ownership_macro() -> Result<(), MockError> {
        let mut deps = mock_dependencies();

        let env = mock_env();
        let info = mock_info(OLD_OWNER, &[]);

        cw_ownable::initialize_owner(&mut deps.storage, &deps.api, Some(OLD_OWNER))?;

        let mut_deps = deps.as_mut();

        // ExecuteMsg for testing the macro
        let transfer_ownership_action = Action::TransferOwnership {
            new_owner: NEW_OWNER.to_string(),
            expiry: None,
        };

        let ownership_msg = ExecuteMsg::UpdateOwnership(transfer_ownership_action);

        let result: Result<_, OwnershipError> = match ownership_msg {
            ExecuteMsg::UpdateOwnership(action) => {
                execute_update_ownership!(MockResponse, mut_deps, env, info, action)
            }
        };

        let expected_response = MockResponse::new(
            "update_ownership",
            vec![
                ("owner", OLD_OWNER),
                ("pending_owner", NEW_OWNER),
                ("pending_expiry", "none"),
            ],
        );

        assert_eq!(result.unwrap(), expected_response);

        Ok(())
    }

    #[test]
    fn test_query_ownership_macro() -> Result<(), MockError> {
        let mut deps = mock_dependencies();
        let _env = mock_env();

        let old_owner = "owner1";

        cw_ownable::initialize_owner(&mut deps.storage, &deps.api, Some(old_owner))?;

        // Ownership query message for testing the macro
        let ownership_query_msg = QueryMsg::Ownership {};

        let result: StdResult<Binary> = match ownership_query_msg {
            QueryMsg::Ownership {} => query_ownership!(deps.as_ref()),
        };

        let expected = Ownership {
            owner: Some(Addr::unchecked(old_owner)),
            pending_owner: None,
            pending_expiry: None,
        };

        // Deserialize the query response
        let actual: Ownership<Addr> = from_binary(&result.unwrap())?;

        assert_eq!(actual, expected);

        Ok(())
    }
}
