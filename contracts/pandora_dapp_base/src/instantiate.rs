use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdResult};
// use pandora_os::pandora_dapp::traits::DappResult;
use serde::de::DeserializeOwned;
use serde::Serialize;

use pandora_os::native::memory::item::Memory;
use pandora_os::pandora_dapp::msg::DappInstantiateMsg;
use pandora_os::pandora_dapp::CustomMsg;

use crate::state::{DappContract, DappState};

// use cw2::set_contract_version;
// use pandora_dapp::{CustomMsg, DappExecute};

// // version info for migration info
// const CONTRACT_NAME: &str = "crates.io:cw721-base";
// const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

impl<'a, T, C> DappContract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
    pub fn instantiate(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: DappInstantiateMsg,
    ) -> StdResult<Response<C>> {
        // TODO we could use versioncontrol and pass in the name and the version of the contract
        let memory = Memory {
            address: deps.api.addr_validate(&msg.memory_address)?,
        };

        // Base state
        let state = DappState {
            // Proxy gets set by manager after Init
            proxy_address: Addr::unchecked(""),
            traders: vec![],
            memory,
        };

        self.base_state.save(deps.storage, &state)?;
        self.admin.set(deps, Some(info.sender))?;

        Ok(Response::default())
    }
}
