use crate::dapp_base::common::TEST_CREATOR;
use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, Addr, Empty, Timestamp, Uint128};
use pandora_os::core::treasury::msg as TreasuryMsg;
use pandora_os::native::memory::msg as MemoryMsg;
use terra_mocks::TerraMockQuerier;
use terra_multi_test::{App, BankKeeper, ContractWrapper, Executor};
use terraswap::asset::{AssetInfo, PairInfo};

pub struct BaseContracts {
    pub whale: Addr,
    pub memory: Addr,
    pub treasury: Addr,
    pub whale_ust_pair: Addr,
    pub whale_ust: Addr,
}

/// Creates the basic contract instances needed to test the dapp.
/// Whale token, Memory, Treasury, Whale/UST pair, Whale/UST LP
pub fn init_contracts(app: &mut App) -> BaseContracts {
    let owner = Addr::unchecked(TEST_CREATOR);

    // Instantiate WHALE Token Contract
    let cw20_token_contract = Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));

    let cw20_token_code_id = app.store_code(cw20_token_contract);

    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("Whale token"),
        symbol: String::from("WHALE"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let whale_token_instance = app
        .instantiate_contract(
            cw20_token_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("WHALE"),
            None,
        )
        .unwrap();

    // Upload Treasury Contract
    let treasury_contract = Box::new(ContractWrapper::new(
        treasury::contract::execute,
        treasury::contract::instantiate,
        treasury::contract::query,
    ));

    let treasury_code_id = app.store_code(treasury_contract);

    let treasury_instantiate_msg = TreasuryMsg::InstantiateMsg {};

    // Instantiate Treasury Contract
    let treasury_instance = app
        .instantiate_contract(
            treasury_code_id,
            owner.clone(),
            &treasury_instantiate_msg,
            &[],
            "Treasury",
            None,
        )
        .unwrap();

    // Upload Memory Contract
    let memory_contract = Box::new(ContractWrapper::new(
        memory::contract::execute,
        memory::contract::instantiate,
        memory::contract::query,
    ));

    let memory_code_id = app.store_code(memory_contract);

    let memory_instantiate_msg = MemoryMsg::InstantiateMsg {};

    // Init contract
    let memory_instance = app
        .instantiate_contract(
            memory_code_id,
            owner.clone(),
            &memory_instantiate_msg,
            &[],
            "Memory",
            None,
        )
        .unwrap();

    // Instantiate the terraswap pair
    let (pair, lp) = instantiate_pair(app, &owner.clone(), &whale_token_instance);

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1571797419);
    });

    BaseContracts {
        treasury: treasury_instance,
        memory: memory_instance,
        whale: whale_token_instance,
        whale_ust_pair: pair,
        whale_ust: lp,
    }
}

pub fn mock_app() -> App<Empty> {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();
    let custom_querier: TerraMockQuerier =
        TerraMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, &[])]));

    App::new(api, env.block, bank, MockStorage::new(), custom_querier)
    // let custom_handler = CachingCustomHandler::<CustomMsg, Empty>::new();
    // AppBuilder::new().with_custom(custom_handler).build()
}

/// Create terraswap WHALE/UST pair
fn instantiate_pair(
    mut router: &mut App,
    owner: &Addr,
    whale_token_instance: &Addr,
) -> (Addr, Addr) {
    let token_contract_code_id = store_token_code(&mut router);

    let pair_contract_code_id = store_pair_code(&mut router);

    let msg = terraswap::pair::InstantiateMsg {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfo::Token {
                contract_addr: whale_token_instance.to_string(),
            },
        ],
        token_code_id: token_contract_code_id,
    };

    let pair = router
        .instantiate_contract(
            pair_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIRRR"),
            None,
        )
        .unwrap();

    let res: PairInfo = router
        .wrap()
        .query_wasm_smart(pair.clone(), &terraswap::pair::QueryMsg::Pair {})
        .unwrap();
    assert_eq!("Contract #3", res.contract_addr);
    assert_eq!("Contract #4", res.liquidity_token);

    (pair, Addr::unchecked(res.liquidity_token))
}

/// Whitelist a dapp on the treasury
pub fn whitelist_dapp(app: &mut App, owner: &Addr, treasury_instance: &Addr, dapp_instance: &Addr) {
    let msg = TreasuryMsg::ExecuteMsg::AddDApp {
        dapp: dapp_instance.to_string(),
    };
    let _res = app
        .execute_contract(owner.clone(), treasury_instance.clone(), &msg, &[])
        .unwrap();
    // Check if it was added
    let resp: TreasuryMsg::ConfigResponse = app
        .wrap()
        .query_wasm_smart(treasury_instance, &TreasuryMsg::QueryMsg::Config {})
        .unwrap();

    // Check config
    assert!(resp.dapps.contains(&dapp_instance.to_string()));
}

/// Mint Whale tokens
pub fn mint_some_whale(
    app: &mut App,
    owner: Addr,
    whale_token_instance: Addr,
    amount: Uint128,
    to: String,
) {
    let msg = cw20::Cw20ExecuteMsg::Mint {
        recipient: to.clone(),
        amount,
    };
    let res = app
        .execute_contract(owner.clone(), whale_token_instance.clone(), &msg, &[])
        .unwrap();
    assert_eq!(res.events[1].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[1].attributes[2], attr("to", to));
    assert_eq!(res.events[1].attributes[3], attr("amount", amount));
}

pub fn store_token_code(app: &mut App) -> u64 {
    let whale_token_contract = Box::new(ContractWrapper::new(
        terraswap_token::contract::execute,
        terraswap_token::contract::instantiate,
        terraswap_token::contract::query,
    ));

    app.store_code(whale_token_contract)
}

pub fn store_pair_code(app: &mut App) -> u64 {
    let pair_contract = Box::new(
        ContractWrapper::new(
            terraswap_pair::contract::execute,
            terraswap_pair::contract::instantiate,
            terraswap_pair::contract::query,
        )
        .with_reply(terraswap_pair::contract::reply),
    );

    app.store_code(pair_contract)
}

#[allow(dead_code)]
fn store_factory_code(app: &mut App) -> u64 {
    let factory_contract = Box::new(
        ContractWrapper::new(
            terraswap_factory::contract::execute,
            terraswap_factory::contract::instantiate,
            terraswap_factory::contract::query,
        )
        .with_reply(terraswap_factory::contract::reply),
    );

    app.store_code(factory_contract)
}
