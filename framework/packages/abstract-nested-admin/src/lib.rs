use cosmwasm_std::{
    attr, Addr, CustomQuery, Deps, DepsMut, MessageInfo, QuerierWrapper, Response, StdError,
    StdResult,
};
use cw_controllers::{Admin, AdminError, AdminResponse};
use cw_gov_ownable::Ownership;
use schemars::JsonSchema;

use abstract_std::objects::gov_type::GovernanceDetails;

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
            Some(admin) => Self::is_admin_custom(&deps.querier, caller, admin),
            None => Ok(false),
        }
    }

    /// Compares the provided admin to the caller.
    /// Can be used when other ownership structure than `cw-controller::Admin` is used.
    pub fn is_admin_custom<Q: CustomQuery>(
        querier: &QuerierWrapper<Q>,
        caller: &Addr,
        admin: Addr,
    ) -> StdResult<bool> {
        // Initial check if directly called by the admin
        if caller == admin {
            Ok(true)
        } else {
            // Check if top level owner address is equal to the caller
            Ok(query_top_level_owner_addr(querier, admin)
                .map(|admin| admin == caller)
                .unwrap_or(false))
        }
    }

    /// Assert the caller is the admin of this nested ownership structures
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

    /// Assert the caller is the admin of this nested ownership structures
    /// Either directly or indirectly
    pub fn assert_admin_custom<Q: CustomQuery>(
        querier: &QuerierWrapper<Q>,
        caller: &Addr,
        admin: Addr,
    ) -> Result<(), AdminError> {
        if !Self::is_admin_custom(querier, caller, admin)? {
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
            Some(owner) => Some(query_top_level_owner_addr(&deps.querier, owner).map_err(|_| {
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

pub fn query_top_level_owner_addr<Q: CustomQuery>(
    querier: &QuerierWrapper<Q>,
    maybe_manager: Addr,
) -> StdResult<Addr> {
    // Get top level account owner address
    query_top_level_owner(querier, maybe_manager).and_then(|ownership| {
        ownership
            .owner
            .owner_address(&querier.into_empty())
            .ok_or(StdError::generic_err("Top level account got renounced"))
    })
}

pub fn query_top_level_owner<Q: CustomQuery>(
    querier: &QuerierWrapper<Q>,
    maybe_manager: Addr,
) -> StdResult<Ownership<Addr>> {
    // Starting from (potentially)manager that owns this module
    let mut current = cw_gov_ownable::query_ownership(querier, maybe_manager);
    // Get sub-accounts until we get non-sub-account governance or reach recursion limit
    for _ in 0..MAX_ADMIN_RECURSION {
        match current {
            Ok(Ownership {
                owner: GovernanceDetails::SubAccount { manager, .. },
                ..
            }) => {
                current = cw_gov_ownable::query_ownership(querier, manager);
            }
            _ => break,
        }
    }

    current
}
