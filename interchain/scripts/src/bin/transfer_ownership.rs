use abstract_interface::{
    Abstract, AccountExecFns, AccountI, ExecuteMsgFns, MFactoryExecFns, RegistryExecFns,
};
use abstract_std::{
    ibc_client::ExecuteMsgFns as _,
    ibc_host::ExecuteMsgFns as _,
    objects::{
        gov_type::{GovAction, GovernanceDetails},
        AccountId,
    },
};
use cosmos_sdk_proto::traits::Message;
use cw20::Expiration;
use cw_orch_daemon::RUNTIME;

use clap::Parser;
use cw_orch::{daemon::networks::parse_network, prelude::*};
use prost::Name;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Run "cargo run --example download_wasms" in the `abstract-interfaces` package before deploying!
fn transfer_admin(network: ChainInfoOwned, new_admin_env_var: &str) -> anyhow::Result<()> {
    let chain = DaemonBuilder::new(network.clone()).build()?;
    let chain_new_admin = DaemonBuilder::new(network.clone())
        .state(chain.state())
        .mnemonic(std::env::var(new_admin_env_var)?)
        .build()?;

    let new_admin = chain_new_admin.sender_addr();

    let abs = Abstract::load_from(chain.clone())?;
    let new_admin_abs = Abstract::load_from(chain_new_admin.clone())?;

    let account = AccountI::load_from(&abs, AccountId::local(0))?;
    let new_admin_account = AccountI::load_from(&new_admin_abs, AccountId::local(0))?;

    // Update all the contracts code admins
    let contract_admin_upgrades = abs
        .contracts()
        .into_iter()
        .map(|(contract, _version)| contract.clone())
        .map(|contract| prost_types::Any {
            value: cosmrs::proto::cosmwasm::wasm::v1::MsgUpdateAdmin {
                sender: chain.sender_addr().to_string(),
                new_admin: new_admin.to_string(),
                contract: contract.address().unwrap().to_string(),
            }
            .encode_to_vec(),
            type_url: cosmrs::proto::cosmwasm::wasm::v1::MsgUpdateAdmin::type_url(),
        })
        .collect::<Vec<_>>();
    chain.commit_any(contract_admin_upgrades, None)?;

    // Update all the contract admins
    // Transfer ownership
    let cw_ownable_transfer_msg = cw_ownable::Action::TransferOwnership {
        new_owner: new_admin.to_string(),
        expiry: None,
    };
    let cw_ownable_accept = cw_ownable::Action::AcceptOwnership;

    // Registry
    abs.registry
        .update_ownership(cw_ownable_transfer_msg.clone())?;
    new_admin_abs
        .registry
        .update_ownership(cw_ownable_accept.clone())?;

    // Ans host
    abs.ans_host
        .update_ownership(cw_ownable_transfer_msg.clone())?;
    new_admin_abs
        .ans_host
        .update_ownership(cw_ownable_accept.clone())?;

    // Module factory
    abs.module_factory
        .update_ownership(cw_ownable_transfer_msg.clone())?;
    new_admin_abs
        .module_factory
        .update_ownership(cw_ownable_accept.clone())?;

    // IBC Client
    abs.ibc
        .client
        .update_ownership(cw_ownable_transfer_msg.clone())?;
    new_admin_abs
        .ibc
        .client
        .update_ownership(cw_ownable_accept.clone())?;

    // IBC Host
    abs.ibc
        .host
        .update_ownership(cw_ownable_transfer_msg.clone())?;
    new_admin_abs
        .ibc
        .host
        .update_ownership(cw_ownable_accept.clone())?;

    // Change the base account ownership as well
    account.update_ownership(GovAction::TransferOwnership {
        new_owner: GovernanceDetails::Monarchy {
            monarch: new_admin.to_string(),
        },
        expiry: Some(Expiration::AtTime(chain.block_info()?.time.plus_hours(2))),
    })?;
    new_admin_account.update_ownership(GovAction::AcceptOwnership {})?;

    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long, num_args = 1)]
    network_id: String,
    #[arg(short, long, num_args = 1)]
    admin_new_mnemonic_env_var: String,
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let args = Arguments::parse();

    // let networks = vec![abstract_scripts::ROLLKIT_TESTNET];

    let network = parse_network(&args.network_id).unwrap();

    if let Err(ref err) = transfer_admin(network.into(), &args.admin_new_mnemonic_env_var) {
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
