use crate::{
    AnsHost, DexApi, IbcClient, Manager, ModuleFactory, OSFactory, Proxy, Subscription,
    TMintStakingApi, VersionControl, ETF as EtfBoot,
};
use abstract_os::{
    objects::OsId, ANS_HOST, ETF, EXCHANGE, IBC_CLIENT, MANAGER, MODULE_FACTORY, OS_FACTORY, PROXY,
    SUBSCRIPTION, TENDERMINT_STAKING, VERSION_CONTROL,
};
use boot_core::{
    prelude::ContractInstance, state::StateInterface, BootEnvironment, IndexResponse, TxHandler,
};

#[cfg(feature = "integration")]
use cw_multi_test::ContractWrapper;

#[allow(clippy::type_complexity)]
pub fn get_native_contracts<Chain: BootEnvironment>(
    chain: Chain,
) -> (
    AnsHost<Chain>,
    OSFactory<Chain>,
    VersionControl<Chain>,
    ModuleFactory<Chain>,
    IbcClient<Chain>,
)
where
    <Chain as TxHandler>::Response: IndexResponse,
{
    let ans_host = AnsHost::new(ANS_HOST, chain.clone());
    let os_factory = OSFactory::new(OS_FACTORY, chain.clone());
    let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
    let module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
    let ibc_client = IbcClient::new(IBC_CLIENT, chain);
    (
        ans_host,
        os_factory,
        version_control,
        module_factory,
        ibc_client,
    )
}

pub fn get_os_core_contracts<Chain: BootEnvironment>(
    chain: Chain,
    os_id: Option<OsId>,
) -> (Manager<Chain>, Proxy<Chain>)
where
    <Chain as TxHandler>::Response: IndexResponse,
{
    if let Some(os_id) = os_id {
        let version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
        let core = version_control.get_os_core(os_id).unwrap();
        chain.state().set_address(MANAGER, &core.manager);
        chain.state().set_address(PROXY, &core.proxy);
        let manager = Manager::new(MANAGER, chain.clone());
        let proxy = Proxy::new(PROXY, chain);
        (manager, proxy)
    } else {
        let manager = Manager::new(MANAGER, chain.clone());
        let proxy = Proxy::new(PROXY, chain);
        (manager, proxy)
    }
}

#[allow(unused_mut)]
pub fn get_apps<Chain: BootEnvironment>(chain: Chain) -> (EtfBoot<Chain>, Subscription<Chain>)
where
    <Chain as TxHandler>::Response: IndexResponse,
{
    let mut etf = EtfBoot::new(ETF, chain.clone());
    let mut subscription = Subscription::new(SUBSCRIPTION, chain);

    #[cfg(feature = "integration")]
    if cfg!(feature = "integration") {
        etf.as_instance_mut()
            .set_mock(Box::new(ContractWrapper::new_with_empty(
                ::etf::contract::execute,
                ::etf::contract::instantiate,
                ::etf::contract::query,
            )));
        subscription
            .as_instance_mut()
            .set_mock(Box::new(ContractWrapper::new_with_empty(
                ::subscription::contract::execute,
                ::subscription::contract::instantiate,
                ::subscription::contract::query,
            )));
    }
    (etf, subscription)
}

#[allow(unused_mut)]
pub fn get_apis<Chain: BootEnvironment>(chain: Chain) -> (DexApi<Chain>, TMintStakingApi<Chain>)
where
    <Chain as TxHandler>::Response: IndexResponse,
{
    let mut dex_api = DexApi::new(EXCHANGE, chain.clone());
    let mut staking_api = TMintStakingApi::new(TENDERMINT_STAKING, chain);
    #[cfg(feature = "integration")]
    if cfg!(feature = "integration") {
        dex_api
            .as_instance_mut()
            .set_mock(Box::new(ContractWrapper::new_with_empty(
                ::dex::contract::execute,
                ::dex::contract::instantiate,
                ::dex::contract::query,
            )));
        staking_api
            .as_instance_mut()
            .set_mock(Box::new(ContractWrapper::new_with_empty(
                ::cw_staking::contract::execute,
                ::cw_staking::contract::instantiate,
                ::cw_staking::contract::query,
            )));
    }
    (dex_api, staking_api)
}
