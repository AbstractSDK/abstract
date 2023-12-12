use cosmwasm_std::{Api, Deps, DepsMut, Env, MessageInfo};

pub trait DepsAccess {
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> DepsMut<'b>;
    fn deps<'a: 'b, 'b>(&'a self) -> Deps<'b>;

    fn env(&self) -> Env;
    fn message_info(&self) -> MessageInfo;

    fn api<'a: 'b, 'b>(&'a self) -> &'b dyn Api {
        self.deps().api
    }
}

pub enum DepsType<'a> {
    User(DepsMut<'a>, Env, MessageInfo),
    Query(Deps<'a>, Env),
    Blockchain(DepsMut<'a>, Env),
}

impl DepsAccess for DepsType<'_> {
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> cosmwasm_std::DepsMut<'b> {
        match self {
            DepsType::User(deps, _, _) => deps.branch(),
            DepsType::Blockchain(deps, _) => deps.branch(),
            DepsType::Query(_, _) => unimplemented!(),
        }
    }

    fn deps<'a: 'b, 'b>(&'a self) -> cosmwasm_std::Deps<'b> {
        match self {
            DepsType::User(deps, _, _) => deps.as_ref(),
            DepsType::Blockchain(deps, _) => deps.as_ref(),
            DepsType::Query(deps, _) => *deps,
        }
    }

    fn env(&self) -> Env {
        match self {
            DepsType::User(_, env, _) => env.clone(),
            DepsType::Blockchain(_, env) => env.clone(),
            DepsType::Query(_, env) => env.clone(),
        }
    }

    fn message_info(&self) -> MessageInfo {
        match self {
            DepsType::User(_, _, info) => info.clone(),
            DepsType::Blockchain(_, _) => unimplemented!(),
            DepsType::Query(_, _) => unimplemented!(),
        }
    }
}

impl<'a> From<(DepsMut<'a>, Env, MessageInfo)> for DepsType<'a> {
    fn from(value: (DepsMut<'a>, Env, MessageInfo)) -> Self {
        DepsType::User(value.0, value.1, value.2)
    }
}

impl<'a> From<(DepsMut<'a>, Env)> for DepsType<'a> {
    fn from(value: (DepsMut<'a>, Env)) -> Self {
        DepsType::Blockchain(value.0, value.1)
    }
}

impl<'a> From<(Deps<'a>, Env)> for DepsType<'a> {
    fn from(value: (Deps<'a>, Env)) -> Self {
        DepsType::Query(value.0, value.1)
    }
}
