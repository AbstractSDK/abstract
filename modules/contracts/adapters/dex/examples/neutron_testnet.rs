use abstract_adapter::objects::pool_id::PoolAddressBase;
use abstract_adapter::objects::{AnsAsset, AssetEntry, PoolMetadata};
use abstract_client::{AbstractClient, Account, Application, Namespace};
use abstract_dex_adapter::interface::DexAdapter;
use abstract_dex_standard::ans_action::DexAnsAction;
use abstract_interface::ExecuteMsgFns;
use abstract_neutron_dex_adapter::NEUTRON;
use cosmwasm_std::{coin, Decimal};
use cw_orch::daemon::{networks::PION_1, Daemon};
use cw_orch::prelude::*;
use neutron_std::types::neutron::dex::DepositOptions;
use neutron_std::types::{
    neutron::dex::MsgDeposit,
    osmosis::tokenfactory::v1beta1::{MsgCreateDenom, MsgMint},
};
use prost::Message;

pub const NTRN: &str = "untrn";
pub const SUB_DENOM: &str = "pooled";

fn new_denom(chain: &Daemon) -> String {
    format!("factory/{}/{}", chain.sender_addr(), SUB_DENOM)
}

#[allow(clippy::type_complexity)]
fn setup_denom_ans(chain: Daemon) -> anyhow::Result<()> {
    let deployment = AbstractClient::new(chain.clone())?;

    // Create some liquidity
    let denom_creation = MsgCreateDenom {
        sender: chain.sender_addr().to_string(),
        subdenom: SUB_DENOM.to_string(),
    };
    // Mint to myself
    let denom_mint = MsgMint {
        sender: chain.sender_addr().to_string(),
        amount: Some(neutron_std::types::cosmos::base::v1beta1::Coin {
            denom: new_denom(&chain),
            amount: "1000000000000".to_string(),
        }),
        mint_to_address: chain.sender_addr().to_string(),
    };

    // Deposit some initial liquidity (create pool)
    let pool_create_msg = MsgDeposit {
        token_a: NTRN.to_string(),
        token_b: new_denom(&chain).to_string(),
        amounts_a: vec!["100000".to_string()],
        amounts_b: vec!["100000".to_string()],
        creator: chain.sender_addr().to_string(),
        receiver: chain.sender_addr().to_string(),
        fees: vec![0],
        options: vec![DepositOptions {
            disable_autoswap: false,
            fail_tx_on_bel: false,
        }],
        tick_indexes_a_to_b: vec![0],
    };

    chain.commit_any(
        vec![
            prost_types::Any {
                type_url: MsgCreateDenom::TYPE_URL.to_string(),
                value: denom_creation.encode_to_vec(),
            },
            prost_types::Any {
                type_url: MsgMint::TYPE_URL.to_string(),
                value: denom_mint.encode_to_vec(),
            },
            prost_types::Any {
                type_url: MsgDeposit::TYPE_URL.to_string(),
                value: pool_create_msg.encode_to_vec(),
            },
        ],
        None,
    )?;

    // We need to register some pairs and assets on the ans host contract
    // Register NTRN and ATOM assets
    deployment
        .name_service()
        .update_asset_addresses(
            vec![
                ("ntrn".to_string(), cw_asset::AssetInfoBase::native(NTRN)),
                (
                    "new_denom".to_string(),
                    cw_asset::AssetInfoBase::native(new_denom(&chain)),
                ),
            ],
            vec![],
        )
        .unwrap();

    deployment
        .name_service()
        .update_dexes(vec![NEUTRON.into()], vec![])
        .unwrap();

    deployment
        .name_service()
        .update_pools(
            vec![(
                PoolAddressBase::id(0u64),
                PoolMetadata::constant_product(
                    NEUTRON,
                    vec!["ntrn".to_string(), "new_denom".to_string()],
                ),
            )],
            vec![],
        )
        .unwrap();

    Ok(())
}
fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let chain = Daemon::builder(PION_1).build()?;
    // setup_denom_ans(chain.clone())?;

    let deployment = AbstractClient::new(chain.clone())?;

    let namespace = Namespace::new("hackerhouse")?;
    let account = deployment.fetch_or_build_account(namespace.clone(), |b| {
        b.namespace(namespace.clone())
            .install_adapter::<DexAdapter<Daemon>>()
    })?;
    let dex_adapter = account.application::<DexAdapter<Daemon>>()?;

    // swap 1_000 ntrn to new_denom
    let asset = AssetEntry::new("ntrn");
    let ask_asset = AssetEntry::new("new_denom");

    let swap_value = 1_000u128;

    let action = DexAnsAction::Swap {
        offer_asset: AnsAsset::new(asset, swap_value),
        ask_asset,
        max_spread: Some(Decimal::percent(30)),
        belief_price: Some(Decimal::percent(1)),
    };

    dex_adapter.module::<DexAdapter<_>>()?.ans_action(
        NEUTRON.into(),
        action,
        account.as_ref(),
        deployment.name_service(),
    )?;

    Ok(())
}
