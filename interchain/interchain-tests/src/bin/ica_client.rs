use abstract_client::{AbstractClient, Namespace};
use abstract_ica_client::chain_types::evm::types::{UCS01_FORWARDER_CONTRACT, UCS01_PROTOCOL};
use abstract_interface::{ExecuteMsgFns, IcaClient};
use abstract_std::{
    ica_client::{IcaAction, IcaActionResult, IcaExecute, InstantiateMsg, QueryMsg, QueryMsgFns},
    objects::{
        namespace::ABSTRACT_NAMESPACE, ContractEntry, UncheckedChannelEntry, UncheckedContractEntry,
    },
    IBC_CLIENT, ICA_CLIENT,
};
use alloy::{
    primitives::{Address, Uint},
    providers::Provider,
};
use alloy::{
    providers::RootProvider,
    sol_types::SolCall,
    transports::http::{Client, Http},
};
use cosmwasm_std::{coin, Binary, HexBinary};
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::PacketAnalysis;
use cw_orch_interchain::{core::IbcQueryHandler, prelude::InterchainEnv};
use evm_note::msg::QueryMsgFns as _;
use evm_note::{interface::EvmNote, msg::EvmMsg};
use networks::union::UNION_TESTNET_8;
use polytone_evm::bind::{
    evmvoice::EvmVoice::{self, Sender},
    ierc20::ERC20,
};
use queriers::{Ibc, Node};
use std::future::IntoFuture;
use union_connector::{interchain_env::UnionInterchainEnv, networks::UncheckedRemoteEvmConfig};

const TEST_ACCOUNT_NAMESPACE: &str = "testing";

pub const CHAIN_NAME: &str = "bartio";

fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv()?;
    pretty_env_logger::init();
    // This is an integration test with Abstract And polytone EVM already deployed on Union

    let chain_info = UNION_TESTNET_8;
    let chain = Daemon::builder(chain_info.clone()).build()?;

    // If it's not deployed, we can redeploy it here
    // let abs = AbstractClient::builder(chain.clone()).build(chain.sender().clone())?;
    let abs = AbstractClient::new(chain.clone())?;

    // We get the account
    let account = abs
        .account_builder()
        .namespace(Namespace::new(TEST_ACCOUNT_NAMESPACE)?)
        .build()?;

    let account_coins = coin(9, chain_info.gas_denom);
    // We start by sending some funds to the interchain account to be able to send it around in the ica action
    {
        let account_balance = account.query_balance(chain_info.gas_denom)?;
        if account_balance < account_coins.amount {
            log::warn!("Sending some funds from wallet to account.");
            // @feedback make it easier to send funds from wallet?
            //  - maybe     .deposit() method
            chain.rt_handle.block_on(chain.sender().bank_send(
                // @feedback: test_acc.address() to get the address of the proxy?
                &account.address()?,
                vec![account_coins.clone()],
            ))?;
        }
    }

    // We need to register the EVM note here (already existent in state.json)
    // abs.registry().register_natives(vec![(
    //     evm_note.as_instance(),
    //     evm_note::contract::CONTRACT_VERSION.to_string(),
    // )])?;

    // We need to register the IBC channels

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

    let evm_config = union_connector::networks::get_remote_evm_config(CHAIN_NAME).unwrap();
    let evm_note = EvmNote::new(chain.clone());
    let interchain = UnionInterchainEnv::new(chain.clone(), &evm_config);

    let remote_address = get_remote_address(
        &evm_note,
        &interchain,
        &evm_config,
        account.address()?.as_str(),
    )?;

    log::info!(
        "Sending funds to the remote address on {CHAIN_NAME}: {:?}",
        remote_address
    );

    let funds_before = get_balance(&interchain, &evm_config, &remote_address)?;

    // We send some funds from the cosmos chain to the remote address
    let funds_after = {
        let ica_action = abs.ica_client().ica_action(
            account.address()?.to_string(),
            vec![IcaAction::Fund {
                funds: vec![account_coins.clone()],
                receiver: Some(remote_address.as_slice().into()),
                memo: None,
            }],
            "bartio".parse()?,
        )?;

        // In case we want to execute the transaction
        let tx_response = account.execute(ica_action.msgs, &[])?;
        interchain.await_and_check_packets(&chain.chain_id(), tx_response.into())?;
        let funds_after = get_balance(&interchain, &evm_config, &remote_address)?;
        assert_eq!((funds_after - funds_before).to_string(), 9.to_string());
        funds_after

        // In case we want to debug and the transaction has already been executed
        // let tx_hash = "012BD18DDDEA3E73193AF2172B1958417245D20EB7E16A394260893F3C4149F9";
        // interchain
        //     .await_from_tx_hash(&chain.chain_id(), tx_hash.to_string())?
        //     .assert()?;
        // get_balance(&interchain, &evm_config, &remote_address)?
    };

    // We want to send the funds back now from the address to the local address
    {
        let muno_erc20_addr = evm_config.muno_erc20.unwrap();

        let approve_msg = EvmMsg::call(
            muno_erc20_addr.to_string(),
            HexBinary::from(
                ERC20::approveCall::new((
                    Address::parse_checksummed(evm_config.ucs01_handler, None)?,
                    alloy::primitives::U256::from(3),
                ))
                .abi_encode(),
            ),
        );
        let send_back_msg = EvmMsg::call(
            evm_config.ucs01_handler.to_string(),
            HexBinary::from(
                polytone_evm::bind::irelay::IRelay::sendCall::new(
                    // srcChannel: channel-90 - receiver: 0x3d95c07a0380cff70fb9d086f076e19a8a3807cb - denom: 0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14 - amount: 50000000000000 - extension:  - revNumber: 9 - revHeight: 1000000099 - timeStamp: 0
                    (
                        evm_config.ics20_src_channel.to_string(),
                        bech32::decode(&account.address()?.to_string())?.1.into(),
                        vec![polytone_evm::bind::irelay::IRelay::LocalToken {
                            denom: Address::parse_checksummed(muno_erc20_addr, None)?,
                            amount: account_coins.amount.u128(),
                        }],
                        "".to_string(),
                        polytone_evm::bind::irelay::IRelay::IbcCoreClientV1HeightData {
                            revision_number: 9,
                            revision_height: 1000000099,
                        },
                        0,
                    ),
                )
                .abi_encode(),
            ),
        );

        let ica_action = abs.ica_client().ica_action(
            account.address()?.to_string(),
            vec![IcaAction::Execute(IcaExecute::Evm {
                msgs: vec![approve_msg, send_back_msg],
                callback: None,
            })],
            CHAIN_NAME.parse()?,
        )?;

        let tx_response = account.execute(ica_action.msgs, &[])?;
        interchain.await_and_check_packets(&chain.chain_id(), tx_response.into())?;
        let funds_final = get_balance(&interchain, &evm_config, &remote_address)?;
        assert_eq!((funds_after - funds_final).to_string(), 9.to_string());

        // In case we want to debug and the transaction has already been executed
        // let tx_hash = "827026275B6931567810D471ADCE9779D0B4FE9AC57F50783EAAADE16E6553F4";
        // interchain
        //     .await_from_tx_hash(&evm_config.chain_id().to_string(), tx_hash.to_string())?
        //     .assert()?;
    }

    Ok(())
}

fn get_balance(
    interchain: &UnionInterchainEnv,
    evm_config: &UncheckedRemoteEvmConfig,
    address: &Address,
) -> anyhow::Result<Uint<256, 4>> {
    let evm = interchain.get_evm_chain(evm_config.chain_id)?;
    let muno_erc20_addr = evm_config.muno_erc20.unwrap();
    let muno_erc20 = ERC20::new(
        Address::parse_checksummed(muno_erc20_addr, None)?,
        evm.clone(),
    );
    let remote_muno_balance = interchain
        .rt
        .block_on(muno_erc20.balanceOf(*address).call().into_future())?;

    Ok(remote_muno_balance._0)
}

// let evm= EvmVoice::new(evm_note_address, evm);

fn predict_remote_address(
    cosmos_note: &EvmNote<Daemon>,
    interchain: UnionInterchainEnv,
    evm_config: &UncheckedRemoteEvmConfig,
    sender_address: &str,
) -> anyhow::Result<Address> {
    let rt = &cosmos_note.environment().rt_handle;
    let evm_voice = {
        let voice_address = cosmos_note.pair()?.unwrap().remote_port;
        let evm = interchain.get_evm_chain(evm_config.chain_id)?;
        EvmVoice::new(voice_address.parse()?, evm)
    };
    let source_port = format!("wasm.{}", cosmos_note.addr_str()?);
    let connection = {
        let source_channel = cosmos_note.active_channel()?.unwrap();
        let node: Ibc = cosmos_note.environment().querier();
        let active_channel = rt.block_on(node._channel(source_port.clone(), source_channel))?;
        let destination_channel = active_channel.counterparty.unwrap().channel_id;
        rt.block_on(
            evm_voice
                .channelToConnection(destination_channel.to_string())
                .call()
                .into_future(),
        )?
        ._0
    };

    let sender = Sender {
        connection: connection.to_string(),
        port: source_port.to_string(),
        sender: sender_address.to_string(),
    };
    Ok(rt
        .block_on(
            evm_voice
                .getExpectedProxyAddress(sender.clone())
                .call()
                .into_future(),
        )?
        ._0)
}

fn get_remote_address(
    cosmos_note: &EvmNote<Daemon>,
    interchain: &UnionInterchainEnv,
    evm_config: &UncheckedRemoteEvmConfig,
    sender_address: &str,
) -> anyhow::Result<Address> {
    let rt = &cosmos_note.environment().rt_handle;
    let evm_voice = {
        let voice_address = cosmos_note.pair()?.unwrap().remote_port;
        let evm = interchain.get_evm_chain(evm_config.chain_id)?;
        EvmVoice::new(voice_address.parse()?, evm)
    };
    let source_port = format!("wasm.{}", cosmos_note.addr_str()?);
    let connection = {
        let source_channel = cosmos_note.active_channel()?.unwrap();
        let node: Ibc = cosmos_note.environment().querier();
        let active_channel = rt.block_on(node._channel(source_port.clone(), source_channel))?;
        let destination_channel = active_channel.counterparty.unwrap().channel_id;
        rt.block_on(
            evm_voice
                .channelToConnection(destination_channel.to_string())
                .call()
                .into_future(),
        )?
        ._0
    };

    let sender = Sender {
        connection: connection.to_string(),
        port: source_port.to_string(),
        sender: sender_address.to_string(),
    };
    let proxy_address = rt
        .block_on(
            evm_voice
                .getProxyAddress(sender.clone())
                .call()
                .into_future(),
        )?
        ._0;
    if proxy_address != Address::ZERO {
        Ok(proxy_address)
    } else {
        // In case no proxy address is registered, we fetch it !
        let proxy_address = rt
            .block_on(
                evm_voice
                    .getExpectedProxyAddress(sender.clone())
                    .call()
                    .into_future(),
            )?
            ._0;

        Ok(proxy_address)
    }
}
