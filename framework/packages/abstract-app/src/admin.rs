use abstract_core::{
    manager::{self, state::AccountInfo},
    objects::gov_type::GovernanceDetails,
};
use cosmwasm_std::{attr, Addr, CustomQuery, Deps, DepsMut, MessageInfo, Response, StdResult};
use cw_controllers::{Admin, AdminError, AdminResponse};
use schemars::JsonSchema;

pub const MAX_ADMIN_RECURSION: usize = 2;

/// App Admin object
/// This object has same api to the [cw_controllers::Admin]
/// but allows top-level abstract account owner to have admin privileges on the app
pub struct AppAdmin<'a>(Admin<'a>);

impl<'a> AppAdmin<'a> {
    pub const fn new(namespace: &'a str) -> Self {
        AppAdmin(Admin::new(namespace))
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
                    return Ok(true);
                }

                // Check for top level owner

                // Starting from (potentially)manager that owns this app
                let mut current = manager::state::INFO.query(&deps.querier, owner.clone());
                // Get sub-accounts until we get non-sub-account governance or reach recursion limit
                for _ in 0..MAX_ADMIN_RECURSION {
                    match &current {
                        Ok(AccountInfo {
                            governance_details: GovernanceDetails::SubAccount { manager, .. },
                            ..
                        }) => {
                            current = manager::state::INFO.query(&deps.querier, manager.clone());
                        }
                        _ => break,
                    }
                }

                // Check if top level owner address is equal to the caller
                let is_admin = current
                    .map(|info| info.governance_details.owner_address() == caller)
                    .unwrap_or(false);
                Ok(is_admin)
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

    // TODO: this will only return direct admin
    pub fn query_admin<Q: CustomQuery>(&self, deps: Deps<Q>) -> StdResult<AdminResponse> {
        self.0.query_admin(deps)
    }
}
