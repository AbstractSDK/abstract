use abstract_client::{AbstractClient, Namespace};
use abstract_ica_client::chain_types::evm::types::{UCS01_FORWARDER_CONTRACT, UCS01_PROTOCOL};
use abstract_interface::{ExecuteMsgFns, IcaClient};
use abstract_std::{
    ica_client::{IcaAction, IcaActionResult, IcaExecute, InstantiateMsg, QueryMsg, QueryMsgFns},
    objects::{UncheckedChannelEntry, UncheckedContractEntry},
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
            chain.rt_handle.block_on(
                chain
                    .sender()
                    .bank_send(&account.address()?, vec![account_coins.clone()]),
            )?;
        }
    }

    let evm_config = union_connector::networks::get_remote_evm_config(CHAIN_NAME).unwrap();
    let evm_note = EvmNote::new(chain.clone());

    // We need to register the EVM note here (already existent in state.json)
    abs.registry().register_natives(vec![(
        evm_note.as_instance(),
        evm_note::contract::CONTRACT_VERSION.to_string(),
    )])?;

    // We need to register the IBC channels

    abs.name_service().update_channels(
        vec![
            (
                UncheckedChannelEntry::new(evm_config.chain_name, UCS01_PROTOCOL),
                evm_config.ics20_dst_channel.to_string(),
            ),
            (
                UncheckedChannelEntry::new(
                    evm_config.chain_name,
                    format!("{}/counterparty", UCS01_PROTOCOL).as_str(),
                ),
                evm_config.ics20_src_channel.to_string(),
            ),
        ],
        vec![],
    )?;
    abs.name_service().update_contract_addresses(
        vec![(
            UncheckedContractEntry {
                contract: UCS01_FORWARDER_CONTRACT.to_string(),
                protocol: UCS01_PROTOCOL.to_string(),
            },
            evm_config.ucs01_forwarder.to_string(),
        )],
        vec![],
    )?;

    let interchain = UnionInterchainEnv::new(chain.clone(), &evm_config);

    let remote_address = UnionInterchainEnv::get_remote_address(
        &evm_note,
        &interchain,
        evm_config.chain_id,
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

        let send_back_msgs = interchain.ibc_send_back_msgs(
            evm_config.chain_id.to_string(),
            muno_erc20_addr,
            account_coins.amount,
            account.address()?,
        )?;

        let ica_action = abs.ica_client().ica_action(
            account.address()?.to_string(),
            vec![IcaAction::Execute(IcaExecute::Evm {
                msgs: send_back_msgs,
                callback: None,
            })],
            CHAIN_NAME.parse()?,
        )?;

        // In case we want to execute the transaction
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
