use cosmwasm_std::{Addr, Deps};

use crate::{error::AbstractError, AbstractResult};

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum ModuleReference {
    /// Account Contract
    Account(u64),
    /// Native Abstract Contracts
    Native(Addr),
    /// Installable adapters
    Adapter(Addr),
    /// Installable apps
    App(u64),
    /// A stand-alone contract
    Standalone(u64),
    /// A contract that exposes some service to others.
    Service(Addr),
}

impl ModuleReference {
    /// Validates that addresses are valid
    pub fn validate(&self, deps: Deps) -> AbstractResult<()> {
        match self {
            ModuleReference::Native(addr)
            | ModuleReference::Adapter(addr)
            | ModuleReference::Service(addr) => {
                deps.api.addr_validate(addr.as_str())?;
            }
            _ => (),
        };
        Ok(())
    }

    pub fn unwrap_account(&self) -> AbstractResult<u64> {
        match self {
            ModuleReference::Account(v) => Ok(*v),
            _ => Err(AbstractError::Assert(
                "module reference not an account module.".to_string(),
            )),
        }
    }

    pub fn unwrap_native(&self) -> AbstractResult<Addr> {
        match self {
            ModuleReference::Native(addr) => Ok(addr.clone()),
            _ => Err(AbstractError::Assert(
                "module reference not a native module.".to_string(),
            )),
        }
    }

    pub fn unwrap_adapter(&self) -> AbstractResult<Addr> {
        match self {
            ModuleReference::Adapter(addr) => Ok(addr.clone()),
            _ => Err(AbstractError::Assert(
                "module reference not an api module.".to_string(),
            )),
        }
    }

    pub fn unwrap_app(&self) -> AbstractResult<u64> {
        match self {
            ModuleReference::App(v) => Ok(*v),
            _ => Err(AbstractError::Assert(
                "module reference not an app module.".to_string(),
            )),
        }
    }

    pub fn unwrap_standalone(&self) -> AbstractResult<u64> {
        match self {
            ModuleReference::Standalone(v) => Ok(*v),
            _ => Err(AbstractError::Assert(
                "module reference not a standalone module.".to_string(),
            )),
        }
    }

    pub fn unwrap_service(&self) -> AbstractResult<Addr> {
        match self {
            ModuleReference::Service(addr) => Ok(addr.clone()),
            _ => Err(AbstractError::Assert(
                "module reference not a service module.".to_string(),
            )),
        }
    }

    /// Unwraps the module reference and returns the address of the module.
    /// Throws an error if the module reference is not an address.
    pub fn unwrap_addr(&self) -> AbstractResult<Addr> {
        match self {
            ModuleReference::Native(addr)
            | ModuleReference::Adapter(addr)
            | ModuleReference::Service(addr) => Ok(addr.clone()),
            _ => Err(AbstractError::Assert(
                "module reference not a native or api module.".to_string(),
            )),
        }
    }

    // Unwraps the module reference and returns code id of the module
    // Throws an error if the module reference is not an code id
    pub fn unwrap_code_id(&self) -> AbstractResult<u64> {
        match self {
            ModuleReference::Account(code_id)
            | ModuleReference::App(code_id)
            | ModuleReference::Standalone(code_id) => Ok(*code_id),
            _ => Err(AbstractError::Assert(
                "module reference not account, app or standalone".to_owned(),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use cosmwasm_std::testing::mock_dependencies;
    use speculoos::prelude::*;

    use super::*;

    #[coverage_helper::test]
    fn core() {
        let account = ModuleReference::Account(1);
        assert_eq!(account.unwrap_account().unwrap(), 1);
        assert!(account.unwrap_native().is_err());
        assert!(account.unwrap_adapter().is_err());
        assert!(account.unwrap_app().is_err());
        assert!(account.unwrap_standalone().is_err());
        assert!(account.unwrap_service().is_err());
    }

    #[coverage_helper::test]
    fn native() {
        let native = ModuleReference::Native(Addr::unchecked("addr"));
        assert!(native.unwrap_account().is_err());
        assert_eq!(native.unwrap_native().unwrap(), Addr::unchecked("addr"));
        assert!(native.unwrap_adapter().is_err());
        assert!(native.unwrap_app().is_err());
        assert!(native.unwrap_standalone().is_err());
        assert!(native.unwrap_service().is_err());
    }

    #[coverage_helper::test]
    fn service() {
        let service = ModuleReference::Service(Addr::unchecked("addr"));
        assert!(service.unwrap_account().is_err());
        assert!(service.unwrap_native().is_err());
        assert!(service.unwrap_adapter().is_err());
        assert!(service.unwrap_app().is_err());
        assert!(service.unwrap_standalone().is_err());
        assert_eq!(service.unwrap_service().unwrap(), Addr::unchecked("addr"));
    }

    #[coverage_helper::test]
    fn adapter() {
        let adapter = ModuleReference::Adapter(Addr::unchecked("addr"));
        assert!(adapter.unwrap_account().is_err());
        assert!(adapter.unwrap_native().is_err());
        assert_eq!(adapter.unwrap_adapter().unwrap(), Addr::unchecked("addr"));
        assert!(adapter.unwrap_app().is_err());
        assert!(adapter.unwrap_standalone().is_err());
        assert!(adapter.unwrap_service().is_err());
    }

    #[coverage_helper::test]
    fn app() {
        let app = ModuleReference::App(1);
        assert!(app.unwrap_account().is_err());
        assert!(app.unwrap_native().is_err());
        assert!(app.unwrap_adapter().is_err());
        assert_eq!(app.unwrap_app().unwrap(), 1);
        assert!(app.unwrap_standalone().is_err());
        assert!(app.unwrap_service().is_err());
    }

    #[coverage_helper::test]
    fn standalone() {
        let standalone = ModuleReference::Standalone(1);
        assert!(standalone.unwrap_account().is_err());
        assert!(standalone.unwrap_native().is_err());
        assert!(standalone.unwrap_adapter().is_err());
        assert!(standalone.unwrap_app().is_err());
        assert_eq!(standalone.unwrap_standalone().unwrap(), 1);
        assert!(standalone.unwrap_service().is_err());
    }

    #[coverage_helper::test]
    fn unwrap_addr() {
        let native = ModuleReference::Native(Addr::unchecked("addr"));
        assert_eq!(native.unwrap_addr().unwrap(), Addr::unchecked("addr"));
        let api = ModuleReference::Adapter(Addr::unchecked("addr"));
        assert_eq!(api.unwrap_addr().unwrap(), Addr::unchecked("addr"));
        let service = ModuleReference::Service(Addr::unchecked("addr"));
        assert_eq!(service.unwrap_addr().unwrap(), Addr::unchecked("addr"));

        let account = ModuleReference::Account(1);
        assert!(account.unwrap_addr().is_err());
    }

    #[coverage_helper::test]
    fn test_validate_happy_path() {
        let deps = mock_dependencies();

        let native = ModuleReference::Native(deps.api.addr_make("addr"));
        assert_that!(native.validate(deps.as_ref())).is_ok();

        let api = ModuleReference::Adapter(deps.api.addr_make("addr"));
        assert_that!(api.validate(deps.as_ref())).is_ok();

        let service = ModuleReference::Service(deps.api.addr_make("addr"));
        assert_that!(service.validate(deps.as_ref())).is_ok();

        let account = ModuleReference::Account(1);
        assert_that!(account.validate(deps.as_ref())).is_ok();

        let app = ModuleReference::App(1);
        assert_that!(app.validate(deps.as_ref())).is_ok();

        let standalone = ModuleReference::Standalone(1);
        assert_that!(standalone.validate(deps.as_ref())).is_ok();
    }

    #[coverage_helper::test]
    fn test_validate_bad_address() {
        let deps = mock_dependencies();

        let native = ModuleReference::Native(Addr::unchecked(""));
        assert_that!(native.validate(deps.as_ref())).is_err();

        let api = ModuleReference::Adapter(Addr::unchecked("abcde"));
        assert_that!(api.validate(deps.as_ref())).is_err();

        let service = ModuleReference::Service(Addr::unchecked("non_bech"));
        assert_that!(service.validate(deps.as_ref())).is_err();
    }
}
