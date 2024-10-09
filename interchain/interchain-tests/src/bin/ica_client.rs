use abstract_client::{AbstractClient, Namespace};
use abstract_interface::AccountI;
use abstract_std::{
    ica_client::{IcaAction, IcaActionResult, QueryMsg},
    IBC_CLIENT, ICA_CLIENT,
};
use cosmwasm_std::coins;
use cw_orch::prelude::*;
use networks::union::UNION_TESTNET_8;

const TEST_ACCOUNT_NAMESPACE: &str = "testing";

fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv()?;
    pretty_env_logger::init();
    // This is an integration test with Abstract And polytone EVM already deployed on Union

    // If it's not deployed, we can redeploy it here
    let chain_info = UNION_TESTNET_8;

    let chain = Daemon::builder(chain_info.clone()).build()?;

    let account_wasm = AccountI::<Daemon>::wasm(&chain_info.into());

    let img_size = std::fs::metadata(account_wasm.path()).unwrap().len();
    panic!("{:?}", img_size);

    let abs = AbstractClient::builder(chain.clone()).build(chain.sender().clone())?;
    // let abs = AbstractClient::new(chain.clone())?;

    // We get the account and install the ICA client app on it
    let account = abs
        .account_builder()
        .namespace(Namespace::new(TEST_ACCOUNT_NAMESPACE)?)
        .build()?;
    // Install IBC if not installed
    if !account.module_installed(IBC_CLIENT)? {
        account
            .as_ref()
            .install_module::<Empty>(IBC_CLIENT, None, &[])?;
    }
    // Install ICA_CLIENT if not installed
    if !account.module_installed(ICA_CLIENT)? {
        account
            .as_ref()
            .install_module::<Empty>(ICA_CLIENT, None, &[])?;
    }
    // We start by sending some funds to the interchain account to be able to send it around in the ica action

    let account_balance = account.query_balance(chain_info.gas_denom)?;
    let account_coins = coins(9, chain_info.gas_denom);
    if account_balance < account_coins[0].amount {
        log::warn!("Sending some funds from wallet to account.");
        // @feedback make it easier to send funds from wallet?
        //  - maybe     .deposit() method
        chain.rt_handle.block_on(chain.sender().bank_send(
            // @feedback: test_acc.address() to get the address of the proxy?
            &account.address()?,
            account_coins.clone(),
        ))?;
    }

    // We query the ICA client action from the script

    // We send the message from the account directly
    let ica_msg: IcaActionResult = account.query_module(
        ICA_CLIENT,
        &QueryMsg::IcaAction {
            account_address: account.address()?.to_string(),
            chain: "bartio".parse()?,
            actions: vec![IcaAction::Fund {
                funds: account_coins,
                receiver: None,
                memo: None,
            }],
        },
    )?;
    println!("{:?}", ica_msg);

    // We make sure the messages do the right actions with a query on the EVM chain

    Ok(())
}
