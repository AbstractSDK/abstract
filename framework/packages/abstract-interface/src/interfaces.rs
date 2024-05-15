use abstract_std::{
    objects::AccountId, ACCOUNT_FACTORY, ANS_HOST, IBC_CLIENT, IBC_HOST, MODULE_FACTORY, PROFILE,
    PROFILE_MARKETPLACE, VERSION_CONTROL,
};
use bs_profile::Metadata;
use cw_orch::prelude::*;

use crate::{
    AccountFactory, AnsHost, IbcClient, IbcHost, Manager, ModuleFactory, Profile,
    ProfileMarketplace, Proxy, VersionControl,
};

#[allow(clippy::type_complexity)]
pub fn get_native_contracts<Chain: CwEnv>(
    chain: Chain,
) -> (
    AnsHost<Chain>,
    AccountFactory<Chain>,
    VersionControl<Chain>,
    ModuleFactory<Chain>,
    Profile<Chain, Metadata>,
    ProfileMarketplace<Chain>,
)
where
    <Chain as cw_orch::environment::TxHandler>::Response: IndexResponse,
{
    let ans_host = AnsHost::new(ANS_HOST, chain.clone());
    let account_factory = AccountFactory::new(ACCOUNT_FACTORY, chain.clone());
    let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
    let module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
    let profile_marketplace = ProfileMarketplace::new(PROFILE_MARKETPLACE, chain.clone());
    let bs721_profile = Profile::new(PROFILE, chain.clone());
    (
        ans_host,
        account_factory,
        version_control,
        module_factory,
        bs721_profile,
        profile_marketplace,
    )
}

pub fn get_account_contracts<Chain: CwEnv>(
    version_control: &VersionControl<Chain>,
    account_id: AccountId,
) -> (Manager<Chain>, Proxy<Chain>)
where
    <Chain as cw_orch::environment::TxHandler>::Response: IndexResponse,
{
    let chain = version_control.get_chain().clone();

    let manager = Manager::new_from_id(&account_id, chain.clone());
    let proxy = Proxy::new_from_id(&account_id, chain);

    let account_base = version_control.get_account(account_id.clone()).unwrap();
    manager.set_address(&account_base.manager);
    proxy.set_address(&account_base.proxy);

    (manager, proxy)
}

pub fn get_ibc_contracts<Chain: CwEnv>(chain: Chain) -> (IbcClient<Chain>, IbcHost<Chain>)
where
    <Chain as cw_orch::environment::TxHandler>::Response: IndexResponse,
{
    let ibc_client = IbcClient::new(IBC_CLIENT, chain.clone());
    let ibc_host = IbcHost::new(IBC_HOST, chain);

    (ibc_client, ibc_host)
}
