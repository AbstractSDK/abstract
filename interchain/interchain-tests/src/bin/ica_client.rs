use abstract_client::{AbstractClient, Namespace};
use abstract_ica_client::chain_types::evm::types::{UCS01_FORWARDER_CONTRACT, UCS01_PROTOCOL};
use abstract_interface::{ExecuteMsgFns, IcaClient};
use abstract_std::{
    ica_client::{IcaAction, IcaActionResult, InstantiateMsg, QueryMsg, QueryMsgFns},
    objects::{
        namespace::ABSTRACT_NAMESPACE, ContractEntry, UncheckedChannelEntry, UncheckedContractEntry,
    },
    IBC_CLIENT, ICA_CLIENT,
};
use alloy::{
    primitives::{Address, Uint},
    providers::Provider,
};
use cosmwasm_std::{coins, Binary};
use cw_orch::prelude::*;
use cw_orch_interchain::{core::IbcQueryHandler, prelude::InterchainEnv};
use evm_note::interface::EvmNote;
use networks::union::UNION_TESTNET_8;
use polytone_evm::bind::ierc20::ERC20;
use std::future::IntoFuture;
use union_connector::{interchain_env::UnionInterchainEnv, networks::UncheckedRemoteEvmConfig};

const TEST_ACCOUNT_NAMESPACE: &str = "testing";

pub const CHAIN_NAME: &str = "bartio";

fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv()?;
    pretty_env_logger::init();
    // This is an integration test with Abstract And polytone EVM already deployed on Union

    // If it's not deployed, we can redeploy it here
    let chain_info = UNION_TESTNET_8;

    let chain = Daemon::builder(chain_info.clone()).build()?;

    // let abs = AbstractClient::builder(chain.clone()).build(chain.sender().clone())?;
    let abs = AbstractClient::new(chain.clone())?;

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

    // We need to register the EVM note here (already existent in state.json)
    // abs.registry().register_natives(vec![(
    //     EvmNote::new(chain.clone()).as_instance(),
    //     evm_note::contract::CONTRACT_VERSION.to_string(),
    // )])?;

    // We need to register the IBC channels

    let evm_config = union_connector::networks::get_remote_evm_config(CHAIN_NAME).unwrap();
    // abs.name_service().update_channels(
    //     vec![
    //         (
    //             UncheckedChannelEntry::new(evm_config.chain_name, UCS01_PROTOCOL),
    //             evm_config.ics20_dst_channel.to_string(),
    //         ),
    //         (
    //             UncheckedChannelEntry::new(
    //                 evm_config.chain_name,
    //                 format!("{}/counterparty", UCS01_PROTOCOL).as_str(),
    //             ),
    //             evm_config.ics20_src_channel.to_string(),
    //         ),
    //     ],
    //     vec![],
    // )?;
    // abs.name_service().update_contract_addresses(
    //     vec![(
    //         UncheckedContractEntry {
    //             contract: UCS01_FORWARDER_CONTRACT.to_string(),
    //             protocol: UCS01_PROTOCOL.to_string(),
    //         },
    //         evm_config.ucs01_forwarder.to_string(),
    //     )],
    //     vec![],
    // )?;

    // We query the ICA client action from the script
    let receiver_address = "76FaA72D2949072f24251f4D84cDCb60265d6697";

    let interchain = UnionInterchainEnv::new(chain.clone(), &evm_config);
    let funds_before = get_balance(&interchain, &evm_config, receiver_address)?;

    // We send the message from the account directly
    let ica_action = abs.ica_client().ica_action(
        account.address()?.to_string(),
        vec![IcaAction::Fund {
            funds: account_coins,
            receiver: Some(hex::decode(receiver_address)?.into()),
            memo: None,
        }],
        "bartio".parse()?,
    )?;

    let tx_response = account.execute(ica_action.msgs, &[])?;

    // We make sure the messages do the right actions with a query on the EVM chain
    interchain.await_and_check_packets(&chain.chain_id(), tx_response.into())?;
    // let tx_hash = "66F33B8A09DFCD8079B110BD84D58D2644D19018C2F9B53789B521A4FA091D3D";
    // interchain.await_from_tx_hash(&chain.chain_id(), tx_hash.to_string())?;

    let funds_after = get_balance(&interchain, &evm_config, receiver_address)?;

    assert_eq!((funds_after - funds_before).to_string(), 9.to_string());

    Ok(())
}

fn get_balance(
    interchain: &UnionInterchainEnv,
    evm_config: &UncheckedRemoteEvmConfig,
    address: &str,
) -> anyhow::Result<Uint<256, 4>> {
    let evm = interchain.get_evm_chain(evm_config.chain_id)?;
    let muno_erc20_addr = evm_config.muno_erc20.unwrap();
    let muno_erc20 = ERC20::new(
        Address::parse_checksummed(muno_erc20_addr, None)?,
        evm.clone(),
    );
    let remote_address: Address = address.parse()?;
    let remote_muno_balance = interchain
        .rt
        .block_on(muno_erc20.balanceOf(remote_address).call().into_future())?;

    Ok(remote_muno_balance._0)
}
