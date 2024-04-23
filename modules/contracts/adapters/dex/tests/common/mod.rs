use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_interface::{AbstractAccount, AccountFactory};
use cw_orch::prelude::*;
pub fn create_default_account<Chain: CwEnv>(
    factory: &AccountFactory<Chain>,
) -> anyhow::Result<AbstractAccount<Chain>> {
    let os = factory.create_default_account(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(factory.get_chain().sender()).to_string(),
    })?;
    Ok(os)
}

// /// Instantiates the dex adapter and registers it with the version control
// #[allow(dead_code)]
// pub fn init_dex_adapter(
//     chain: Mock,
//     deployment: &Abstract<Mock>,
//     version: Option<String>,
// ) -> anyhow::Result<DexAdapter<Mock>> {
//     let mut dex_adapter = DexAdapter::new(EXCHANGE, chain);
//     dex_adapter
//         .as_instance_mut()
//         .set_mock(Box::new(boot_core::ContractWrapper::new_with_empty(
//             ::dex::contract::execute,
//             ::dex::contract::instantiate,
//             ::dex::contract::query,
//         )));
//     dex_adapter.upload()?;
//     dex_adapter.instantiate(
//         &InstantiateMsg::<DexInstantiateMsg>{
//             app: DexInstantiateMsg{
//                 swap_fee: Decimal::percent(1),
//                 recipient_os: 0,
//             },
//             base: abstract_core::adapter::BaseInstantiateMsg {
//                 ans_host_address: deployment.ans_host.addr_str()?,
//                 version_control_address: deployment.version_control.addr_str()?,
//             },
//         },
//         None,
//         None,
//     )?;

//     let version: Version = version
//         .unwrap_or_else(|| deployment.version.to_string())
//         .parse()?;

//     deployment
//         .version_control
//         .register_adapters(vec![dex_adapter.as_instance()], &version)?;
//     Ok(dex_adapter)
// }
