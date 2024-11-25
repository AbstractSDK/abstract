use crate::ORACLE_ADAPTER_ID;
use abstract_adapter::sdk::{
    features::{AccountIdentification, Dependencies, ModuleIdentification},
    AbstractSdkResult, AdapterInterface,
};
use abstract_adapter::traits::AbstractNameService;
use abstract_oracle_standard::msg::{OracleName, Seconds};
use abstract_oracle_standard::msg::{OracleQueryMsg, PriceResponse};
use cosmwasm_std::Deps;

// API for Abstract SDK users
/// Interact with the dex adapter in your module.
pub trait OracleInterface:
    AccountIdentification + Dependencies + ModuleIdentification + AbstractNameService
{
    /// Construct a new dex interface.
    fn oracle<'a>(&'a self, deps: Deps<'a>, name: OracleName) -> Oracle<Self> {
        Oracle {
            base: self,
            deps,
            name,
        }
    }
}

impl<T: AccountIdentification + Dependencies + ModuleIdentification + AbstractNameService>
    OracleInterface for T
{
}

#[derive(Clone)]
pub struct Oracle<'a, T: OracleInterface> {
    pub(crate) base: &'a T,
    pub(crate) name: OracleName,
    pub(crate) deps: Deps<'a>,
}

impl<'a, T: OracleInterface> Oracle<'a, T> {
    /// returns DEX name
    pub fn oracle_name(&self) -> OracleName {
        self.name.clone()
    }

    /// Query a price from the oracle
    pub fn price(
        &self,
        price_source_key: String,
        max_age: Seconds,
    ) -> AbstractSdkResult<PriceResponse> {
        let adapters = self.base.adapters(self.deps);

        adapters.query(
            ORACLE_ADAPTER_ID,
            OracleQueryMsg::Price {
                price_source_key,
                oracle: self.oracle_name(),
                max_age,
            },
        )
    }
}
