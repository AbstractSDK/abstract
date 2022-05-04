use cosmwasm_std::{Deps, DepsMut, Empty, MessageInfo, Response, StdResult};
use cw_controllers::AdminResponse;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::pandora_dapp::query::DappStateResponse;

use super::query::TradersResponse;

pub trait CustomMsg: Clone + std::fmt::Debug + PartialEq + JsonSchema {}

// Allow use of cosmwasm-std::Empty
impl CustomMsg for Empty {}

/// TODO: How can we properly generify the Result?
///  C is not known at compiletime... and we don't have access to Errors
// pub type DappResult<C> = Result<Response<C>, dyn ToString /* DappError */>;

pub trait Dapp<T, C>: DappExecute<T, C> + DappQuery<T>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
}

/// Declaration of all ExecuteMsg the pandora-dapp-base supports
pub trait DappExecute<T, C>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
    type Err: ToString;

    fn update_config(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        proxy_address: Option<String>,
    ) -> Result<Response<C>, Self::Err>;

    fn update_traders(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    ) -> Result<Response<C>, Self::Err>;

    fn update_admin(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        admin: String,
    ) -> Result<Response<C>, Self::Err>;
}

/// This is where all the "interfaces" for the dapp queries are stored. All of these must be implemented.
pub trait DappQuery<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    /// TODO: we could expose these on the top level to avoid returning the entire state
    // fn is_authorized_trader(&self, trader: &Addr);
    // fn assert_authorized_trader(&self, trader: &Addr) -> Result<(), BaseDAppError>;

    fn dapp_config(&self, deps: Deps) -> StdResult<DappStateResponse>;

    fn admin(&self, deps: Deps) -> StdResult<AdminResponse>;

    /// TODO: enable pagination
    fn all_traders(
        &self,
        deps: Deps,
        // start_after: Option<String>,
        // limit: Option<u32>,
    ) -> StdResult<TradersResponse>;
}
