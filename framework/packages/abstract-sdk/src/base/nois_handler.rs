use crate::base::Handler;
use super::contract_base::{
    AbstractContract, NoisCallbackHandlerFn
};
use crate::{
    AbstractSdkError, AbstractSdkResult,
};

pub trait NoisHandler: Handler
    where
        Self: Sized + 'static,
{
    /// Get a nois callback handler if it exists.
    fn maybe_nois_callback_handler(
        &self,
    ) -> Option<NoisCallbackHandlerFn<Self, Self::Error>> {
        let contract = self.contract();
        contract.nois_callback_handler
    }
    /// Get a nois callback handler or return an error.
    fn nois_callback_handler(
        &self,
    ) -> AbstractSdkResult<NoisCallbackHandlerFn<Self, Self::Error>> {
        let Some(handler) = self.maybe_nois_callback_handler() else {
            return Err(AbstractSdkError::MissingHandler { endpoint: "receive".to_string() })
        };
        Ok(handler)
    }
}

impl<Module, Error: From<AbstractSdkError>> AbstractContract<Module, Error>
    where
        Module: NoisHandler,
{
    /// Add nois callback handler to the contract.
    pub const fn with_nois_callback(
        mut self,
        nois_callback_handler: NoisCallbackHandlerFn<Module, Error>,
    ) -> Self {
        self.nois_callback_handler = Some(nois_callback_handler);
        self
    }

}