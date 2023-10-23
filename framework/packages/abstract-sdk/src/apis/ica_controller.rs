use crate::{
    features::AbstractRegistryAccess, AbstractSdkError, AbstractSdkResult, ModuleInterface,
    ModuleRegistryInterface,
};
use abstract_core::{
    adapter::AdapterRequestMsg,
    objects::module::{ModuleId, ModuleInfo, ModuleVersion},
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    instantiate2_address, to_binary, wasm_execute, Addr, Binary, CodeInfoResponse,
    ContractInfoResponse, CosmosMsg, Deps, DepsMut, Empty, Env, SubMsg, WasmMsg, Coin,
};
use cw_ica_controller::types::msg::{InstantiateMsg, ExecuteMsg};
use cw_storage_plus::{Item, Map};
use serde::{de::DeserializeOwned, Serialize};

pub const ICA_MODULE_NAME: &str = "ibc:ica-controller";
pub const ICA_CONTROLLER_REPLY_ID: u64 = 428880;

#[cw_serde]
pub struct IcaState {
    sequence: u64,
}

/// Interact with an ICA controller
pub trait IcaController: ModuleRegistryInterface {
    /**
        API for creating and interacting with an ICA controller.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let module = MockModule::new();
        # let deps = mock_dependencies();

        let ica_client: IcaClient<MockModule>  = module.ica_client(deps.as_ref());
        ```
    */
    fn ica_client<'a>(&'a self, deps: Deps<'a>) -> IcaClient<Self> {
        IcaClient {
            base: self,
            deps,
            state: Item::new("icast"),
            controllers: Map::new("icact"),
        }
    }
}

impl<T> IcaController for T where T: ModuleRegistryInterface {}

/**
    API for accessing Abstract IcaClient installed on the account.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # let module = MockModule::new();
    # let deps = mock_dependencies();

    let ica_client: IcaClient<MockModule>  = module.ica_client(deps.as_ref());
    ```
*/

pub struct IcaClient<'a, T: IcaController> {
    base: &'a T,
    deps: Deps<'a>,
    state: Item<'a, IcaState>,
    controllers: Map<'a, u64, Addr>,
}

impl<'a, T: IcaController> IcaClient<'a, T> {
    /// Creates an interchain account controller.
    /// Returns the creation message, address and sequence number of the controller.
    /// The sequence number is used to retrieve the controller for interactions.
    #[must_use]
    pub fn create_controller(
        &self,
        deps: DepsMut,
        env: Env,
        version: ModuleVersion,
        admin: Option<String>,
        init_msg: InstantiateMsg,
        label: Option<String>,
    ) -> AbstractSdkResult<(CosmosMsg, Addr, u64)> {
        // Instantiate an ICA client
        let registry = self.base.module_registry(self.deps.clone());
        let module = registry.query_module(ModuleInfo::from_id(ICA_MODULE_NAME, version)?)?;

        let creator = deps.api.addr_canonicalize(env.contract.address.as_str())?;
        let ContractInfoResponse { code_id, .. } = deps
            .querier
            .query_wasm_contract_info(env.contract.address)?;
        let CodeInfoResponse { checksum, .. } = deps.querier.query_wasm_code_info(code_id)?;

        let sequence = self.state.load(self.deps.storage).unwrap_or(IcaState {
            sequence: 0,
        }).sequence;

        let path = format!("instance/{sequence}");
        let salt = Binary::from(path.as_bytes());

        let label = label.unwrap_or_else(|| format!("ICA number {} created with Abstract", sequence));

        let address =
            deps.api
                .addr_humanize(&instantiate2_address(&checksum, &creator, &salt).map_err(
                    |e| {
                        AbstractSdkError::Std(cosmwasm_std::StdError::GenericErr {
                            msg: e.to_string(),
                        })
                    },
                )?)?;

        // Instantiate module
        let create_msg =
            WasmMsg::Instantiate2 {
                admin,
                code_id: module.reference.unwrap_standalone()?,
                label,
                msg: to_binary(&init_msg)?,
                funds: vec![],
                salt,
            };

        self.state.save(deps.storage, &IcaState { sequence: sequence + 1 })?;
        self.controllers.save(deps.storage, sequence, &address)?;

        Ok((create_msg.into(), address, sequence))
    }

    fn execute_on(&self, controller_seq_number: u64, msg: &ExecuteMsg, funds: Vec<Coin>) -> AbstractSdkResult<CosmosMsg> {
        let controller = self.controllers.load(self.deps.storage, controller_seq_number)?;
        let msg = WasmMsg::Execute { contract_addr: controller.into(), msg: to_binary(&msg)?, funds };
        Ok(msg.into())
    }
}
