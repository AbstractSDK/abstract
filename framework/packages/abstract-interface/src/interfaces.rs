use abstract_std::{objects::AccountId, ANS_HOST, IBC_CLIENT, IBC_HOST, MODULE_FACTORY, REGISTRY};
use cw_orch::{environment::Environment, prelude::*};

use crate::{
    AbstractInterfaceError, AccountI, AnsHost, IbcClient, IbcHost, ModuleFactory, Registry,
};

#[allow(clippy::type_complexity)]
pub fn get_native_contracts<Chain: CwEnv>(
    chain: Chain,
) -> (AnsHost<Chain>, Registry<Chain>, ModuleFactory<Chain>)
where
    <Chain as cw_orch::environment::TxHandler>::Response: IndexResponse,
{
    let ans_host = AnsHost::new(ANS_HOST, chain.clone());
    let registry = Registry::new(REGISTRY, chain.clone());
    let module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
    (ans_host, registry, module_factory)
}

pub fn get_account_contract<Chain: CwEnv>(
    registry: &Registry<Chain>,
    account_id: AccountId,
) -> Result<AccountI<Chain>, AbstractInterfaceError>
where
    <Chain as cw_orch::environment::TxHandler>::Response: IndexResponse,
{
    let chain = registry.environment().clone();

    let account_interface = AccountI::new_from_id(&account_id, chain.clone());

    let account = registry.get_account(account_id.clone())?;
    account_interface.set_address(account.addr());

    Ok(account_interface)
}
