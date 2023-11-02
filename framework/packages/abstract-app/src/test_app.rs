use abstract_sdk::{features::{ExecutionStack, Executables, DepsAccess}, namespaces::ADMIN_NAMESPACE};
use cosmwasm_std::{Deps, DepsMut, Response, Empty};
use cw_controllers::Admin;
use cw_storage_plus::Item;

use crate::{mock::{MockAppContract, MOCK_APP}, state::ModuleEnv, AppError, AppContract};

// TODO: add macro here that generates the private struct below
// The macro should:
// 1. Generate a struct that contains this struct and the ModuleEnv
// 2. Generate a new function that instantiates the struct
// 3. 

// This is the custom struct defined by the dev. 
// it contains all the state and handler functions of the contract.
pub struct TestContract {
    // Custom state goes here (like Sylvia)
    pub admin: Admin<'static>,
    pub config: Item<'static, u64>,

    // added automatically
    pub(crate) contract: MockAppContract,
    pub(crate) env: ModuleEnv<'static>,
}

impl Into<MockAppContract> for &TestContract {
    fn into(self) -> MockAppContract {
        self.contract
    }
}

// #[contract] TODO: re-enable this macro
impl TestContract {
    // new function must be implemented manually (sylvia)
    pub fn new(deps: DepsMut) -> Self {
        Self {
            admin: Admin::new(ADMIN_NAMESPACE),
            config: Item::new("cfg"),
            contract: MOCK_APP,
            env: ModuleEnv::new(deps),
        }
    }

    // TODO: re-enable macro #[msg(instantiate)] 
    // the macro removes the impl here and applies it to `_TestContract`
    pub fn instantiate(
        &self,
        admin: Option<String>,
    ) -> Result<Response, AppError> {
        let admin = admin.map(|a| self.env.deps.api.addr_validate(&a)).transpose()?;
        self.admin.set(self.env.deps, admin)?;

        self.bank()

        Ok(Response::new())
    }
}

mod const_contract {
    //! the ConstContract is a generated Const that contains the user-defined contract but exposes the 
    //! base-implementation function endpoints (init, exec, query, migrate) with the custom implementation hooked up.
    //! 
    
    // TODO: generate this struct


}

// Generated from previous struct
pub struct _TestContract<'a> {
    // Module environment (contains deps and executable stack)
    env: ModuleEnv<'a>
}

impl<'a> _TestContract<'a> {
    // This function gets called at the beginning of every endpoint (init, exec, query, migrate)
    pub fn new(deps: DepsMut<'a>) -> Self {
        Self {
            env: ModuleEnv::new(deps),
        }
    }

    pub fn _instantiate(
        &mut self,
        admin: Option<String>,
        sequence: u16,
    ) -> Result<Response, AppError> {
        let admin = admin.map(|a| self.deps().api.addr_validate(&a)).transpose()?;
        
        Self::storage().admin.set(self.deps_mut(), admin)?;

        Ok(Response::new())
    }


}

impl ExecutionStack for _TestContract<'_> {
    fn stack_mut(&mut self) -> &mut Executables {
        &mut self.env.executable_stack
    }
}

// Access the deps inserted into the contract at instantiation.
impl<'a:'b, 'b:'c, 'c> DepsAccess<'a,'b, 'c> for _TestContract<'a> {
    fn deps_mut(&'b mut self) -> DepsMut<'c> {
        self.env.deps.branch()
    }
    fn deps(&'b self) -> Deps<'c> {
        self.env.deps.as_ref()
    }
}

