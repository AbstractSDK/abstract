use crate::{AccountFactory, AnsHost, IbcClient, Manager, ModuleFactory, Proxy, VersionControl};
use abstract_core::{
    objects::AccountId, ACCOUNT_FACTORY, ANS_HOST, IBC_CLIENT, MANAGER, MODULE_FACTORY, PROXY,
    VERSION_CONTROL,
};
use cw_orch::prelude::*;

#[allow(clippy::type_complexity)]
pub fn get_native_contracts<Chain: CwEnv>(
    chain: Chain,
) -> (
    AnsHost<Chain>,
    AccountFactory<Chain>,
    VersionControl<Chain>,
    ModuleFactory<Chain>,
    IbcClient<Chain>,
)
where
    <Chain as cw_orch::environment::TxHandler>::Response: IndexResponse,
{
    let ans_host = AnsHost::new(ANS_HOST, chain.clone());
    let account_factory = AccountFactory::new(ACCOUNT_FACTORY, chain.clone());
    let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
    let module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
    let ibc_client = IbcClient::new(IBC_CLIENT, chain);
    (
        ans_host,
        account_factory,
        version_control,
        module_factory,
        ibc_client,
    )
}

pub fn get_account_contracts<Chain: CwEnv>(
    chain: Chain,
    account_id: Option<AccountId>,
) -> (Manager<Chain>, Proxy<Chain>)
where
    <Chain as cw_orch::environment::TxHandler>::Response: IndexResponse,
{
    if let Some(account_id) = account_id {
        let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
        let account_base = version_control.get_account(account_id).unwrap();
        chain.state().set_address(MANAGER, &account_base.manager);
        chain.state().set_address(PROXY, &account_base.proxy);
        let manager = Manager::new(MANAGER, chain.clone());
        let proxy = Proxy::new(PROXY, chain);
        (manager, proxy)
    } else {
        let manager = Manager::new(MANAGER, chain.clone());
        let proxy = Proxy::new(PROXY, chain);
        (manager, proxy)
    }
}
