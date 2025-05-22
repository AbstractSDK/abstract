use abstract_ica_client::msg::ExecuteMsgFns;
use abstract_interface::{ica_client::IcaClient, Abstract, AccountI, RegistryExecFns};
use abstract_std::{
    ethereum::ETHEREUM_SEPOLIA,
    objects::{gov_type::GovernanceDetails, module::ModuleInfo},
    ICA_CLIENT,
};
use cw_orch_daemon::RUNTIME;

use abstract_scripts::{assert_wallet_balance, SUPPORTED_CHAINS};

use clap::Parser;
use cw_orch::{
    contract::Contract,
    daemon::networks::parse_network,
    environment::{ChainKind, NetworkInfo},
    prelude::*,
};

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const EVM_NOTE_ID: &str = "abstract:evm-note";

// Run "cargo run --example download_wasms" in the `abstract-interfaces` package before deploying!
fn full_deploy(mut networks: Vec<ChainInfoOwned>) -> anyhow::Result<()> {
    if networks.is_empty() {
        networks = SUPPORTED_CHAINS.iter().map(|x| x.clone().into()).collect();
    }

    let networks = RUNTIME.block_on(assert_wallet_balance(networks));

    for network in networks {
        let chain = DaemonBuilder::new(network.clone()).build()?;
        let abs_deployment = Abstract::load_from(chain.clone())?;

        // Version check
        // TODO: automate this
        let evm_note_addr = "union1uz8gd9z30thd8d8vxrch5m7s6ryeamfuxmh759tq53ww76lzj8dqkp97q5";
        let expected_evm_note_version = "0.4.0";
        let evm_note_cw2 = chain
            .wasm_querier()
            .item_query(&Addr::unchecked(evm_note_addr), cw2::CONTRACT)?;
        assert_eq!(evm_note_cw2.version, expected_evm_note_version);

        let ica_client = IcaClient::new(chain.clone());
        ica_client.upload_if_needed();
        ica_client.instantiate(
            &abstract_ica_client::msg::InstantiateMsg {
                ans_host_address: abs_deployment.ans_host.addr_str()?,
                registry_address: abs_deployment.registry.addr_str()?,
            },
            Some(&chain.sender_addr()),
            &[],
        )?;
        ica_client.register_infrastructure(ETHEREUM_SEPOLIA.parse().unwrap(), evm_note_addr);
        abs_deployment.registry.register_services(vec![(
            ica_client.as_instance(),
            abstract_ica_client::contract::CONTRACT_VERSION.to_owned(),
        )])?;
        abs_deployment.registry.approve_any_abstract_modules()?;
    }

    Ok(())
}

// #[derive(Parser, Default, Debug)]
// #[command(author, version, about, long_about = None)]
// struct Arguments {
//     /// Network Id to deploy on
//     #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
//     network_ids: Vec<String>,
// }

pub const UNION_NET: NetworkInfo = NetworkInfo {
    chain_name: "union",
    pub_address_prefix: "union",
    coin_type: 118,
};
pub const UNION_TESTNET_10: ChainInfo = ChainInfo {
    kind: ChainKind::Testnet,
    chain_id: "union-testnet-10",
    gas_denom: "muno",
    gas_price: 1.0,
    grpc_urls: &["https://grpc.rpc-node.union-testnet-10.union.build:443"],
    network_info: UNION_NET,
    lcd_url: None,
    fcd_url: None,
};
fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    // let args = Arguments::parse();

    let networks = vec![UNION_TESTNET_10.into()];

    if let Err(ref err) = full_deploy(networks) {
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
