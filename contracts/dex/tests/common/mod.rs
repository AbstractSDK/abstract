pub const ROOT_USER: &str = "root_user";
use abstract_boot::AbstractAccount;
use abstract_boot::AccountFactory;
use abstract_core::objects::gov_type::GovernanceDetails;

use abstract_boot::boot_core::Mock;
use cosmwasm_std::Addr;

pub fn create_default_account(
    factory: &AccountFactory<Mock>,
) -> anyhow::Result<AbstractAccount<Mock>> {
    let os = factory.create_default_account(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(ROOT_USER).to_string(),
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
