use crate::{
    account::state::{CALLING_TO_AS_ADMIN, CALLING_TO_AS_ADMIN_WILD_CARD},
    objects::{gov_type::GovernanceDetails, ownership::Ownership},
};

use cosmwasm_std::{
    attr, Addr, CustomQuery, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, Response, StdError,
    StdResult,
};
use cw_controllers::{Admin, AdminError, AdminResponse};
use schemars::JsonSchema;

use super::query_ownership;

/// Max account admin recursion
pub const MAX_ADMIN_RECURSION: usize = 2;

/// # Abstract Admin Object
/// This object has a similar api to the [cw_controllers::Admin] object but incorporates nested ownership and abstract-specific Admin checks.
///
/// The ownership of a contract can be nested, meaning that the owner of the contract can be owned by another contract (or address) and so on.
///
/// By using this structure we allow both the direct owner as well as the top-level owner to have permissions to perform actions that are gated by this object.
///
/// See [NestedAdmin::assert_admin] for more details on how the admin rights are checked.
pub struct NestedAdmin(Admin);

impl NestedAdmin {
    pub const fn new(namespace: &'static str) -> Self {
        NestedAdmin(Admin::new(namespace))
    }

    pub fn set<Q: CustomQuery>(&self, deps: DepsMut<Q>, admin: Option<Addr>) -> StdResult<()> {
        self.0.set(deps, admin)
    }

    pub fn get<Q: CustomQuery>(&self, deps: Deps<Q>) -> StdResult<Option<Addr>> {
        self.0.get(deps)
    }

    /// See [NestedAdmin::assert_admin] for more details.
    pub fn is_admin<Q: CustomQuery>(
        &self,
        deps: Deps<Q>,
        env: &Env,
        caller: &Addr,
    ) -> StdResult<bool> {
        match self.0.get(deps)? {
            Some(admin) => Self::is_admin_custom(&deps.querier, env, caller, admin),
            None => Ok(false),
        }
    }

    /// See [NestedAdmin::assert_admin] for more details.
    pub fn is_admin_custom<Q: CustomQuery>(
        querier: &QuerierWrapper<Q>,
        env: &Env,
        caller: &Addr,
        admin: Addr,
    ) -> StdResult<bool> {
        // Initial check if directly called by the admin
        if caller == admin && assert_account_calling_to_as_admin_is_self(querier, env, caller) {
            // If the caller is the admin, we still need to check that
            // if it's an account it's authorized to act as an admin

            Ok(true)
        } else {
            // Check if top level owner address is equal to the caller
            Ok(query_top_level_owner_addr(querier, admin)
                .map(|admin| admin == caller)
                .unwrap_or(false))
        }
    }

    /// Assert that the caller is allowed to perform admin actions.
    ///
    /// This method will pass in two specific scenarios:
    ///
    /// - If the caller is the direct admin of the contract. I.e. the admin stored in this contract. AND the state `CALLING_TO_AS_ADMIN` is set to the contract address or a wildcard.
    /// - If the caller is the **top-level** admin of the chain of ownership, starting from this contract.
    pub fn assert_admin<Q: CustomQuery>(
        &self,
        deps: Deps<Q>,
        env: &Env,
        caller: &Addr,
    ) -> Result<(), AdminError> {
        if !self.is_admin(deps, env, caller)? {
            Err(AdminError::NotAdmin {})
        } else {
            Ok(())
        }
    }

    /// Assert that the caller is allowed to perform admin actions.
    ///
    /// This method will pass in two specific scenarios:
    ///
    /// - If the caller is the direct admin of the contract. I.e. the admin stored in this contract. AND the state `CALLING_TO_AS_ADMIN` is set to the contract address or a wildcard.
    /// - If the caller is the **top-level** admin of the chain of ownership, starting from this contract.
    pub fn assert_admin_custom<Q: CustomQuery>(
        querier: &QuerierWrapper<Q>,
        env: &Env,
        caller: &Addr,
        admin: Addr,
    ) -> Result<(), AdminError> {
        if !Self::is_admin_custom(querier, env, caller, admin)? {
            Err(AdminError::NotAdmin {})
        } else {
            Ok(())
        }
    }

    pub fn execute_update_admin<C, Q: CustomQuery>(
        &self,
        deps: DepsMut<Q>,
        env: &Env,
        info: MessageInfo,
        new_admin: Option<Addr>,
    ) -> Result<Response<C>, AdminError>
    where
        C: Clone + core::fmt::Debug + PartialEq + JsonSchema,
    {
        self.assert_admin(deps.as_ref(), env, &info.sender)?;

        let admin_str = match new_admin.as_ref() {
            Some(admin) => admin.to_string(),
            None => "None".to_string(),
        };
        let attributes = vec![
            attr("action", "update_admin"),
            attr("admin", admin_str),
            attr("sender", info.sender),
        ];

        self.set(deps, new_admin)?;

        Ok(Response::new().add_attributes(attributes))
    }

    // This method queries direct module owner
    pub fn query_admin<Q: CustomQuery>(&self, deps: Deps<Q>) -> StdResult<AdminResponse> {
        self.0.query_admin(deps)
    }

    // This method tries to get top-level account owner
    pub fn query_account_owner<Q: CustomQuery>(&self, deps: Deps<Q>) -> StdResult<AdminResponse> {
        let admin = match self.0.get(deps)? {
            Some(owner) => Some(query_top_level_owner_addr(&deps.querier, owner).map_err(|_| {
                StdError::generic_err(
                    "Failed to query top level owner. Make sure this module is owned by the account",
                )
            })?),
            None => None,
        };
        Ok(AdminResponse {
            admin: admin.map(|addr| addr.into_string()),
        })
    }
}

pub fn query_top_level_owner_addr<Q: CustomQuery>(
    querier: &QuerierWrapper<Q>,
    maybe_account: Addr,
) -> StdResult<Addr> {
    // Get top level account owner address
    query_top_level_owner(querier, maybe_account).and_then(|ownership| {
        ownership
            .owner
            .owner_address(&querier.into_empty())
            .ok_or(StdError::generic_err("Top level account got renounced"))
    })
}

pub fn query_top_level_owner<Q: CustomQuery>(
    querier: &QuerierWrapper<Q>,
    maybe_account: Addr,
) -> StdResult<Ownership<Addr>> {
    // Starting from (potentially)account that owns this module
    let mut current = query_ownership(querier, maybe_account);
    // Get sub-accounts until we get non-sub-account governance or reach recursion limit
    for _ in 0..MAX_ADMIN_RECURSION {
        match current {
            Ok(Ownership {
                owner: GovernanceDetails::SubAccount { account },
                ..
            }) => {
                current = query_ownership(querier, account);
            }
            _ => break,
        }
    }

    current
}

/// Assert that the account has a valid calling to the contract as an admin.
pub fn assert_account_calling_to_as_admin_is_self<Q: CustomQuery>(
    querier: &QuerierWrapper<Q>,
    env: &Env,
    maybe_account: &Addr,
) -> bool {
    CALLING_TO_AS_ADMIN
        .query(querier, maybe_account.clone())
        .map(|admin_call_to| {
            admin_call_to == env.contract.address
                || admin_call_to.as_str() == CALLING_TO_AS_ADMIN_WILD_CARD
        })
        .unwrap_or(false)
}
