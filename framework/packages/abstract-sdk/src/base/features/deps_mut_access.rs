use cosmwasm_std::{DepsMut, Env, MessageInfo};

use super::DepsAccess;

pub trait DepsMutAccess: DepsAccess {
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> DepsMut<'b>;
}

impl DepsMutAccess for (DepsMut<'_>, Env, MessageInfo) {
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> cosmwasm_std::DepsMut<'b> {
        self.0.branch()
    }
}

impl DepsMutAccess for (DepsMut<'_>, Env) {
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> cosmwasm_std::DepsMut<'b> {
        self.0.branch()
    }
}
