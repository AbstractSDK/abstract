use cosmwasm_std::{
    attr, Addr, CustomQuery, Deps, DepsMut, MessageInfo, QuerierWrapper, Response, StdError,
    StdResult,
};
use cw_controllers::{Admin, AdminError, AdminResponse};
use schemars::JsonSchema;

use crate::{
    manager::{self, state::AccountInfo},
    objects::gov_type::GovernanceDetails,
};

/// Max manager admin recursion
pub const MAX_ADMIN_RECURSION: usize = 2;

/// Abstract Admin object
/// This object has same api to the [cw_controllers::Admin]
/// With added query_account_owner method (will get top-level owner in case of sub-accounts)
/// but allows top-level abstract account owner to have admin privileges on the module
pub struct NestedAdmin<'a>(Admin<'a>);

impl<'a> NestedAdmin<'a> {
    pub const fn new(namespace: &'a str) -> Self {
        NestedAdmin(Admin::new(namespace))
    }

    pub fn set<Q: CustomQuery>(&self, deps: DepsMut<Q>, admin: Option<Addr>) -> StdResult<()> {
        self.0.set(deps, admin)
    }

    pub fn get<Q: CustomQuery>(&self, deps: Deps<Q>) -> StdResult<Option<Addr>> {
        self.0.get(deps)
    }

    pub fn is_admin<Q: CustomQuery>(&self, deps: Deps<Q>, caller: &Addr) -> StdResult<bool> {
        match self.0.get(deps)? {
            Some(owner) => {
                // Initial check if directly called by the owner
                if caller == owner {
                    Ok(true)
                } else {
                    // Check if top level owner address is equal to the caller
                    Ok(query_top_level_owner(&deps.querier, owner)
                        .map(|owner| owner == caller)
                        .unwrap_or(false))
                }
            }
            None => Ok(false),
        }
    }

    pub fn assert_admin<Q: CustomQuery>(
        &self,
        deps: Deps<Q>,
        caller: &Addr,
    ) -> Result<(), AdminError> {
        if !self.is_admin(deps, caller)? {
            Err(AdminError::NotAdmin {})
        } else {
            Ok(())
        }
    }

    pub fn execute_update_admin<C, Q: CustomQuery>(
        &self,
        deps: DepsMut<Q>,
        info: MessageInfo,
        new_admin: Option<Addr>,
    ) -> Result<Response<C>, AdminError>
    where
        C: Clone + core::fmt::Debug + PartialEq + JsonSchema,
    {
        self.assert_admin(deps.as_ref(), &info.sender)?;

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
            Some(owner) => Some(query_top_level_owner(&deps.querier, owner).map_err(|_| {
                StdError::generic_err(
                    "Failed to query top level owner. Make sure this module is owned by the manager",
                )
            })?),
            None => None,
        };
        Ok(AdminResponse {
            admin: admin.map(|addr| addr.into_string()),
        })
    }
}

pub fn query_top_level_owner<Q: CustomQuery>(
    querier: &QuerierWrapper<Q>,
    maybe_manager: Addr,
) -> StdResult<Addr> {
    // Starting from (potentially)manager that owns this module
    let mut current = manager::state::INFO.query(querier, maybe_manager.clone());
    // Get sub-accounts until we get non-sub-account governance or reach recursion limit
    for _ in 0..MAX_ADMIN_RECURSION {
        match &current {
            Ok(AccountInfo {
                governance_details: GovernanceDetails::SubAccount { manager, .. },
                ..
            }) => {
                current = manager::state::INFO.query(querier, manager.clone());
            }
            _ => break,
        }
    }

    // Get top level account owner address
    current.and_then(|info| {
        info.governance_details
            .owner_address()
            .ok_or(StdError::generic_err("Top level account got renounced"))
    })
}

#[cosmwasm_schema::cw_serde]
pub struct TopLevelOwnerResponse {
    pub address: Addr,
}
