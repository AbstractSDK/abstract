use abstract_os::IBC_CLIENT;

use crate::{
    AnsHost, DexExtension, IbcClient, Manager, ModuleFactory, OSFactory, Proxy, Subscription,
    TMintStakingExtension, VersionControl, ETF as EtfBoot,
};
use abstract_os::{
    ANS_HOST, ETF, EXCHANGE, MANAGER, MODULE_FACTORY, OS_FACTORY, PROXY, SUBSCRIPTION,
    TENDERMINT_STAKING, VERSION_CONTROL,
};
use boot_core::{state::StateInterface, BootEnvironment, IndexResponse, TxHandler};

#[allow(clippy::type_complexity)]
pub fn get_native_contracts<Chain: BootEnvironment>(
    chain: &Chain,
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
    let ans_host = AnsHost::new(ANS_HOST, chain);
    let os_factory = OSFactory::new(OS_FACTORY, chain);
    let version_control = VersionControl::new(VERSION_CONTROL, chain);
    let module_factory = ModuleFactory::new(MODULE_FACTORY, chain);
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
    chain: &Chain,
    os_id: Option<u32>,
) -> (Manager<Chain>, Proxy<Chain>)
where
    <Chain as TxHandler>::Response: IndexResponse,
{
    if let Some(os_id) = os_id {
        let version_control = VersionControl::new(VERSION_CONTROL, chain);
        let core = version_control.get_os_core(os_id).unwrap();
        chain.state().set_address(MANAGER, &core.manager);
        chain.state().set_address(PROXY, &core.proxy);
        let manager = Manager::new(MANAGER, chain);
        let proxy = Proxy::new(PROXY, chain);
        (manager, proxy)
    } else {
        let manager = Manager::new(MANAGER, chain);
        let proxy = Proxy::new(PROXY, chain);
        (manager, proxy)
    }
}

pub fn get_apps<Chain: BootEnvironment>(chain: &Chain) -> (EtfBoot<Chain>, Subscription<Chain>)
where
    <Chain as TxHandler>::Response: IndexResponse,
{
    let liquidity_interface = EtfBoot::new(ETF, chain);
    let subscription = Subscription::new(SUBSCRIPTION, chain);
    (liquidity_interface, subscription)
}

pub fn get_extensions<Chain: BootEnvironment>(
    chain: &Chain,
) -> (DexExtension<Chain>, TMintStakingExtension<Chain>)
where
    <Chain as TxHandler>::Response: IndexResponse,
{
    let dex_extension = DexExtension::new(EXCHANGE, chain);
    let staking_extension = TMintStakingExtension::new(TENDERMINT_STAKING, chain);
    (dex_extension, staking_extension)
}
