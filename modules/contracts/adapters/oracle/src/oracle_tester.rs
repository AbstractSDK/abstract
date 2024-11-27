use crate::{interface::OracleAdapter, ORACLE_ADAPTER_ID};
use abstract_adapter::abstract_interface::{AdapterDeployer, DeployStrategy, RegistryExecFns};
use abstract_adapter::std::objects::module::{ModuleInfo, ModuleVersion};
use abstract_client::{AbstractClient, Environment};

use abstract_oracle_standard::msg::{OracleQueryMsgFns, PriceResponse};
use cw_orch::{environment::MutCwEnv, prelude::*};

use cw_orch::anyhow;

pub trait MockOracle<Chain: MutCwEnv> {
    const MAX_AGE: u64;

    /// Name of the oracle
    fn name(&self) -> String;

    /// First asset
    fn price_source_key(&self) -> String;

    /// Ans setup for this oracle
    /// For instance, for Pyth, we just register pyth Contract Entry inside ans
    fn ans_setup(&self, abstr_deployment: &AbstractClient<Chain>) -> anyhow::Result<()>;
}

pub struct OracleTester<Chain: MutCwEnv, Oracle: MockOracle<Chain>> {
    pub abstr_deployment: AbstractClient<Chain>,
    pub oracle_adapter: OracleAdapter<Chain>,
    pub oracle: Oracle,
}

impl<Chain: MutCwEnv, Oracle: MockOracle<Chain>> OracleTester<Chain, Oracle> {
    /// Used to test new code
    pub fn new(abstr_deployment: AbstractClient<Chain>, oracle: Oracle) -> anyhow::Result<Self> {
        // Re-register oracle adapter, to make sure it's latest
        let _ = abstr_deployment
            .registry()
            .remove_module(ModuleInfo::from_id(
                ORACLE_ADAPTER_ID,
                ModuleVersion::Version(crate::contract::CONTRACT_VERSION.to_owned()),
            )?);

        let oracle_adapter = OracleAdapter::new(abstr_deployment.environment());
        oracle_adapter.deploy(
            crate::contract::CONTRACT_VERSION.parse()?,
            Empty {},
            DeployStrategy::Force,
        )?;

        oracle.ans_setup(&abstr_deployment)?;

        Ok(Self {
            abstr_deployment,
            oracle_adapter,
            oracle,
        })
    }

    /// Used to test on-chain code
    pub fn new_live(
        abstr_deployment: AbstractClient<Chain>,
        oracle: Oracle,
    ) -> anyhow::Result<Self> {
        let account = abstr_deployment.account_builder().build()?;
        let oracle_adapter =
            account.install_adapter::<crate::interface::OracleAdapter<Chain>>(&[])?;

        Ok(Self {
            abstr_deployment,
            oracle_adapter: oracle_adapter.module()?,
            oracle,
        })
    }

    pub fn test_price(&self) -> anyhow::Result<PriceResponse> {
        // Get the price associated with the ID
        self.oracle_adapter
            .price(
                Oracle::MAX_AGE,
                self.oracle.name(),
                self.oracle.price_source_key(),
            )
            .map_err(Into::into)
    }
}
