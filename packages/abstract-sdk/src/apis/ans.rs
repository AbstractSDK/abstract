#![allow(unused)]

use abstract_os::objects::ans_host::AnsHost;
use cosmwasm_std::{Deps, StdResult};

use crate::ans_resolve::Resolve;

use super::AbstractNameSystem;
/// Perform queries on the Abstract Name System.
pub trait AnsInterface: AbstractNameSystem {
    fn ans<'a>(&'a self, deps: Deps<'a>) -> Ans<Self> {
        Ans {
            base: self,
            deps,
            host: self.ans_host(deps).unwrap(),
        }
    }
}

impl<T> AnsInterface for T where T: AbstractNameSystem {}

#[derive(Clone)]
pub struct Ans<'a, T: AnsInterface> {
    base: &'a T,
    deps: Deps<'a>,
    host: AnsHost,
}

impl<'a, T: AnsInterface> Ans<'a, T> {
    pub fn query<R: Resolve>(&self, entry: &R) -> StdResult<R::Output> {
        entry.resolve(&self.deps.querier, &self.host)
    }
    pub fn host(&self) -> &AnsHost {
        &self.host
    }
}
