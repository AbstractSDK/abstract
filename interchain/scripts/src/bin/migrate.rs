use abstract_cw_staking::{interface::CwStakingAdapter, CW_STAKING_ADAPTER_ID};
use abstract_dex_adapter::{interface::DexAdapter, msg::DexInstantiateMsg, DEX_ADAPTER_ID};
use abstract_interface::{Abstract, AdapterDeployer, AppDeployer, DeployStrategy};
use abstract_money_market_adapter::{
    interface::MoneyMarketAdapter, msg::MoneyMarketInstantiateMsg, MONEY_MARKET_ADAPTER_ID,
};
use challenge_app::{contract::CHALLENGE_APP_ID, Challenge};
use clap::Parser;
use cosmwasm_std::Decimal;
use cw_orch::prelude::{
    networks::{parse_network, ChainInfo},
    *,
};
use dca_app::{contract::DCA_APP_ID, DCA};
use tokio::runtime::Runtime;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn migrate(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    for network in networks {
        let chain = DaemonBuilder::new(network).handle(rt.handle()).build()?;

        let deployment = Abstract::load_from(chain.clone())?;

        deployment.migrate_if_version_changed()?;

        // Deploy Adapters
        CwStakingAdapter::new(CW_STAKING_ADAPTER_ID, chain.clone()).deploy(
            abstract_cw_staking::contract::CONTRACT_VERSION.parse()?,
            Empty {},
            DeployStrategy::Try,
        )?;
        // TODO: DEX oversized, not deployed in current release
        // DexAdapter::new(DEX_ADAPTER_ID, chain.clone()).deploy(
        //     abstract_dex_adapter::contract::CONTRACT_VERSION.parse()?,
        //     DexInstantiateMsg {
        //         recipient_account: 0,
        //         swap_fee: Decimal::permille(3),
        //     },
        //     DeployStrategy::Try,
        // )?;
        MoneyMarketAdapter::new(MONEY_MARKET_ADAPTER_ID, chain.clone()).deploy(
            abstract_money_market_adapter::contract::CONTRACT_VERSION.parse()?,
            MoneyMarketInstantiateMsg {
                recipient_account: 0,
                fee: Decimal::permille(3),
            },
            DeployStrategy::Try,
        )?;

        // Deploy apps

        DCA::new(DCA_APP_ID, chain.clone()).deploy(
            dca_app::contract::DCA_APP_VERSION.parse()?,
            DeployStrategy::Try,
        )?;
        Challenge::new(CHALLENGE_APP_ID, chain.clone()).deploy(
            challenge_app::contract::CHALLENGE_APP_VERSION.parse()?,
            DeployStrategy::Try,
        )?;

        deployment.version_control.approve_any_abstract_modules()?;
    }

    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    network_ids: Vec<String>,
}

fn main() {
    dotenv().ok();
    env_logger::init();
    use dotenv::dotenv;
    let args = Arguments::parse();

    let networks = args
        .network_ids
        .iter()
        .map(|n| parse_network(n).unwrap())
        .collect::<Vec<_>>();

    if let Err(ref err) = migrate(networks) {
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
