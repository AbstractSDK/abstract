use crate::{
    AccountFactory, AnsHost, IbcClient, IbcHost, Manager, ModuleFactory, Proxy, VersionControl,
};
use abstract_core::{
    objects::AccountId, ACCOUNT_FACTORY, ANS_HOST, IBC_CLIENT, IBC_HOST, MANAGER, MODULE_FACTORY,
    PROXY, VERSION_CONTROL,
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
)
where
    <Chain as cw_orch::environment::TxHandler>::Response: IndexResponse,
{
    let ans_host = AnsHost::new(ANS_HOST, chain.clone());
    let account_factory = AccountFactory::new(ACCOUNT_FACTORY, chain.clone());
    let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
    let module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
    (ans_host, account_factory, version_control, module_factory)
}

pub fn get_account_contracts<Chain: CwEnv>(
    version_control: &VersionControl<Chain>,
    account_id: Option<AccountId>,
) -> (Manager<Chain>, Proxy<Chain>)
where
    <Chain as cw_orch::environment::TxHandler>::Response: IndexResponse,
{
    let chain = version_control.get_chain().clone();
    if let Some(account_id) = account_id {
        let manager_contract_id = format!("{MANAGER}-{account_id}");
        let proxy_contract_id = format!("{PROXY}-{account_id}");
        
        let account_base = version_control.get_account(account_id.clone()).unwrap();
        chain
            .state()
            .set_address(&manager_contract_id, &account_base.manager);
        chain
            .state()
            .set_address(&proxy_contract_id, &account_base.proxy);
        let manager = Manager::new(manager_contract_id, chain.clone());
        let proxy = Proxy::new(proxy_contract_id, chain);
        (manager, proxy)
    } else {
        // TODO: shouldn't be used
        let manager = Manager::new(MANAGER, chain.clone());
        let proxy = Proxy::new(PROXY, chain);
        (manager, proxy)
    }
}

pub fn get_ibc_contracts<Chain: CwEnv>(chain: Chain) -> (IbcClient<Chain>, IbcHost<Chain>)
where
    <Chain as cw_orch::environment::TxHandler>::Response: IndexResponse,
{
    let ibc_client = IbcClient::new(IBC_CLIENT, chain.clone());
    let ibc_host = IbcHost::new(IBC_HOST, chain);

    (ibc_client, ibc_host)
}
