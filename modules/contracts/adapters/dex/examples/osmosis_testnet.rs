use abstract_adapter::objects::pool_id::PoolAddressBase;
use abstract_adapter::objects::{AnsAsset, AssetEntry, PoolMetadata, UniquePoolId};
use abstract_client::{AbstractClient, Namespace};
use abstract_dex_adapter::interface::DexAdapter;
use abstract_dex_standard::ans_action::DexAnsAction;
use abstract_interface::ExecuteMsgFns;
use abstract_osmosis_adapter::OSMOSIS;
use cosmwasm_std::{coin, coins, Decimal};
use cw_orch::daemon::RUNTIME;
use cw_orch::daemon::{networks::OSMO_5, Daemon};
use cw_orch::prelude::*;
use osmosis_std::types::osmosis::gamm::poolmodels::balancer::v1beta1::MsgCreateBalancerPool;
use osmosis_std::types::osmosis::gamm::v1beta1::PoolAsset;
use osmosis_std::types::osmosis::gamm::v1beta1::PoolParams;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{MsgCreateDenom, MsgMint};
use prost_13::Message;

pub const OSMO: &str = "uosmo";
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
        amount: Some(osmosis_std::types::cosmos::base::v1beta1::Coin {
            denom: new_denom(&chain),
            amount: "1000000000000".to_string(),
        }),
        mint_to_address: chain.sender_addr().to_string(),
    };

    // Create pool
    let pool_create_msg = MsgCreateBalancerPool {
        sender: chain.sender_addr().to_string(),
        pool_params: Some(PoolParams {
            swap_fee: "10000000000000000".to_string(),
            exit_fee: "0".to_string(),
            smooth_weight_change_params: None,
        }),
        pool_assets: [coin(100_000, OSMO), coin(100_000, new_denom(&chain))]
            .iter()
            .map(|c| PoolAsset {
                token: Some(osmosis_std::types::cosmos::base::v1beta1::Coin {
                    denom: c.denom.to_owned(),
                    amount: format!("{}", c.amount),
                }),
                weight: "1000000".to_string(),
            })
            .collect(),
        future_pool_governor: "".to_string(),
    };

    let response = chain.commit_any(
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
                type_url: MsgCreateBalancerPool::TYPE_URL.to_string(),
                value: pool_create_msg.encode_to_vec(),
            },
        ],
        None,
    )?;

    let pool_id: u64 = response.get_events("pool_created")[0].get_attributes("pool_id")[0]
        .value
        .parse()?;

    // We need to register some pairs and assets on the ans host contract
    // Register OSMO and ATOM assets
    deployment
        .name_service()
        .update_asset_addresses(
            vec![
                ("osmo".to_string(), cw_asset::AssetInfoBase::native(OSMO)),
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
        .update_dexes(vec![OSMOSIS.into()], vec![])
        .unwrap();

    deployment
        .name_service()
        .update_pools(
            vec![(
                PoolAddressBase::id(pool_id),
                PoolMetadata::constant_product(
                    OSMOSIS,
                    vec!["osmo".to_string(), "new_denom".to_string()],
                ),
            )],
            vec![UniquePoolId::new(1)],
        )
        .unwrap();

    Ok(())
}
fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let chain = Daemon::builder(OSMO_5).build()?;
    setup_denom_ans(chain.clone())?;

    let deployment = AbstractClient::new(chain.clone())?;

    let namespace = Namespace::new("hackerhouse")?;
    let account = deployment.fetch_or_build_account(namespace.clone(), |b| {
        b.namespace(namespace.clone())
            .install_adapter::<DexAdapter<Daemon>>()
    })?;
    let dex_adapter = account.application::<DexAdapter<Daemon>>()?;

    RUNTIME.block_on(
        chain
            .sender()
            .bank_send(&account.address()?, coins(1_000, OSMO)),
    )?;

    // swap 1_000 osmo to new_denom
    let asset = AssetEntry::new("osmo");
    let ask_asset = AssetEntry::new("new_denom");

    let swap_value = 1_000u128;

    let action = DexAnsAction::Swap {
        offer_asset: AnsAsset::new(asset, swap_value),
        ask_asset,
        max_spread: Some(Decimal::percent(30)),
        belief_price: Some(Decimal::percent(1)),
    };

    dex_adapter.module::<DexAdapter<_>>()?.ans_action(
        OSMOSIS.into(),
        action,
        account.as_ref(),
        deployment.name_service(),
    )?;

    Ok(())
}
