use abstract_boot::Abstract;

use boot_core::{networks::NetworkInfo, *};

use abstract_core::ans_host::{AssetListResponse, ExecuteMsg, QueryMsgFns};

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

    let mut all_asset_entries = vec![];

    let mut last_asset = None;

    loop {
        let AssetListResponse { mut assets } =
            deployment.ans_host.asset_list(None, None, last_asset)?;
        if assets.is_empty() {
            break;
        }
        all_asset_entries.append(&mut assets);
        last_asset = all_asset_entries.last().map(|(entry, _)| entry.to_string());
    }

    let mut assets_to_remove = vec![];
    let mut assets_to_add = vec![];

    for (entry, info) in all_asset_entries {
        if entry.clone().as_str().starts_with("wynd/") {
            assets_to_remove.push(entry.to_string());
            assets_to_add.push((
                entry.as_str().replace("wynd/", "wyndex/").to_string(),
                info.into(),
            ));
        }
    }

    // println!("Removing {} assets", assets_to_remove.len());
    // println!("Removing assets: {:?}", assets_to_remove);
    // println!("Adding {} assets", assets_to_add.len());
    // println!("Adding assets: {:?}", assets_to_add);

    // add the assets
    deployment
        .ans_host
        .execute_chunked(&assets_to_add, 25, |chunk| {
            ExecuteMsg::UpdateAssetAddresses {
                to_add: chunk.to_vec(),
                to_remove: vec![],
            }
        })?;

    // remove the assets
    deployment
        .ans_host
        .execute_chunked(&assets_to_remove, 25, |chunk| {
            ExecuteMsg::UpdateAssetAddresses {
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
