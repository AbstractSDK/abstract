/// This module includes expected failure cases for the contract
/// as well as asserting the behavior of the contract when an exception occurs
/// Exception cases tested:
/// - Collection with out any routes set by the owner
/// - Collection, with established routes but nothing to collect
/// - Adding a route for collection but the pools are not valid
/// - A non owner attempting to perform a spend to an entity
/// - A non owner attempting to update the routes, for example the trader setting the routes it will trade with
mod fail_cases {
    use crate::msg::AssetWithLimit;
    use crate::multitest::suite::SuiteBuilder;

    use cosmwasm_std::{Addr, Uint128};
    use wyndex::asset::token_asset_info;
    use wyndex::asset::AssetInfo;

    #[test]
    fn collect_without_any_routes() {
        let mut suite = SuiteBuilder::new().build();

        let owner = suite.owner.clone();

        let token = suite.instantiate_token(&owner, "wynd");

        let token_to_swap = suite.instantiate_token(&owner, "notwynd");

        let nominated_trader_contract = suite
            .setup_trader(
                Addr::unchecked("owner"),
                AssetInfo::Token(token.to_string()),
            )
            .unwrap();
        let assets_to_take = vec![AssetWithLimit {
            info: AssetInfo::Token(token_to_swap.to_string()),
            limit: None,
        }];

        // Mint some cw20 into the trader contract
        suite
            .mint_cw20(
                &owner,
                &token_to_swap,
                1_000_000u128,
                nominated_trader_contract.as_ref(),
            )
            .unwrap();

        let err = suite
            .trade_collected_assets(
                &suite.trader.clone(),
                nominated_trader_contract.clone(),
                assets_to_take,
            )
            .unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            "Cannot swap contract2. No swap destinations"
        );
    }
    #[test]
    fn collect_without_any_accrued_assets() {
        let mut suite = SuiteBuilder::new().build();

        let owner = suite.owner.clone();

        let token = suite.instantiate_token(&owner, "wynd");

        let token_to_swap = suite.instantiate_token(&owner, "notwynd");

        let nominated_trader_contract = suite
            .setup_trader(
                Addr::unchecked("owner"),
                AssetInfo::Token(token.to_string()),
            )
            .unwrap();

        // Mint some cw20 into the trader contract
        suite
            .mint_cw20(
                &owner,
                &token_to_swap,
                1_000_000u128,
                nominated_trader_contract.as_ref(),
            )
            .unwrap();

        // Not necessarily a sad case but we should have a test over calling collect with no assets
        // Call collect for a native token, we have no native tokens yet so this should fail as no swap collection messages can be built.
        let err = suite
            .trade_collected_assets(
                &suite.trader.clone(),
                nominated_trader_contract.clone(),
                vec![AssetWithLimit {
                    info: AssetInfo::Token(token.to_string()),
                    limit: None,
                }],
            )
            .unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            "not enough pools to route assets to desired base token"
        );
    }

    #[test]
    fn add_route_with_non_existant_pools() {
        let mut suite = SuiteBuilder::new().build();

        let owner = suite.owner.clone();

        let token = suite.instantiate_token(&owner, "wynd");

        let token_to_swap = suite.instantiate_token(&owner, "notwynd");

        let nominated_trader_contract = suite
            .setup_trader(
                Addr::unchecked("owner"),
                AssetInfo::Token(token.to_string()),
            )
            .unwrap();

        // Mint some cw20 into the trader contract
        suite
            .mint_cw20(
                &owner,
                &token_to_swap,
                1_000_000u128,
                nominated_trader_contract.as_ref(),
            )
            .unwrap();

        let new_route = Some(vec![(
            token_asset_info(token_to_swap.as_ref()),
            token_asset_info(token.as_ref()),
        )]);

        let err = suite
            .update_routes(
                &suite.owner.to_string(),
                nominated_trader_contract.clone(),
                new_route,
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            "Invalid route. Pool contract2 to contract1 not found"
        );
    }

    #[test]
    fn spend_from_a_non_owner() {
        let mut suite = SuiteBuilder::new().build();

        let owner = suite.owner.clone();

        let token = suite.instantiate_token(&owner, "wynd");

        let token_to_swap = suite.instantiate_token(&owner, "notwynd");

        let nominated_trader_contract = suite
            .setup_trader(
                Addr::unchecked("owner"),
                AssetInfo::Token(token.to_string()),
            )
            .unwrap();

        // Mint some cw20 into the trader contract
        suite
            .mint_cw20(
                &owner,
                &token_to_swap,
                1_000_000u128,
                nominated_trader_contract.as_ref(),
            )
            .unwrap();

        // Mint some more cw20 into the trader contract
        suite
            .mint_cw20(
                &owner,
                &token,
                1_000_000u128,
                nominated_trader_contract.as_ref(),
            )
            .unwrap();
        let err = suite
            .spend(
                &suite.trader.to_string(),
                nominated_trader_contract.clone(),
                suite.trader.clone(),
                Uint128::new(100_000u128),
            )
            .unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            "Unauthorized Only the owner can submit a spend"
        );
    }
    #[test]
    fn non_owner_update_routes() {
        let mut suite = SuiteBuilder::new().build();

        let owner = suite.owner.clone();

        let token = suite.instantiate_token(&owner, "wynd");

        let token_to_swap = suite.instantiate_token(&owner, "notwynd");

        let nominated_trader_contract = suite
            .setup_trader(
                Addr::unchecked("owner"),
                AssetInfo::Token(token.to_string()),
            )
            .unwrap();

        let new_route = Some(vec![(
            token_asset_info(token.as_ref()),
            token_asset_info(token_to_swap.as_ref()),
        )]);
        let err = suite
            .update_routes(
                &suite.trader.to_string(),
                nominated_trader_contract,
                new_route,
                None,
            )
            .unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            "Unauthorized Only the owner can update routes"
        );
    }
}

mod trader_flows {
    use crate::msg::AssetWithLimit;
    use crate::multitest::suite::SuiteBuilder;

    use cosmwasm_std::{assert_approx_eq, coin, Addr, Uint128};
    use wyndex::asset::{native_asset_info, token_asset_info};
    use wyndex::{asset::AssetInfo, factory::PairType};

    #[test]
    // This test ensures that an owner can both add and remove assets as designated routes
    // that can be used to collect assets from the trader contract
    fn update_routes_works() {
        let ujuno = "ujuno";

        let mut suite = SuiteBuilder::new().build();

        let owner = suite.owner.clone();

        let token = suite.instantiate_token(&owner, "wynd");

        let token_to_swap = suite.instantiate_token(&owner, "notwynd");

        let nominated_trader_contract = suite
            .setup_trader(
                Addr::unchecked("owner"),
                AssetInfo::Token(token.to_string()),
            )
            .unwrap();

        // create LP for just instantiated tokens
        suite
            .create_pair_and_provide_liquidity(
                PairType::Xyk {},
                (AssetInfo::Token(token.to_string()), 100_000_000u128),
                (AssetInfo::Native(ujuno.to_owned()), 100_000_000u128),
                vec![coin(100_000_000, ujuno)],
            )
            .unwrap();

        // create LP for just instantiated tokens
        suite
            .create_pair_and_provide_liquidity(
                PairType::Xyk {},
                (AssetInfo::Token(token_to_swap.to_string()), 100_000_000u128),
                (AssetInfo::Native(ujuno.to_owned()), 100_000_000u128),
                vec![coin(100_000_000, ujuno)],
            )
            .unwrap();

        suite
            .create_pair_and_provide_liquidity(
                PairType::Xyk {},
                (AssetInfo::Token(token_to_swap.to_string()), 100_000_000u128),
                (AssetInfo::Token(token.to_string()), 100_000_000u128),
                vec![],
            )
            .unwrap();

        // Mint some cw20 for user to exchange
        suite
            .mint_cw20(
                &owner,
                &token_to_swap,
                1_000_000u128,
                nominated_trader_contract.as_ref(),
            )
            .unwrap();

        let new_route = Some(vec![(
            token_asset_info(token.as_ref()),
            token_asset_info(token_to_swap.as_ref()),
        )]);

        suite
            .update_routes(
                &suite.owner.to_string(),
                nominated_trader_contract.clone(),
                new_route,
                None,
            )
            .unwrap();
        // Query routes to confirm
        let routes = suite
            .query_routes(&nominated_trader_contract.clone())
            .unwrap();

        // Enforce our update was taken into account
        assert_eq!(routes, vec![(token.to_string(), token_to_swap.to_string())]);

        // Next remove the route
        suite
            .update_routes(
                &suite.owner.to_string(),
                nominated_trader_contract.clone(),
                None,
                Some(vec![token_asset_info(token.as_ref())]),
            )
            .unwrap();
        // Query routes to confirm
        let routes = suite
            .query_routes(&nominated_trader_contract.clone())
            .unwrap();

        // Enforce our update was taken into account
        assert!(routes.is_empty());

        // Add a new route
        let new_route = Some(vec![(
            token_asset_info(token.as_ref()),
            native_asset_info(ujuno),
        )]);

        suite
            .update_routes(
                &suite.owner.to_string(),
                nominated_trader_contract.clone(),
                new_route,
                None,
            )
            .unwrap();
        // Query routes to confirm
        let routes = suite
            .query_routes(&nominated_trader_contract.clone())
            .unwrap();

        // Enforce our update was taken into account
        assert_eq!(routes, vec![(token.to_string(), ujuno.to_string())]);
    }

    #[test]
    // This test is the ensure that we can also use a Native as the base token that collected fees are swapped too
    // Assuming the relevant pools exist and the routes are set we should be able to use a Native token OR a cw20 as the base token
    // Maybe we deploy 2 or 3 of these and have fees split between them
    // to be collected into different base tokens
    fn collect_into_native_token() {
        let ujuno = "ujuno";

        let mut suite = SuiteBuilder::new().build();

        let owner = suite.owner.clone();

        let token_to_swap = suite.instantiate_token(&owner, "notwynd");

        let nominated_trader_contract = suite
            .setup_trader(
                Addr::unchecked("owner"),
                AssetInfo::Native(ujuno.to_string()),
            )
            .unwrap();
        // Create 1 pair, cw20 token to swap and the desired native token
        suite
            .create_pair_and_provide_liquidity(
                PairType::Xyk {},
                (AssetInfo::Token(token_to_swap.to_string()), 100_000_000u128),
                (AssetInfo::Native(ujuno.to_string()), 100_000_000u128),
                vec![coin(100_000_000, ujuno)],
            )
            .unwrap();

        // Mint some cw20 for user to exchange
        suite
            .mint_cw20(
                &owner,
                &token_to_swap,
                1_000_000u128,
                nominated_trader_contract.as_ref(),
            )
            .unwrap();
        // Setup 1 route
        let new_route = Some(vec![(
            native_asset_info(ujuno),
            token_asset_info(token_to_swap.as_ref()),
        )]);
        suite
            .update_routes(
                &suite.owner.to_string(),
                nominated_trader_contract.clone(),
                new_route,
                None,
            )
            .unwrap();

        let assets_to_take = vec![AssetWithLimit {
            info: AssetInfo::Token(token_to_swap.to_string()),
            limit: None,
        }];

        let res = suite
            .trade_collected_assets(
                &suite.trader.clone(),
                nominated_trader_contract.clone(),
                assets_to_take,
            )
            .unwrap();
        assert_eq!(res.events.len(), 6);
        let balance = suite
            .query_trader_balances(
                &nominated_trader_contract.clone(),
                vec![AssetInfo::Native(ujuno.to_string())],
            )
            .unwrap();
        // Verify the first balance is our native token and that we have 1_000_000u128 worth give or take 1%
        assert!(balance.balances[0].info.is_native_token());
        assert_approx_eq!(balance.balances[0].amount, 1_000_000u128.into(), "0.01");
    }

    #[test]
    // This test is the most basic success case
    // Swapping a 'collected' cw20 token to our desired cw20 token
    fn basic_setup_one_pair() {
        let mut suite = SuiteBuilder::new().build();

        let owner = suite.owner.clone();

        let token = suite.instantiate_token(&owner, "wynd");

        let token_to_swap = suite.instantiate_token(&owner, "notwynd");

        let nominated_trader_contract = suite
            .setup_trader(
                Addr::unchecked("owner"),
                AssetInfo::Token(token.to_string()),
            )
            .unwrap();
        // Create 1 pair, cw20 token to swap and the desired token
        suite
            .create_pair_and_provide_liquidity(
                PairType::Xyk {},
                (AssetInfo::Token(token_to_swap.to_string()), 100_000_000u128),
                (AssetInfo::Token(token.to_string()), 100_000_000u128),
                vec![],
            )
            .unwrap();

        // Mint some cw20 for user to exchange
        suite
            .mint_cw20(
                &owner,
                &token_to_swap,
                1_000_000u128,
                nominated_trader_contract.as_ref(),
            )
            .unwrap();
        // Setup 1 route
        let new_route = Some(vec![(
            token_asset_info(token.as_ref()),
            token_asset_info(token_to_swap.as_ref()),
        )]);
        suite
            .update_routes(
                &suite.owner.to_string(),
                nominated_trader_contract.clone(),
                new_route,
                None,
            )
            .unwrap();

        let assets_to_take = vec![AssetWithLimit {
            info: AssetInfo::Token(token_to_swap.to_string()),
            limit: None,
        }];

        let res = suite
            .trade_collected_assets(
                &suite.trader.clone(),
                nominated_trader_contract.clone(),
                assets_to_take,
            )
            .unwrap();
        assert_eq!(res.events.len(), 7);
        let balance = suite
            .query_cw20_balance(nominated_trader_contract.as_ref(), &token)
            .unwrap();
        // Can we verify this is a valid number somehow?
        assert_approx_eq!(balance, 990099u128, "0.000001");

        // Next case verifying we can collect with limit enforced
        // Mint some cw20 for user to exchange
        suite
            .mint_cw20(
                &owner,
                &token_to_swap,
                1_000_000u128,
                nominated_trader_contract.as_ref(),
            )
            .unwrap();

        let assets_to_take = vec![AssetWithLimit {
            info: AssetInfo::Token(token_to_swap.to_string()),
            limit: Some(Uint128::new(100_000u128)),
        }];

        let res = suite
            .trade_collected_assets(
                &suite.trader.clone(),
                nominated_trader_contract.clone(),
                assets_to_take,
            )
            .unwrap();
        assert_eq!(res.events.len(), 7);
        let new_balance = suite
            .query_cw20_balance(nominated_trader_contract.as_ref(), &token)
            .unwrap();
        // Previously queried balance is 100_000u128 larger now within 0.01% approximation showing we have successfully gotten with the limit
        assert_approx_eq!(balance + 100_000u128, new_balance, "0.01");
    }

    #[test]
    // For this test we want to test multiple tokens to be collected
    // and not only can we swap with 1 token we can do for example 2 or 3; native and cw20
    fn multiple_tokens_collection() {
        let ujuno = "ujuno";
        let user = "user";

        let mut suite = SuiteBuilder::new()
            .with_funds(user, &[coin(100u128, ujuno)])
            .build();

        let owner = suite.owner.clone();

        let token = suite.instantiate_token(&owner, "wynd");
        let token_to_swap = suite.instantiate_token(&owner, "notwynd");

        let nominated_trader_contract = suite
            .setup_trader(
                Addr::unchecked("owner"),
                AssetInfo::Token(token.to_string()),
            )
            .unwrap();

        // create LP for desired token and a native token
        suite
            .create_pair_and_provide_liquidity(
                PairType::Xyk {},
                (AssetInfo::Token(token.to_string()), 100_000_000u128),
                (AssetInfo::Native(ujuno.to_owned()), 100_000_000u128),
                vec![coin(100_000_000, ujuno)],
            )
            .unwrap();
        // create LP between desired token and a cw20 token
        suite
            .create_pair_and_provide_liquidity(
                PairType::Xyk {},
                (AssetInfo::Token(token_to_swap.to_string()), 100_000_000u128),
                (AssetInfo::Token(token.to_string()), 100_000_000u128),
                vec![],
            )
            .unwrap();

        // Mint some cw20 to the trader contract for later collection
        suite
            .mint_cw20(
                &owner,
                &token_to_swap,
                1_000_000u128,
                nominated_trader_contract.as_ref(),
            )
            .unwrap();

        // Send some Native tokens to the trader contract from the user; Native
        suite
            .send_native_tokens(
                user,
                nominated_trader_contract.clone(),
                &[coin(100u128, ujuno)],
            )
            .unwrap();
        // Setup routes for trades, each token in this case has a direct pair
        let new_route = Some(vec![
            (
                token_asset_info(token.as_ref()),
                token_asset_info(token_to_swap.as_ref()),
            ),
            (token_asset_info(token.as_ref()), native_asset_info(ujuno)),
        ]);
        suite
            .update_routes(
                &suite.owner.to_string(),
                nominated_trader_contract.clone(),
                new_route,
                None,
            )
            .unwrap();

        // Define some assets to collect, in this case both a cw20 and a native token
        let assets_to_take = vec![
            AssetWithLimit {
                info: AssetInfo::Token(token_to_swap.to_string()),
                limit: None,
            },
            AssetWithLimit {
                info: AssetInfo::Native(ujuno.to_string()),
                limit: None,
            },
        ];
        // Perform the collection
        let res = suite
            .trade_collected_assets(
                &suite.trader.clone(),
                nominated_trader_contract.clone(),
                assets_to_take,
            )
            .unwrap();
        // Ensure we have enough event indicating swap have happened
        assert_eq!(res.events.len(), 11);
        let balance = suite
            .query_cw20_balance(nominated_trader_contract.as_ref(), &token)
            .unwrap();
        // Previously queried balance now within 0.01% approximation of this value showing we have successfully gotten tokens with the limit
        assert_approx_eq!(balance, 990099u128, "0.01");

        // Ensure we have no more of either 'collected' token anymore
        let balances = suite
            .query_trader_balances(
                &nominated_trader_contract.clone(),
                vec![
                    AssetInfo::Native(ujuno.to_owned()),
                    AssetInfo::Token(token_to_swap.to_string()),
                ],
            )
            .unwrap();
        assert_eq!(balances.balances, vec![])
    }

    #[test]
    // This test ensures that the owner and NOT the trader can
    // spend an amount of the desired token to a given individual
    fn owner_can_spend() {
        let mut suite = SuiteBuilder::new().build();

        let owner = suite.owner.clone();

        let token = suite.instantiate_token(&owner, "wynd");

        let nominated_trader_contract = suite
            .setup_trader(
                Addr::unchecked("owner"),
                AssetInfo::Token(token.to_string()),
            )
            .unwrap();

        // Mint some token to user for spending
        suite
            .mint_cw20(
                &owner,
                &token,
                1_000_000u128,
                nominated_trader_contract.as_ref(),
            )
            .unwrap();
        // Owner performs a spend sender a number of 'token' to a recipient
        suite
            .spend(
                &suite.owner.clone(),
                nominated_trader_contract.clone(),
                suite.trader.clone(),
                Uint128::new(100_000u128),
            )
            .unwrap();
        // Recipient has received the balance
        let balance = suite
            .query_cw20_balance(&suite.trader.clone(), &token)
            .unwrap();
        assert_eq!(balance, 100_000u128);
    }

    #[test]
    // This test sets up a number of bridge pools rather than a direct pool for
    // the token to be collected and the desired token
    // This means that when collecting we will need to go
    // through the registered routes for each token
    // each resultant route token, that is to say non direct swaps
    // will have their next hop or swap done as a SubMsg using `swap_hop_assets`
    // this process continues until we get to the desired token
    // Before doing the process or even collection
    // the route is validated prior to ensure the path exists
    fn swap_using_routes() {
        let ujuno = "ujuno";

        let mut suite = SuiteBuilder::new().build();

        let owner = suite.owner.clone();

        let token = suite.instantiate_token(&owner, "wynd");

        let token_to_swap = suite.instantiate_token(&owner, "notwynd");

        let bridge = suite.instantiate_token(&owner, "bridge");

        let nominated_trader_contract = suite
            .setup_trader(
                Addr::unchecked("owner"),
                AssetInfo::Token(token.to_string()),
            )
            .unwrap();

        // create 3 pairs, one for a native to the desired token
        // one with cw20 token to swap and another bridge token
        // third and final pair is the bridge token to our desired token
        suite
            .create_pair_and_provide_liquidity(
                PairType::Xyk {},
                (AssetInfo::Native(ujuno.to_owned()), 100_000_000u128),
                (AssetInfo::Token(token.to_string()), 100_000_000u128),
                vec![coin(100_000_000, ujuno)],
            )
            .unwrap();
        suite
            .create_pair_and_provide_liquidity(
                PairType::Xyk {},
                (AssetInfo::Token(token_to_swap.to_string()), 100_000_000u128),
                (AssetInfo::Token(bridge.to_string()), 100_000_000u128),
                vec![],
            )
            .unwrap();
        suite
            .create_pair_and_provide_liquidity(
                PairType::Xyk {},
                (AssetInfo::Token(bridge.to_string()), 100_000_000u128),
                (AssetInfo::Token(token.to_string()), 100_000_000u128),
                vec![],
            )
            .unwrap();

        // Mint some cw20 for user to exchange
        suite
            .mint_cw20(
                &owner,
                &token_to_swap,
                1_000_000u128,
                nominated_trader_contract.as_ref(),
            )
            .unwrap();
        // Setup only 1 route, the cw20 token to swap and the bridge
        let new_route = Some(vec![(
            token_asset_info(token_to_swap.as_ref()),
            token_asset_info(bridge.as_ref()),
        )]);

        suite
            .update_routes(
                &suite.owner.to_string(),
                nominated_trader_contract.clone(),
                new_route,
                None,
            )
            .unwrap();

        let assets_to_take = vec![AssetWithLimit {
            info: AssetInfo::Token(token_to_swap.to_string()),
            limit: None,
        }];

        // This one should fail, because we are doing it with the owner
        let err = suite
            .trade_collected_assets(
                &suite.owner.clone(),
                nominated_trader_contract.clone(),
                assets_to_take.clone(),
            )
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            "Unauthorized Only the nominated trader can collect fees"
        );

        let res = suite
            .trade_collected_assets(
                &suite.trader.clone(),
                nominated_trader_contract.clone(),
                assets_to_take,
            )
            .unwrap();
        // More messages than usual indicating we did a swap with route assets.
        assert_eq!(res.events.len(), 15);
        // Verify we have the correct balance after the swaps with 2% or less slippage
        let balance = suite
            .query_cw20_balance(nominated_trader_contract.as_ref(), &token)
            .unwrap();
        // Verify we have the correct balance after the swaps with 2% or less slippage
        assert_approx_eq!(balance, 1_000_000u128, "0.02");
    }
}
