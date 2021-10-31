use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

use cosmwasm_std::{attr, Addr, MessageInfo, Response, StdError, StdResult, Storage};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TraderResponse {
    pub trader: Option<String>,
}

/// Errors returned from Trader
#[derive(Error, Debug, PartialEq)]
pub enum TraderError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Caller is not authorized trader")]
    NotTrader {},
}

// state/logic
pub struct Trader<'a>(Item<'a, Option<Addr>>);

// this is the core business logic we expose
impl<'a> Trader<'a> {
    pub const fn new(namespace: &'a str) -> Self {
        Trader(Item::new(namespace))
    }

    pub fn set(&self, storage: &mut dyn Storage, trader: Option<Addr>) -> StdResult<()> {
        self.0.save(storage, &trader)
    }

    pub fn get(&self, storage: &dyn Storage) -> StdResult<Option<Addr>> {
        self.0.load(storage)
    }

    /// Returns Ok(true) if this is an trader, Ok(false) if not and an Error if
    /// we hit an error with Api or Storage usage
    pub fn is_trader(&self, storage: &dyn Storage, caller: &Addr) -> StdResult<bool> {
        match self.0.load(storage)? {
            Some(owner) => Ok(caller == &owner),
            None => Ok(false),
        }
    }

    /// Like is_trader but returns TraderError::NotTrader if not trader.
    /// Helper for a nice one-line auth check.
    pub fn assert_trader(&self, storage: &dyn Storage, caller: &Addr) -> Result<(), TraderError> {
        if !self.is_trader(storage, caller)? {
            Err(TraderError::NotTrader {})
        } else {
            Ok(())
        }
    }

    pub fn execute_update_trader<C>(
        &self,
        storage: &mut dyn Storage,
        info: MessageInfo,
        new_trader: Option<Addr>,
    ) -> Result<Response<C>, TraderError>
    where
        C: Clone + fmt::Debug + PartialEq + JsonSchema,
    {
        let trader_str = match new_trader.as_ref() {
            Some(trader) => trader.to_string(),
            None => "None".to_string(),
        };
        let attributes = vec![
            attr("action", "update_trader"),
            attr("trader", trader_str),
            attr("sender", info.sender),
        ];

        self.set(storage, new_trader)?;

        Ok(Response::new().add_attributes(attributes))
    }

    pub fn query_trader(&self, storage: &dyn Storage) -> StdResult<TraderResponse> {
        let trader = self.get(storage)?.map(String::from);
        Ok(TraderResponse { trader })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::Empty;

    #[test]
    fn set_and_get_trader() {
        let mut deps = mock_dependencies(&[]);
        let control = Trader::new("foo");

        // initialize and check
        let trader = Some(Addr::unchecked("trader"));
        control.set(&mut deps.storage, trader.clone()).unwrap();
        let got = control.get(&deps.storage).unwrap();
        assert_eq!(trader, got);

        // clear it and check
        control.set(&mut deps.storage, None).unwrap();
        let got = control.get(&deps.storage).unwrap();
        assert_eq!(None, got);
    }

    #[test]
    fn trader_checks() {
        let mut deps = mock_dependencies(&[]);

        let control = Trader::new("foo");
        let owner = Addr::unchecked("big boss");
        let imposter = Addr::unchecked("imposter");

        // ensure checks proper with owner set
        control.set(&mut deps.storage, Some(owner.clone())).unwrap();
        assert!(control.is_trader(&deps.storage, &owner).unwrap());
        assert!(!(control.is_trader(&deps.storage, &imposter).unwrap()));
        control.assert_trader(&deps.storage, &owner).unwrap();
        let err = control.assert_trader(&deps.storage, &imposter).unwrap_err();
        assert_eq!(TraderError::NotTrader {}, err);

        // ensure checks proper with owner None
        control.set(&mut deps.storage, None).unwrap();
        assert!(!(control.is_trader(&deps.storage, &owner).unwrap()));
        assert!(!(control.is_trader(&deps.storage, &imposter).unwrap()));
        let err = control.assert_trader(&deps.storage, &owner).unwrap_err();
        assert_eq!(TraderError::NotTrader {}, err);
        let err = control.assert_trader(&deps.storage, &imposter).unwrap_err();
        assert_eq!(TraderError::NotTrader {}, err);
    }

    #[test]
    fn test_execute_query() {
        let mut deps = mock_dependencies(&[]);

        // initial setup
        let control = Trader::new("foo");
        let owner = Addr::unchecked("big boss");
        let friend = Addr::unchecked("buddy");
        control.set(&mut deps.storage, Some(owner.clone())).unwrap();

        // query shows results
        let res = control.query_trader(&deps.storage).unwrap();
        assert_eq!(Some(owner.to_string()), res.trader);

        let new_trader = Some(friend.clone());
        // owner can update
        let info = mock_info(owner.as_ref(), &[]);
        let res = control
            .execute_update_trader::<Empty>(&mut deps.storage, info, new_trader)
            .unwrap();
        assert_eq!(0, res.messages.len());

        // query shows results
        let res = control.query_trader(&deps.storage).unwrap();
        assert_eq!(Some(friend.to_string()), res.trader);
    }
}
