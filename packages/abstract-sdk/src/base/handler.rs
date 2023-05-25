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

/// Accessor trait for an object that wraps an [`AbstractContract`].
pub trait Handler
where
    Self: Sized + 'static,
{
    /// Error type for the contract
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

    /// Returns the contract object.
    fn contract(&self) -> &AbstractContract<Self, Self::Error>;

    /// Returns the cw2 contract version.
    fn stored_version(&self, store: &dyn Storage) -> AbstractSdkResult<ContractVersion> {
        let contract = self.contract();
        contract.version.load(store).map_err(Into::into)
    }

    /// Returns the static contract info.
    fn info(&self) -> (ModuleId, VersionString, ModuleMetadata) {
        let contract = self.contract();
        contract.info
    }

    /// Returns the static contract dependencies.
    fn dependencies(&self) -> &'static [StaticDependency] {
        let contract = self.contract();
        contract.dependencies
    }
    /// Get an execute handler if it exists.
    fn maybe_execute_handler(
        &self,
    ) -> Option<ExecuteHandlerFn<Self, Self::CustomExecMsg, Self::Error>> {
        let contract = self.contract();
        contract.execute_handler
    }
    /// Get an execute handler or return an error.
    fn execute_handler(
        &self,
    ) -> AbstractSdkResult<ExecuteHandlerFn<Self, Self::CustomExecMsg, Self::Error>> {
        let Some(handler) = self.maybe_execute_handler() else {
            return Err(AbstractSdkError::MissingHandler { endpoint: "execution handler".to_string() })
        };
        Ok(handler)
    }

    /// Get a instantiate handler if it exists.
    fn maybe_instantiate_handler(
        &self,
    ) -> Option<InstantiateHandlerFn<Self, Self::CustomInitMsg, Self::Error>> {
        let contract = self.contract();
        contract.instantiate_handler
    }
    /// Get an instantiate handler or return an error.
    fn instantiate_handler(
        &self,
    ) -> AbstractSdkResult<InstantiateHandlerFn<Self, Self::CustomInitMsg, Self::Error>> {
        let Some(handler) = self.maybe_instantiate_handler() else {
            return Err(AbstractSdkError::MissingHandler { endpoint: "instantiate".to_string() })
        };
        Ok(handler)
    }

    /// Get a query handler if it exists.
    fn maybe_query_handler(
        &self,
    ) -> Option<QueryHandlerFn<Self, Self::CustomQueryMsg, Self::Error>> {
        let contract = self.contract();
        contract.query_handler
    }
    /// Get a query handler or return an error.
    fn query_handler(
        &self,
    ) -> AbstractSdkResult<QueryHandlerFn<Self, Self::CustomQueryMsg, Self::Error>> {
        let Some(handler) = self.maybe_query_handler() else {
            return Err(AbstractSdkError::MissingHandler { endpoint: "query".to_string() })
        };
        Ok(handler)
    }

    /// Get a migrate handler if it exists.
    fn maybe_migrate_handler(
        &self,
    ) -> Option<MigrateHandlerFn<Self, Self::CustomMigrateMsg, Self::Error>> {
        let contract = self.contract();
        contract.migrate_handler
    }
    /// Get a migrate handler or return an error.
    fn migrate_handler(
        &self,
    ) -> AbstractSdkResult<MigrateHandlerFn<Self, Self::CustomMigrateMsg, Self::Error>> {
        let Some(handler) = self.maybe_migrate_handler() else {
            return Err(AbstractSdkError::MissingHandler { endpoint: "migrate".to_string() })
        };
        Ok(handler)
    }

    /// Get a sudo handler if it exists.
    fn maybe_sudo_handler(&self) -> Option<SudoHandlerFn<Self, Self::SudoMsg, Self::Error>> {
        let contract = self.contract();
        contract.sudo_handler
    }
    /// Get a sudo handler or return an error.
    fn sudo_handler(&self) -> AbstractSdkResult<SudoHandlerFn<Self, Self::SudoMsg, Self::Error>> {
        let Some(handler) = self.maybe_sudo_handler() else {
            return Err(AbstractSdkError::MissingHandler { endpoint: "sudo".to_string() })
        };
        Ok(handler)
    }

    /// Get a reply handler if it exists.
    fn maybe_receive_handler(
        &self,
    ) -> Option<ReceiveHandlerFn<Self, Self::ReceiveMsg, Self::Error>> {
        let contract = self.contract();
        contract.receive_handler
    }
    /// Get a receive handler or return an error.
    fn receive_handler(
        &self,
    ) -> AbstractSdkResult<ReceiveHandlerFn<Self, Self::ReceiveMsg, Self::Error>> {
        let Some(handler) = self.maybe_receive_handler() else {
            return Err(AbstractSdkError::MissingHandler { endpoint: "receive".to_string() })
        };
        Ok(handler)
    }
    /// Get an ibc callback handler if it exists.
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
    /// Get a reply handler if it exists.
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
    /// Get a reply handler or return an error.
    fn reply_handler(&self, id: u64) -> AbstractSdkResult<ReplyHandlerFn<Self, Self::Error>> {
        let Some(handler) = self.maybe_reply_handler(id) else {
            return Err(AbstractSdkError::MissingHandler { endpoint: format! {"reply with id {id}"} })
        };
        Ok(handler)
    }
}
