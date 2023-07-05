use cw_orch::{
    daemon::ChainInfo,
    prelude::{
        networks::{JUNO_1, OSMO_5, PHOENIX_1, PION_1, PISCO_1, UNI_6},
        *,
    },
};

const GAS_TO_DEPLOY: u64 = 60_000_000;
pub const SUPPORTED_CHAINS: &[ChainInfo] =
    &[JUNO_1, UNI_6, PHOENIX_1, PISCO_1, PION_1, NEUTRON_1, OSMO_5];

pub const NEUTRON_1: ChainInfo = ChainInfo {
    kind: cw_orch::daemon::ChainKind::Mainnet,
    chain_id: "neutron-1",
    gas_denom: "untrn",
    gas_price: 0.001,
    grpc_urls: &["https://grpc.novel.remedy.tm.p2p.org"],
    network_info: networks::neutron::NEUTRON_NETWORK,
    lcd_url: Some("https://rest-kralum.neutron-1.neutron.org"),
    fcd_url: None,
};

pub async fn assert_wallet_balance<'a>(mut chains: &'a [ChainInfo<'a>]) -> &'a[ChainInfo<'a>]{
    if chains.is_empty() {
        chains = SUPPORTED_CHAINS;
    }
    // check that the wallet has enough gas on all the chains we want to support
    for chain_info in chains {
        let chain = DaemonAsyncBuilder::default()
            .chain(chain_info.clone())
            .build()
            .await
            .unwrap();
        let fee_token = chain.state.as_ref().chain_data.fees.fee_tokens[0].clone();
        let fee = (GAS_TO_DEPLOY as f64 * fee_token.fixed_min_gas_price) as u64;
        let bank = chain.query_client::<queriers::Bank>();
        let balance = bank
            .balance(chain.sender(), Some(fee_token.denom.clone()))
            .await
            .unwrap()[0]
            .clone();

        log::debug!(
            "Checking balance {} on chain {}, address {}. Expecting {}{}",
            balance.amount,
            chain_info.chain_id,
            chain.sender(),
            fee,
            fee_token.denom.as_str()
        );
        if fee > balance.amount.parse().unwrap() {
            panic!("Not enough funds on chain {} to deploy the contract. Needed: {}{} but only have: {}{}", chain_info.chain_id, fee, fee_token.denom.as_str(), balance.amount, fee_token.denom);
        }
        // check if we have enough funds
    }

    chains
}
