use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;

use pandora_os::pandora_dapp::msg::DappExecuteMsg;
use pandora_os::pandora_dapp::{CustomMsg, DappExecute};

use crate::error::DappError;
use crate::state::DappContract;
use crate::DappResult;

impl<'a, T, C> DappContract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
    pub fn execute(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        message: DappExecuteMsg,
    ) -> DappResult<C> {
        match message {
            DappExecuteMsg::UpdateConfig { proxy_address } => {
                self.update_config(deps, info, proxy_address)
            }
            DappExecuteMsg::UpdateTraders { to_add, to_remove } => {
                self.update_traders(deps, info, to_add, to_remove)
            }
            DappExecuteMsg::SetAdmin { admin } => self.update_admin(deps, info, admin),
        }
    }
}

impl<'a, T, C> DappExecute<T, C> for DappContract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
    type Err = DappError;

    fn update_config(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        proxy_address: Option<String>,
    ) -> DappResult<C> {
        self._update_config(deps, info, proxy_address)?;

        Ok(Response::default().add_attribute("action", "update_config"))
    }

    fn update_traders(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    ) -> DappResult<C> {
        self._update_traders(deps, info, to_add, to_remove)?;

        Ok(Response::default().add_attribute("action", "update_traders"))
    }

    fn update_admin(&self, deps: DepsMut, info: MessageInfo, admin: String) -> DappResult<C> {
        let (prev_admin, new_admin) = self._update_admin(deps, info, admin)?;

        Ok(Response::default()
            .add_attribute("action", "update_admin")
            .add_attribute("previous_admin", prev_admin)
            .add_attribute("new_admin", new_admin))
    }
}

impl<'a, T, C> DappContract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
    pub fn _update_config(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        proxy_address: Option<String>,
    ) -> Result<(), DappError> {
        // Only the admin should be able to call this
        self.admin.assert_admin(deps.as_ref(), &info.sender)?;

        let mut state = self.base_state.load(deps.storage)?;

        if let Some(proxy_address) = proxy_address {
            state.proxy_address = deps.api.addr_validate(proxy_address.as_str())?;
        }

        self.base_state.save(deps.storage, &state)?;
        Ok(())
    }

    pub fn _update_traders(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    ) -> Result<(), DappError> {
        // Only the admin should be able to call this
        self.admin.assert_admin(deps.as_ref(), &info.sender)?;

        let mut state = self.base_state.load(deps.storage)?;

        // Handle the addition of traders
        if let Some(to_add) = to_add {
            for trader in to_add {
                let trader_addr = deps.api.addr_validate(trader.as_str())?;
                if !state.traders.contains(&trader_addr) {
                    state.traders.push(trader_addr);
                } else {
                    return Err(DappError::TraderAlreadyPresent { trader });
                }
            }
        }

        // Handling the removal of traders
        if let Some(to_remove) = to_remove {
            for trader in to_remove {
                let trader_addr = deps.api.addr_validate(trader.as_str())?;
                if let Some(trader_pos) = state.traders.iter().position(|a| a == &trader_addr) {
                    state.traders.remove(trader_pos);
                } else {
                    return Err(DappError::TraderNotPresent { trader });
                }
            }
        }

        // at least one trader is always required
        if state.traders.is_empty() {
            return Err(DappError::TraderRequired {});
        }

        self.base_state.save(deps.storage, &state)?;
        Ok(())
    }

    pub fn _update_admin(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        admin: String,
    ) -> Result<(Addr, Addr), DappError> {
        self.admin.assert_admin(deps.as_ref(), &info.sender)?;

        let prev_admin = self.admin.get(deps.as_ref())?.unwrap();

        let new_admin = deps.api.addr_validate(&admin)?;
        self.admin
            .execute_update_admin::<C>(deps, info, Some(new_admin.clone()))?;

        Ok((prev_admin, new_admin))
    }
}
