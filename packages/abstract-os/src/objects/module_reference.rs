use cosmwasm_std::{Addr, StdError, StdResult};

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum ModuleReference {
    /// Core Abstract Contracts
    Core(u64),
    /// Native Abstract Contracts
    Native(Addr),
    /// Installable extensions
    Extension(Addr),
    /// Installable apps
    App(u64),
    /// A stand-alone contract
    Standalone(u64),
}

impl ModuleReference {
    pub fn unwrap_core(&self) -> StdResult<u64> {
        match self {
            ModuleReference::Core(v) => Ok(*v),
            _ => Err(StdError::generic_err("Not a core module.")),
        }
    }

    pub fn unwrap_native(&self) -> StdResult<Addr> {
        match self {
            ModuleReference::Native(addr) => Ok(addr.clone()),
            _ => Err(StdError::generic_err("Not a native module.")),
        }
    }

    pub fn unwrap_extension(&self) -> StdResult<Addr> {
        match self {
            ModuleReference::Extension(addr) => Ok(addr.clone()),
            _ => Err(StdError::generic_err("Not an extension module.")),
        }
    }

    pub fn unwrap_app(&self) -> StdResult<u64> {
        match self {
            ModuleReference::App(v) => Ok(*v),
            _ => Err(StdError::generic_err("Not an app module.")),
        }
    }

    pub fn unwrap_standalone(&self) -> StdResult<u64> {
        match self {
            ModuleReference::Standalone(v) => Ok(*v),
            _ => Err(StdError::generic_err("Not a standalone module.")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn core() {
        let core = ModuleReference::Core(1);
        assert_eq!(core.unwrap_core().unwrap(), 1);
        assert!(core.unwrap_native().is_err());
        assert!(core.unwrap_extension().is_err());
        assert!(core.unwrap_app().is_err());
        assert!(core.unwrap_standalone().is_err());
    }
    #[test]
    fn native() {
        let native = ModuleReference::Native(Addr::unchecked("addr"));
        assert!(native.unwrap_core().is_err());
        assert_eq!(native.unwrap_native().unwrap(), Addr::unchecked("addr"));
        assert!(native.unwrap_extension().is_err());
        assert!(native.unwrap_app().is_err());
        assert!(native.unwrap_standalone().is_err());
    }

    #[test]
    fn extension() {
        let extension = ModuleReference::Extension(Addr::unchecked("addr"));
        assert!(extension.unwrap_core().is_err());
        assert!(extension.unwrap_native().is_err());
        assert_eq!(
            extension.unwrap_extension().unwrap(),
            Addr::unchecked("addr")
        );
        assert!(extension.unwrap_app().is_err());
        assert!(extension.unwrap_standalone().is_err());
    }

    #[test]
    fn app() {
        let app = ModuleReference::App(1);
        assert!(app.unwrap_core().is_err());
        assert!(app.unwrap_native().is_err());
        assert!(app.unwrap_extension().is_err());
        assert_eq!(app.unwrap_app().unwrap(), 1);
        assert!(app.unwrap_standalone().is_err());
    }

    #[test]
    fn standalone() {
        let standalone = ModuleReference::Standalone(1);
        assert!(standalone.unwrap_core().is_err());
        assert!(standalone.unwrap_native().is_err());
        assert!(standalone.unwrap_extension().is_err());
        assert!(standalone.unwrap_app().is_err());
        assert_eq!(standalone.unwrap_standalone().unwrap(), 1);
    }
}
