use crate::commands::{query_asks_by_seller, query_bids_by_bidder};
use crate::contract::{execute, instantiate};
#[cfg(test)]
use abstract_std::profile_marketplace::{state::*, *};
use bs_std::NATIVE_DENOM;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, Addr, DepsMut, Timestamp, Uint128};

const CREATOR: &str = "creator";
const TOKEN_ID: &str = "name";
const TOKEN_ID_NEXT: &str = "name2";

// Governance parameters
const TRADING_FEE_BASIS_POINTS: u64 = 200; // 2%

#[test]
fn ask_indexed_map() {
    let mut deps = mock_dependencies();
    let seller = Addr::unchecked("seller");

    let env = mock_env();

    let ask = Ask {
        token_id: TOKEN_ID.to_string(),
        id: 1,
        seller: seller.clone(),
        renewal_time: env.block.time,
        renewal_fund: Uint128::zero(),
    };
    let key = ask_key(TOKEN_ID);
    let res = asks().save(deps.as_mut().storage, key.clone(), &ask);
    assert!(res.is_ok());

    let ask2 = Ask {
        token_id: TOKEN_ID_NEXT.to_string(),
        id: 2,
        seller: seller.clone(),
        renewal_time: env.block.time,
        renewal_fund: Uint128::zero(),
    };
    let key2 = ask_key(TOKEN_ID_NEXT);
    let res = asks().save(deps.as_mut().storage, key2, &ask2);
    assert!(res.is_ok());

    let res = asks().load(deps.as_ref().storage, key);
    assert_eq!(res.unwrap(), ask);

    let res = query_asks_by_seller(deps.as_ref(), seller, None, None).unwrap();
    assert_eq!(res.len(), 2);
    assert_eq!(res[0], ask);
}

#[test]
fn bid_indexed_map() {
    let mut deps = mock_dependencies();
    let bidder = Addr::unchecked("bidder");

    let bid = Bid {
        token_id: TOKEN_ID.to_string(),
        bidder: bidder.clone(),
        amount: Uint128::from(500u128),
        created_time: Timestamp::from_seconds(6),
    };
    let key = bid_key(TOKEN_ID, &bidder);
    let res = bids().save(deps.as_mut().storage, key.clone(), &bid);
    assert!(res.is_ok());

    let bid2 = Bid {
        token_id: TOKEN_ID_NEXT.to_string(),
        bidder: bidder.clone(),
        amount: Uint128::from(500u128),
        created_time: Timestamp::from_seconds(6),
    };
    let key2 = bid_key(TOKEN_ID_NEXT, &bidder);
    let res = bids().save(deps.as_mut().storage, key2, &bid2);
    assert!(res.is_ok());

    let res = bids().load(deps.as_ref().storage, key);
    assert_eq!(res.unwrap(), bid);

    let res = query_bids_by_bidder(deps.as_ref(), bidder, None, None).unwrap();
    assert_eq!(res.len(), 2);
    assert_eq!(res[0], bid);
}

fn setup_contract(deps: DepsMut) {
    let msg = InstantiateMsg {
        trading_fee_bps: TRADING_FEE_BASIS_POINTS,
        min_price: Uint128::from(5u128),
        ask_interval: 60,
        factory: Addr::unchecked("factory".to_string()),
        collection:  Addr::unchecked("profile_collection".to_string()),
    };
    let info = mock_info(CREATOR, &[]);
    let res = instantiate(deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        trading_fee_bps: TRADING_FEE_BASIS_POINTS,
        min_price: Uint128::from(5u128),
        ask_interval: 60,
        factory: Addr::unchecked("factory".to_string()),
        collection: Addr::unchecked("profile_collection".to_string()),
    };
    let info = mock_info("creator", &coins(1000, NATIVE_DENOM));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn bad_fees_initialization() {
    let mut deps = mock_dependencies();

    // throw error if trading fee bps > 100%
    let msg = InstantiateMsg {
        trading_fee_bps: 10001,
        min_price: Uint128::from(5u128),
        ask_interval: 60,
        factory: Addr::unchecked("factory".to_string()),
        collection:  Addr::unchecked("profile_collection".to_string()),
    };
    let info = mock_info("creator", &coins(1000, NATIVE_DENOM));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_err());
}

#[test]
fn try_set_bid() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    let bidder = mock_info("bidder", &coins(1000, NATIVE_DENOM));

    // Bidder calls SetBid before an Ask is set, fails
    let set_bid_msg = ExecuteMsg::SetBid {
        token_id: TOKEN_ID.to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), bidder, set_bid_msg);
    assert!(res.is_err());
}

#[test]
fn try_set_ask() {
    let mut deps = mock_dependencies();
    setup_contract(deps.as_mut());

    let set_ask = ExecuteMsg::SetAsk {
        token_id: TOKEN_ID.to_string(),
        seller: CREATOR.to_string(),
    };

    // Reject if not called by the media owner
    let not_allowed = mock_info("random", &[]);
    let err = execute(deps.as_mut(), mock_env(), not_allowed, set_ask);
    assert!(err.is_err());
}
