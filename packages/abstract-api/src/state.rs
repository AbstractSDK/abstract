use std::marker::PhantomData;

use abstract_os::api::ApiInterfaceMsg;

use abstract_os::version_control::Core;
use abstract_sdk::common_module::BASE_STATE_KEY;
use abstract_sdk::memory::Memory;
use abstract_sdk::version_control::verify_os_manager;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage};
use cw2::{ContractVersion, CONTRACT};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::ApiError;

pub const TRADER_NAMESPACE: &str = "traders";

/// The state variables for our ApiContract.
pub struct ApiContract<'a, T: Serialize + DeserializeOwned> {
    /// Map ProxyAddr -> WhitelistedTraders
    pub traders: Map<'a, Addr, Vec<Addr>>,
    /// Every DApp should use the provided memory contract for token/contract address resolution
    pub base_state: Item<'a, ApiState>,
    /// Stores the API version
    pub version: Item<'a, ContractVersion>,
    ///
    pub request_destination: Option<Addr>,

    _phantom_data: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> Default for ApiContract<'static, T> {
    fn default() -> Self {
        Self::new(BASE_STATE_KEY, TRADER_NAMESPACE)
    }
}

/// Constructor
impl<'a, T: Serialize + DeserializeOwned> ApiContract<'a, T> {
    fn new(base_state_key: &'a str, traders_namespace: &'a str) -> Self {
        Self {
            version: CONTRACT,
            base_state: Item::new(base_state_key),
            traders: Map::new(traders_namespace),
            request_destination: None,
            _phantom_data: PhantomData,
        }
    }

    /// Takes request and parses it to a verified
    pub fn handle_request(
        &mut self,
        deps: &mut DepsMut,
        env: Env,
        info: &MessageInfo,
        msg: ApiInterfaceMsg<T>,
    ) -> Result<ApiInterfaceResponse<T>, ApiError> {
        let sender = &info.sender;
        match msg {
            ApiInterfaceMsg::Request(request) => {
                let proxy = match request.proxy_addr {
                    Some(addr) => {
                        let traders = self.traders.load(deps.storage, addr.clone())?;
                        if traders.contains(sender) {
                            addr
                        } else {
                            self.verify_sender_is_manager(deps.as_ref(), sender)
                                .map_err(|_| ApiError::UnauthorizedTraderApiRequest {})?
                                .proxy
                        }
                    }
                    None => {
                        self.verify_sender_is_manager(deps.as_ref(), sender)
                            .map_err(|_| ApiError::UnauthorizedApiRequest {})?
                            .proxy
                    }
                };
                self.request_destination = Some(proxy);
                Ok(ApiInterfaceResponse::ProcessRequest(request.request))
            }
            ApiInterfaceMsg::Configure(exec_msg) => Ok(ApiInterfaceResponse::ExecResponse(
                self.execute(deps, env, info.clone(), exec_msg)?,
            )),
        }
    }
    pub fn verify_sender_is_manager(
        &self,
        deps: Deps,
        maybe_manager: &Addr,
    ) -> Result<Core, ApiError> {
        let version_control_addr = self.base_state.load(deps.storage)?.version_control;
        let core = verify_os_manager(&deps.querier, maybe_manager, &version_control_addr)?;
        Ok(core)
    }

    pub fn state(&self, store: &dyn Storage) -> StdResult<ApiState> {
        self.base_state.load(store)
    }

    pub fn version(&self, store: &dyn Storage) -> StdResult<ContractVersion> {
        self.version.load(store)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ApiInterfaceResponse<T> {
    ProcessRequest(T),
    ExecResponse(Response),
}
/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApiState {
    /// Used to verify requests
    pub version_control: Addr,
    /// Memory contract struct (address)
    pub memory: Memory,
}
