#![cfg(feature = "osmosis")]

use crate::networks::OSMO_5;
use tokio::runtime::Runtime;

use abstract_interface::{Abstract, AbstractAccount};

use abstract_osmosis_adapter::OSMOSIS;

use cosmwasm_std::Uint128;
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;

use anyhow::Result as AnyResult;

use cosmwasm_std::coins;

// TODO: finish it
fn swap() -> AnyResult<()> {
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .chain(OSMO_5)
        .handle(rt.handle())
        .build()?;

    let abstr = Abstract::load_from(chain)?;

    // abstr.account = AbstractAccount::new(chain, Some(1));

    // let proxy_addr = os.proxy.address()?;

    // let swap_value = 1_000_000_000u128;

    // chain.bank_send(proxy_addr.to_string(), coins(swap_value, "uatom"))?;

    // // Before swap, we need to have 0 uosmo and swap_value uatom
    // let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    // assert_eq!(balances, coins(swap_value, "uatom"));
    // // swap 100_000 uatom to uosmo
    // dex_adapter.swap(("atom", swap_value), "osmo", OSMOSIS.into())?;

    // // Assert balances
    // let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    // assert_eq!(balances.len(), 1);
    // let balance = chain.query_balance(proxy_addr.as_ref(), "uosmo")?;
    // assert!(balance > Uint128::zero());

    Ok(())
}

fn main() {}
