use abstract_interface::connection::connect_one_way_to;
use abstract_interface::*;
use abstract_std::{
    objects::{gov_type::GovernanceDetails, TruncatedChainId, UncheckedChannelEntry},
    IBC_CLIENT, ICS20,
};
use cosmwasm_std::{coin, coins, to_json_binary, BankMsg};
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::*;

type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

pub const SOURCE_CHAIN_ID: &str = "source-1";
pub const DEST_CHAIN_ID: &str = "dest-1";
pub const NEW_DUMMY_NAME: &str = "Name that no-one could have thought of";

#[test]
fn transfer_with_account_rename_message() -> AResult {
    let interchain =
        MockBech32InterchainEnv::new(vec![(SOURCE_CHAIN_ID, "source"), (DEST_CHAIN_ID, "dest")]);

    let src = interchain.get_chain(SOURCE_CHAIN_ID)?;
    let dst = interchain.get_chain(DEST_CHAIN_ID)?;

    let src_abstr = Abstract::deploy_on_mock(src.clone())?;
    let dest_abstr = Abstract::deploy_on_mock(dst.clone())?;

    connect_one_way_to(
        &src_abstr.call_as(&Abstract::mock_admin(&src)),
        &dest_abstr.call_as(&Abstract::mock_admin(&dst)),
        &interchain,
    )?;

    let ics20_channel = interchain
        .create_channel(
            SOURCE_CHAIN_ID,
            DEST_CHAIN_ID,
            &PortId::transfer(),
            &PortId::transfer(),
            "ics20-1",
            Some(cosmwasm_std::IbcOrder::Unordered),
        )?
        .interchain_channel;

    // Update the channels on the src side (for sending tokens)
    src_abstr
        .ans_host
        .call_as(&Abstract::mock_admin(&src))
        .update_channels(
            vec![(
                UncheckedChannelEntry {
                    connected_chain: TruncatedChainId::from_chain_id(DEST_CHAIN_ID).to_string(),
                    protocol: ICS20.to_string(),
                },
                ics20_channel
                    .get_chain(SOURCE_CHAIN_ID)?
                    .channel
                    .unwrap()
                    .to_string(),
            )],
            vec![],
        )?;

    // Update the channels on the src side (for sending back tokens)
    dest_abstr
        .ans_host
        .call_as(&Abstract::mock_admin(&dst))
        .update_channels(
            vec![(
                UncheckedChannelEntry {
                    connected_chain: TruncatedChainId::from_chain_id(SOURCE_CHAIN_ID).to_string(),
                    protocol: ICS20.to_string(),
                },
                ics20_channel
                    .get_chain(DEST_CHAIN_ID)?
                    .channel
                    .unwrap()
                    .to_string(),
            )],
            vec![],
        )?;

    let account = AccountI::create(
        &src_abstr,
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            ..Default::default()
        },
        GovernanceDetails::Monarchy {
            monarch: src.sender_addr().to_string(),
        },
        &[],
    )?;
    account.set_ibc_status(true)?;

    pub const INITIAL_AMOUNT: u128 = 100_000;
    pub const LOCAL_TRANSFER_AMOUNT: u128 = 50_000;
    let funds_to_transfer = coin(INITIAL_AMOUNT, "usource");
    let funds_after_ics20 = coin(INITIAL_AMOUNT, "usource1");

    src.add_balance(
        &account.address()?,
        vec![funds_to_transfer.clone(), funds_after_ics20.clone()],
    )?;

    // Here we send some funds on the remote account and then change the name of the account on callback
    let tx_response = account.execute_on_module(
        IBC_CLIENT,
        abstract_std::ibc_client::ExecuteMsg::SendFundsWithActions {
            host_chain: TruncatedChainId::from_chain_id(&dst.chain_id()),
            actions: vec![to_json_binary(
                &abstract_account::msg::ExecuteMsg::Execute {
                    msgs: vec![BankMsg::Send {
                        to_address: src.sender_addr().to_string(),
                        amount: coins(LOCAL_TRANSFER_AMOUNT, "usource1"),
                    }
                    .into()],
                },
            )?],
        },
        vec![funds_to_transfer.clone()],
    )?;
    let src_account_balance = src.balance(&account.address()?, None)?;
    assert_eq!(src_account_balance, coins(INITIAL_AMOUNT, "usource1"));
    let result = interchain.await_and_check_packets(SOURCE_CHAIN_ID, tx_response)?;
    println!("{:?}", result);
    let src_account_balance = src.balance(&account.address()?, None)?;
    assert_eq!(
        src_account_balance,
        coins(LOCAL_TRANSFER_AMOUNT, "usource1")
    );

    let dst_host_balance = dst.balance(&dest_abstr.ibc.host.address()?, None)?;
    assert!(dst_host_balance.is_empty());

    // We fetch the remote account balance
    let mut remote_account_id = account.id()?;
    remote_account_id.push_chain(TruncatedChainId::from_chain_id(SOURCE_CHAIN_ID));

    let remote_account = AccountI::load_from(&dest_abstr, remote_account_id)?;

    let dest_account_balance = dst.balance(&remote_account.address()?, None)?;
    assert_eq!(dest_account_balance.len(), 1);

    // Finally, we assert the sender received their tokens
    let src_account_balance = src.balance(&src.sender_addr(), None)?;
    assert_eq!(
        src_account_balance,
        coins(LOCAL_TRANSFER_AMOUNT, "usource1")
    );

    Ok(())
}

#[test]
fn transfer_with_account_rename_message_timeout() -> AResult {
    let interchain =
        MockBech32InterchainEnv::new(vec![(SOURCE_CHAIN_ID, "source"), (DEST_CHAIN_ID, "dest")]);

    let src = interchain.get_chain(SOURCE_CHAIN_ID)?;
    let dst = interchain.get_chain(DEST_CHAIN_ID)?;

    let src_abstr = Abstract::deploy_on_mock(src.clone())?;
    let dest_abstr = Abstract::deploy_on_mock(dst.clone())?;

    connect_one_way_to(
        &src_abstr.call_as(&Abstract::mock_admin(&src)),
        &dest_abstr.call_as(&Abstract::mock_admin(&dst)),
        &interchain,
    )?;

    let ics20_channel = interchain
        .create_channel(
            SOURCE_CHAIN_ID,
            DEST_CHAIN_ID,
            &PortId::transfer(),
            &PortId::transfer(),
            "ics20-1",
            Some(cosmwasm_std::IbcOrder::Unordered),
        )?
        .interchain_channel;

    // Update the channels on the src side (for sending tokens)
    src_abstr
        .ans_host
        .call_as(&Abstract::mock_admin(&src))
        .update_channels(
            vec![(
                UncheckedChannelEntry {
                    connected_chain: TruncatedChainId::from_chain_id(DEST_CHAIN_ID).to_string(),
                    protocol: ICS20.to_string(),
                },
                ics20_channel
                    .get_chain(SOURCE_CHAIN_ID)?
                    .channel
                    .unwrap()
                    .to_string(),
            )],
            vec![],
        )?;

    // Update the channels on the src side (for sending back tokens)
    dest_abstr
        .ans_host
        .call_as(&Abstract::mock_admin(&dst))
        .update_channels(
            vec![(
                UncheckedChannelEntry {
                    connected_chain: TruncatedChainId::from_chain_id(SOURCE_CHAIN_ID).to_string(),
                    protocol: ICS20.to_string(),
                },
                ics20_channel
                    .get_chain(DEST_CHAIN_ID)?
                    .channel
                    .unwrap()
                    .to_string(),
            )],
            vec![],
        )?;

    let account = AccountI::create(
        &src_abstr,
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            ..Default::default()
        },
        GovernanceDetails::Monarchy {
            monarch: src.sender_addr().to_string(),
        },
        &[],
    )?;

    account.set_ibc_status(true)?;
    let funds_to_transfer = coin(100_000, "usource");

    src.add_balance(&account.address()?, vec![funds_to_transfer.clone()])?;

    // Here we send some funds on the remote account and then change the name of the account on callback
    let tx_response = account.execute_on_module(
        IBC_CLIENT,
        abstract_std::ibc_client::ExecuteMsg::SendFundsWithActions {
            host_chain: TruncatedChainId::from_chain_id(&dst.chain_id()),
            actions: vec![to_json_binary(
                &abstract_account::msg::ExecuteMsg::UpdateInfo {
                    name: Some(NEW_DUMMY_NAME.to_string()),
                    description: None,
                    link: None,
                },
            )?],
        },
        vec![funds_to_transfer.clone()],
    )?;

    // Trigger timeout
    dst.wait_seconds(60 * 60 * 24)?;

    interchain
        .await_packets(SOURCE_CHAIN_ID, tx_response)?
        .assert()
        .unwrap_err();

    let src_account_balance = src.balance(&account.address()?, None)?;
    assert_eq!(src_account_balance.len(), 1);

    let dst_host_balance = dst.balance(&dest_abstr.ibc.host.address()?, None)?;
    assert!(dst_host_balance.is_empty());

    // We fetch the remote account balance
    let mut remote_account_id = account.id()?;
    remote_account_id.push_chain(TruncatedChainId::from_chain_id(SOURCE_CHAIN_ID));

    // Remote account not created
    AccountI::load_from(&dest_abstr, remote_account_id).unwrap_err();

    // Finally, we assert the name change that happened after the callback
    assert_ne!(account.info()?.info.name, Some(NEW_DUMMY_NAME.to_string()));

    Ok(())
}
