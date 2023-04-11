use super::contract_base::{
    AbstractContract, ExecuteHandlerFn, IbcCallbackHandlerFn, InstantiateHandlerFn,
    MigrateHandlerFn, QueryHandlerFn, ReceiveHandlerFn, SudoHandlerFn,
};
use crate::{
    base::{
        contract_base::{ModuleId, ModuleMetadata, VersionString},
        ReplyHandlerFn,
    },
    AbstractSdkError, AbstractSdkResult,
};
use abstract_core::objects::dependency::StaticDependency;
use cosmwasm_std::Storage;
use cw2::ContractVersion;

pub trait Handler
where
    Self: Sized + 'static,
{
    type Error: From<AbstractSdkError>;
    /// Custom init message for the contract
    type CustomInitMsg;
    /// Custom execute message for the contract
    type CustomExecMsg;
    /// Custom query message for the contract
    type CustomQueryMsg;
    /// Custom migrate message for the contract
    type CustomMigrateMsg;
    /// Receive message for the contract
    type ReceiveMsg;
    /// Sudo message for the contract
    type SudoMsg;

    fn contract(&self) -> &AbstractContract<Self, Self::Error>;

    fn stored_version(&self, store: &dyn Storage) -> AbstractSdkResult<ContractVersion> {
        let contract = self.contract();
        contract.version.load(store).map_err(Into::into)
    }

    fn info(&self) -> (ModuleId, VersionString, ModuleMetadata) {
        let contract = self.contract();
        contract.info
    }

    fn dependencies(&self) -> &'static [StaticDependency] {
        let contract = self.contract();
        contract.dependencies
    }
    // Execute
    fn maybe_execute_handler(
        &self,
    ) -> Option<ExecuteHandlerFn<Self, Self::CustomExecMsg, Self::Error>> {
        let contract = self.contract();
        contract.execute_handler
    }
    fn execute_handler(
        &self,
    ) -> AbstractSdkResult<ExecuteHandlerFn<Self, Self::CustomExecMsg, Self::Error>> {
        let Some(handler) = self.maybe_execute_handler() else {
            return Err(AbstractSdkError::MissingHandler { endpoint: "execution handler".to_string() })
        };
        Ok(handler)
    }

    // Instantiate
    fn maybe_instantiate_handler(
        &self,
    ) -> Option<InstantiateHandlerFn<Self, Self::CustomInitMsg, Self::Error>> {
        let contract = self.contract();
        contract.instantiate_handler
    }
    fn instantiate_handler(
        &self,
    ) -> AbstractSdkResult<InstantiateHandlerFn<Self, Self::CustomInitMsg, Self::Error>> {
        let Some(handler) = self.maybe_instantiate_handler() else {
            return Err(AbstractSdkError::MissingHandler { endpoint: "instantiate".to_string() })
        };
        Ok(handler)
    }

    // Query
    fn maybe_query_handler(
        &self,
    ) -> Option<QueryHandlerFn<Self, Self::CustomQueryMsg, Self::Error>> {
        let contract = self.contract();
        contract.query_handler
    }
    fn query_handler(
        &self,
    ) -> AbstractSdkResult<QueryHandlerFn<Self, Self::CustomQueryMsg, Self::Error>> {
        let Some(handler) = self.maybe_query_handler() else {
            return Err(AbstractSdkError::MissingHandler { endpoint: "query".to_string() })
        };
        Ok(handler)
    }

    // Migrate
    fn maybe_migrate_handler(
        &self,
    ) -> Option<MigrateHandlerFn<Self, Self::CustomMigrateMsg, Self::Error>> {
        let contract = self.contract();
        contract.migrate_handler
    }
    fn migrate_handler(
        &self,
    ) -> AbstractSdkResult<MigrateHandlerFn<Self, Self::CustomMigrateMsg, Self::Error>> {
        let Some(handler) = self.maybe_migrate_handler() else {
            return Err(AbstractSdkError::MissingHandler { endpoint: "migrate".to_string() })
        };
        Ok(handler)
    }

    // Sudo
    fn maybe_sudo_handler(&self) -> Option<SudoHandlerFn<Self, Self::SudoMsg, Self::Error>> {
        let contract = self.contract();
        contract.sudo_handler
    }
    fn sudo_handler(&self) -> AbstractSdkResult<SudoHandlerFn<Self, Self::SudoMsg, Self::Error>> {
        let Some(handler) = self.maybe_sudo_handler() else {
            return Err(AbstractSdkError::MissingHandler { endpoint: "sudo".to_string() })
        };
        Ok(handler)
    }

    // Receive
    fn maybe_receive_handler(
        &self,
    ) -> Option<ReceiveHandlerFn<Self, Self::ReceiveMsg, Self::Error>> {
        let contract = self.contract();
        contract.receive_handler
    }
    fn receive_handler(
        &self,
    ) -> AbstractSdkResult<ReceiveHandlerFn<Self, Self::ReceiveMsg, Self::Error>> {
        let Some(handler) = self.maybe_receive_handler() else {
            return Err(AbstractSdkError::MissingHandler { endpoint: "receive".to_string() })
        };
        Ok(handler)
    }
    fn maybe_ibc_callback_handler(
        &self,
        id: &str,
    ) -> Option<IbcCallbackHandlerFn<Self, Self::Error>> {
        let contract = self.contract();
        for ibc_callback_handler in contract.ibc_callback_handlers {
            if ibc_callback_handler.0 == id {
                return Some(ibc_callback_handler.1);
            }
        }
        None
    }

    fn maybe_reply_handler(&self, id: u64) -> Option<ReplyHandlerFn<Self, Self::Error>> {
        let contract = self.contract();
        for reply_handlers in contract.reply_handlers {
            for handler in reply_handlers {
                if handler.0 == id {
                    return Some(handler.1);
                }
            }
        }
        None
    }

    fn reply_handler(&self, id: u64) -> AbstractSdkResult<ReplyHandlerFn<Self, Self::Error>> {
        let Some(handler) = self.maybe_reply_handler(id) else {
            return Err(AbstractSdkError::MissingHandler { endpoint: format! {"reply with id {id}"} })
        };
        Ok(handler)
    }
}
