use pandora_os::memory::msg::*;
use scripts::contract_instances::memory::Memory;
use std::env;

use terra_rust_script::sender::{GroupConfig, Network, Sender};

use secp256k1::Secp256k1;

pub async fn script() -> anyhow::Result<()>
{
    let secp = Secp256k1::new();
    let client = reqwest::Client::new();
    let path = env::var("ADDRESS_JSON")?;
    let propose_on_multisig = true;

    // All configs are set here
    let group_name = "debugging".to_string();
    let config = GroupConfig::new(
        Network::Testnet,
        group_name,
        client,
        "uusd",
        path,
        propose_on_multisig,
        &secp,
    )
    .await?;
    let sender = &Sender::new(&config, secp)?;

    let memory = Memory::new(config);

    memory
        .execute(
            sender,
            ExecuteMsg::update_asset_addresses(vec![],vec![]),
            vec![],
        )
        .await?;
    Ok(())
}



#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    if let Err(ref err) = script().await {
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

