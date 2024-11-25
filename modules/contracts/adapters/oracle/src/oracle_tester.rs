use crate::{interface::OracleAdapter, ORACLE_ADAPTER_ID};
use abstract_adapter::abstract_interface::{AdapterDeployer, DeployStrategy, RegistryExecFns};
use abstract_adapter::std::objects::module::{ModuleInfo, ModuleVersion};
use abstract_client::{AbstractClient, Environment};

use abstract_oracle_standard::msg::OracleQueryMsgFns;
use cw_orch::{environment::MutCwEnv, prelude::*};

use cw_orch::anyhow;

pub trait MockOracle {
    const MAX_AGE: u64;

    /// Name of the dex
    fn name(&self) -> String;

    /// First asset
    fn price_source_key(&self) -> String;

    /// Ans setup for this oracle
    /// For instance, for Pyth, we just register pyth Contract Entry inside ans
    fn ans_setup(&self) -> anyhow::Result<()>;
}

pub struct OracleTester<Chain: MutCwEnv, Oracle: MockOracle> {
    pub abstr_deployment: AbstractClient<Chain>,
    pub oracle_adapter: OracleAdapter<Chain>,
    pub oracle: Oracle,
}

impl<Chain: MutCwEnv, Oracle: MockOracle> OracleTester<Chain, Oracle> {
    pub fn new(abstr_deployment: AbstractClient<Chain>, oracle: Oracle) -> anyhow::Result<Self> {
        // Re-register dex, to make sure it's latest
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

        oracle.ans_setup()?;

        Ok(Self {
            abstr_deployment,
            oracle_adapter,
            oracle,
        })
    }

    pub fn test_price(&self) -> anyhow::Result<()> {
        // Get the price associated with the ID
        let _price = self.oracle_adapter.price(
            Oracle::MAX_AGE,
            self.oracle.name(),
            self.oracle.price_source_key(),
        )?;

        // Price should just exist, not using it here
        Ok(())
    }
}
