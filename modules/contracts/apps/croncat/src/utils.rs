use abstract_core::objects::UncheckedContractEntry;
use abstract_sdk::{feature_objects::AnsHost, prelude::*, AbstractSdkResult};
use cosmwasm_std::{coin, Addr, Api, Coin, Deps, QuerierWrapper};
use croncat_sdk_manager::msg::ManagerQueryMsg;
use cw20::Cw20CoinVerified;
use cw_asset::{AssetError, AssetInfoBase, AssetListUnchecked};

use crate::{contract::CroncatApp, error::AppError, CRON_CAT_FACTORY};

// Check if module is installed on the account
pub(crate) fn assert_module_installed(
    deps: Deps,
    contract_addr: &Addr,
    app: &CroncatApp,
) -> AbstractSdkResult<()> {
    let contract_version = cw2::query_contract_info(&deps.querier, contract_addr)?;
    let modules = app.modules(deps);
    let module_addr = modules.module_address(&contract_version.contract)?;
    if module_addr != contract_addr {
        Err(abstract_core::AbstractError::AppNotInstalled(contract_version.contract).into())
    } else {
        Ok(())
    }
}

// Check if user balance non empty
pub(crate) fn user_balance_nonempty(
    deps: Deps,
    proxy_addr: Addr,
    manager_addr: Addr,
) -> Result<bool, AppError> {
    let coins: Vec<Cw20CoinVerified> = deps.querier.query_wasm_smart(
        manager_addr,
        &ManagerQueryMsg::UsersBalances {
            address: proxy_addr.into_string(),
            from_index: None,
            // One is enough to know
            limit: Some(1),
        },
    )?;
    Ok(!coins.is_empty())
}

// Sort assetlist to coins and cw20s
pub(crate) fn sort_funds(
    api: &dyn Api,
    assets: AssetListUnchecked,
) -> Result<(Vec<Coin>, Vec<Cw20CoinVerified>), AssetError> {
    let assets = assets.check(api, None)?;
    let (funds, cw20s) =
        assets
            .into_iter()
            .fold((vec![], vec![]), |(mut funds, mut cw20s), asset| {
                match &asset.info {
                    AssetInfoBase::Native(denom) => funds.push(coin(asset.amount.u128(), denom)),
                    AssetInfoBase::Cw20(address) => cw20s.push(Cw20CoinVerified {
                        address: address.clone(),
                        amount: asset.amount,
                    }),
                    _ => todo!(),
                }
                (funds, cw20s)
            });
    Ok((funds, cw20s))
}

pub(crate) fn factory_addr(
    querier: &QuerierWrapper,
    ans_host: &AnsHost,
) -> Result<Addr, crate::error::AppError> {
    let factory_entry = UncheckedContractEntry::try_from(CRON_CAT_FACTORY.to_owned())?.into();
    let factory_addr = ans_host.query_contract(querier, &factory_entry)?;
    Ok(factory_addr)
}
