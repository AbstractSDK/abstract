use abstract_boot::Abstract;

use boot_core::{networks::NetworkInfo, *};

use abstract_core::ans_host::{ContractListResponse, ExecuteMsg, QueryMsgFns};
use boot_core::networks::juno::JUNO_CHAIN;
use boot_core::networks::NetworkKind;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub const JUNO_1: NetworkInfo = NetworkInfo {
    kind: NetworkKind::Mainnet,
    id: "juno-1",
    gas_denom: "ujuno",
    gas_price: 0.0025,
    grpc_urls: &["http://juno-grpc.polkachu.com:12690"],
    chain_info: JUNO_CHAIN,
    lcd_url: None,
    fcd_url: None,
};

/// Script that takes existing versions in Version control, removes them, and swaps them wit ha new version
pub fn fix_names() -> anyhow::Result<()> {
    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(JUNO_1).build();
    let (_sender, chain) = instantiate_daemon_env(&rt, options?)?;

    let deployment = Abstract::new(chain);

    let mut all_contract_entries = vec![];

    let mut last_contract = None;

    loop {
        let ContractListResponse { mut contracts } =
            deployment
                .ans_host
                .contract_list(None, None, last_contract)?;
        if contracts.is_empty() {
            break;
        }
        all_contract_entries.append(&mut contracts);
        last_contract = all_contract_entries
            .last()
            .map(|(entry, _)| entry.to_owned());
    }

    let mut contracts_to_remove = vec![];
    let mut contracts_to_add = vec![];

    for (mut entry, addr) in all_contract_entries {
        if entry.protocol == "wynd" {
            contracts_to_remove.push(entry.clone().into());
            entry.protocol = "wyndex".to_string();
            entry.contract = entry.contract.replace("staking/wynd/", "staking/wyndex/");
            contracts_to_add.push((entry.into(), addr.to_string()));
        }
    }
    //
    println!("Removing {} contracts", contracts_to_remove.len());
    println!("Removing contracts: {:?}", contracts_to_remove);
    println!("Adding {} contracts", contracts_to_add.len());
    println!("Adding contracts: {:?}", contracts_to_add);

    // chain.wait_blocks(500).unwrap();

    // add the contracts
    deployment
        .ans_host
        .execute_chunked(&contracts_to_add, 20, |chunk| {
            ExecuteMsg::UpdateContractAddresses {
                to_add: chunk.to_vec(),
                to_remove: vec![],
            }
        })?;

    // remove the contracts
    deployment
        .ans_host
        .execute_chunked(&contracts_to_remove, 20, |chunk| {
            ExecuteMsg::UpdateContractAddresses {
                to_add: vec![],
                to_remove: chunk.to_vec(),
            }
        })?;

    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    if let Err(ref err) = fix_names() {
        log::error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));

        // The backtrace is not always generated. Try to run this example
        // with `$env:RUST_BACKTRACE=1`.
        //    if let Some(backtrace) = e.backtrace() {
        //        log::debug!("backtrace: {:?}", backtrace);
        //    }

        ::std::process::exit(1);
    }
}
