use abstract_std::{
    objects::AccountId, ANS_HOST, IBC_CLIENT, IBC_HOST, MODULE_FACTORY, VERSION_CONTROL,
};
use cw_orch::{environment::Environment, prelude::*};

use crate::{
    AbstractInterfaceError, AccountI, AnsHost, IbcClient, IbcHost, ModuleFactory, VersionControl,
};

#[allow(clippy::type_complexity)]
pub fn get_native_contracts<Chain: CwEnv>(
    chain: Chain,
) -> (AnsHost<Chain>, VersionControl<Chain>, ModuleFactory<Chain>)
where
    <Chain as cw_orch::environment::TxHandler>::Response: IndexResponse,
{
    let ans_host = AnsHost::new(ANS_HOST, chain.clone());
    let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
    let module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
    (ans_host, version_control, module_factory)
}

pub fn get_account_contract<Chain: CwEnv>(
    version_control: &VersionControl<Chain>,
    account_id: AccountId,
) -> Result<AccountI<Chain>, AbstractInterfaceError>
where
    <Chain as cw_orch::environment::TxHandler>::Response: IndexResponse,
{
    let chain = version_control.environment().clone();

    let account = AccountI::new_from_id(&account_id, chain.clone());

    let account_base = version_control.get_account(account_id.clone())?;
    account.set_address(account_base.addr());

    Ok(account)
}

pub fn get_ibc_contracts<Chain: CwEnv>(chain: Chain) -> (IbcClient<Chain>, IbcHost<Chain>)
where
    <Chain as cw_orch::environment::TxHandler>::Response: IndexResponse,
{
    let ibc_client = IbcClient::new(IBC_CLIENT, chain.clone());
    let ibc_host = IbcHost::new(IBC_HOST, chain);

    (ibc_client, ibc_host)
}
