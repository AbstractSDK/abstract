use std::str::FromStr;
use abstract_core::objects::{AccountId, AnsAsset, AssetEntry};
use abstract_sdk::{
    *,
    core::objects::fee::Fee, features::AbstractResponse,
};
use cosmwasm_std::{Addr, coins, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::Item;

use crate::contract::{BetApp, BetResult};
use crate::error::BetError;
use crate::msg::BetExecuteMsg;
use crate::state::*;
use crate::state::CONFIG;
use abstract_sdk::features::AbstractNameService;
use crate::handlers::query;


pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: BetApp,
    msg: BetExecuteMsg,
) -> BetResult {
    match msg {
        BetExecuteMsg::CreateRound {
            name,
            description,
            base_bet_token,
        } => {
            app.admin.assert_admin(deps.as_ref(), &info.sender)?;
            let round_info = RoundInfo {
                name: name.clone(),
                description: description.clone(),
                base_bet_token: base_bet_token.clone(),
                status: RoundStatus::Open,
            };
            // Check asset
            create_round(deps, info, app, round_info)
        }
        BetExecuteMsg::UpdateAccounts { to_add, to_remove, round_id } => {
            // Only admin can register specific accounts
            app.admin.assert_admin(deps.as_ref(), &info.sender)?;

            let round = Round::new(round_id);
            round.assert_not_closed(deps.storage)?;
            deps.api.debug(&format!("to_add: {:?}", to_add));

            update_accounts(deps, info, app, round, to_add, to_remove)
        }
        BetExecuteMsg::UpdateConfig { rake } => {
            app.admin.assert_admin(deps.as_ref(), &info.sender)?;
            // TODO: use config constant, not sure why this is not working.
            let mut config: Config = Item::new("config").load(deps.storage)?;
            let mut attrs = vec![];

            if let Some(rake) = rake {
                config.rake = Fee::new(rake)?;
                attrs.push(("rake", rake.to_string()));
            };

            Ok(app.custom_tag_response(
                Response::default(),
                "update_config",
                attrs,
            ))
        }
        BetExecuteMsg::PlaceBet {
            bet
        } => {
            place_bet(deps, info, app, bet)
        }
        BetExecuteMsg::SetWinner {
            round_id, team_id
        } => {
            app.admin.assert_admin(deps.as_ref(), &info.sender)?;
            let round = Round::new(round_id);
            let current_status = round.status(deps.storage)?;

            match current_status {
                RoundStatus::Open => {
                    round.set_status(deps.storage, RoundStatus::Won { winning_team: team_id })?;

                    Ok(app.custom_tag_response(
                        Response::default(),
                        "update_round_status",
                        vec![("round_id", round_id.to_string()) /*, ("status", new_status.to_string()) */],
                    ))
                }
                _ => Err(BetError::RoundAlreadyClosed(round_id)),
            }
        }
        BetExecuteMsg::DistributeWinnings {
            round_id
        } => distribute_winnings(deps, app, round_id)?,
        _ => panic!("Unsupported execute message"),
    }
}

fn distribute_winnings(deps: DepsMut, app: BetApp, round_id: RoundId) -> Result<Result<Response, BetError>, BetError> {
    let round = Round::new(round_id);
    let current_status = round.status(deps.storage)?;

    let winning_team = match current_status {
        RoundStatus::Won { winning_team } => {
            Ok(winning_team)
        },
        _ => Err(BetError::RoundNotClosed(round_id)),
    }?;

    let winning_odds = ODDS.load(deps.storage, (round_id, winning_team.clone()))?;
    println!("winning_odds: {}", winning_odds);
    let overall_winnings = query::get_total_bets_for_team(deps.storage, round_id, winning_team.clone())?;

    let winning_bets = BETS.load(deps.storage, (round_id, winning_team.clone()))?;
    let mut distribution_msgs = vec![];

    let bank = app.bank(deps.as_ref());
    let round_info = round.info(deps.storage)?.base_bet_token;

    for (better_addr, bet_amount) in winning_bets.iter() {
        let bet_amount = *bet_amount;
        let winnings = bet_amount * winning_odds;

        println!("payout_amount: {}", winnings);
        println!("better_addr: {}, bet_amount: {} winning_total: {}", better_addr, bet_amount, overall_winnings);

        let transfer_asset = AnsAsset::new(round_info.clone(), winnings);

        // Create a transfer message to send the payout to the bettor
        let payout_msg = bank.transfer(vec![transfer_asset], better_addr)?;
        distribution_msgs.push(payout_msg);
    }

    let distribution_msg: CosmosMsg = app.executor(deps.as_ref()).execute(distribution_msgs)?.into();

    // Update round's status to RewardsDistributed or something similar
    round.set_status(deps.storage, RoundStatus::RewardsDistributed)?;

    Ok(Ok(app.tag_response(Response::default().add_message(distribution_msg), "distribute_winnings")))
}

fn update_accounts(deps: DepsMut, info: MessageInfo, app: BetApp, round: Round, to_add: Vec<AccountOdds>, to_remove: Vec<AccountId>) -> BetResult {
    let account_registry = app.account_registry(deps.as_ref());
    for AccountOdds {
        account_id,
        ..
    } in to_add.iter() {
        // ensure account exists
        account_registry.account_base(&account_id).map_err(|_| BetError::AccountNotFound(account_id.clone()))?;
    }

    // register account
    round.update_accounts(deps, to_add, to_remove)?;

    Ok(app.tag_response(
        Response::default(),
        "update_accounts",
    ))
}

pub fn create_round(
    deps: DepsMut,
    msg_info: MessageInfo,
    app: BetApp,
    round: RoundInfo,
) -> BetResult {
    let ans_host = app.ans_host(deps.as_ref())?;
    let mut state = STATE.load(deps.storage)?;

    // Check round
    round.validate(deps.as_ref(), &ans_host)?;

    ROUNDS.save(deps.storage, state.next_round_id, &round)?;
    ROUNDS_TO_ACCOUNTS.save(deps.storage, state.next_round_id, &vec![])?;

    // Update and save the state
    STATE.update(deps.storage, |mut state| -> BetResult<_> {
        state.next_round_id += 1;
        Ok(state)
    })?;
    Ok(app.custom_tag_response(Response::default(), "create_round", vec![("round_id", state.next_round_id.to_string())]))
}

fn place_bet(deps: DepsMut, info: MessageInfo, app: BetApp, bet: NewBet) -> BetResult {
    let bet_asset = CONFIG.load(deps.storage)?.bet_asset;

    let mut messages: Vec<CosmosMsg> = vec![];

    let bank = app.bank(deps.as_ref());

    // Validate round exists
    let round = ROUNDS.may_load(deps.storage, bet.round_id)?;
    if round.is_none() {
        return Err(BetError::RoundNotFound(bet.round_id));
    }
    let round = Round::new(bet.round_id);

    // Ensure the account placing the bet exists
     bet.validate(deps.as_ref(), &bet_asset)?;

     // deposit the sent assets
     let deposit_msg = bank.deposit(vec![bet.asset.clone()])?;
     messages.extend(deposit_msg.into_iter());

    // Record the bet
    let bet_account = bet.account_id;

    let key = (round.id(), bet_account.clone());
    let mut bets = BETS.may_load(deps.storage, key.clone())?.unwrap_or_default();
    // Find and update the existing bet if it exists
    if let Some(index) = bets.iter().position(|(addr, _)| addr == &info.sender) {
        let (_, amount) = &mut bets[index];
        *amount += bet.asset.amount;
    } else {
        // Otherwise, add a new bet
        bets.push((info.sender.clone(), bet.asset.amount));
    }
    // save the bets
    BETS.save(deps.storage, key.clone(), &bets)?;

    // Retrieve the total bets for the round
    let bet_totals = query::get_total_bets_for_all_accounts(deps.storage, round.id())?;
    let rake = CONFIG.load(deps.storage)?.rake.share();

    let round_teams = round.accounts(deps.storage)?;
    for team in round_teams {
        println!("adjusting odds for team: {}", team);
        // adjust the odds for the round
        adjust_odds_for_team(deps.storage, round.id(), team, bet_totals, rake)?;
    }

    Ok(app.tag_response(Response::default().add_messages(messages), "place_bet"))
}

/// Calculates the new odds for the given round/account pair
/// # Returns
/// the new odds
fn adjust_odds_for_team(storage: &mut dyn Storage, round_id: RoundId, team_id: AccountId, bet_totals: Uint128, rake: Decimal) -> StdResult<()> {
    let team_bet_total = query::get_total_bets_for_team(storage, round_id, team_id.clone())?;
    // No action, odds have not changed
    if team_bet_total.is_zero() {
        return Ok(());
    }

    // Calculate the bet-based odds
    let bet_based_odds = Decimal::from_ratio(bet_totals, team_bet_total);

    // Check if it's the first bet for the round
    let is_first_bet = bet_totals == team_bet_total;
    // If it's the first bet, blend the initial odds with the bet-based odds since it was the initial prediction
    let new_odds = if is_first_bet {
        // Retrieve the initial odds
        let initial_odds = ODDS.load(storage, (round_id, team_id.clone()))?;
        // Blend the initial and bet-based odds
        (initial_odds + bet_based_odds) / Decimal::from_str("2.0").unwrap()
    } else {
        bet_based_odds
    };

    // Apply house edge
    let mut adjusted_odds = new_odds * (Decimal::one() - rake);

    // Don't allow odds to go below 1
    if adjusted_odds < Decimal::one() {
        adjusted_odds = Decimal::one();
    }

    ODDS.save(storage, (round_id, team_id.clone()), &adjusted_odds)
}


// #[cfg(test)]
// mod test {
//     use super::*;
//     use speculoos::prelude::*;
//     mod adjust_odds {
//         use cosmwasm_std::testing::mock_dependencies;
//         use super::*;
//
//         #[test]
//         fn test_adjust_odds() {
//             let mut deps = mock_dependencies();
//             let better = Addr::unchecked("better");
//             let account_id = AccountId::local(1u32.into());
//             let expected_odds = Decimal::from_str("0.1").unwrap();
//
//             // Set up the storage
//             // Initial odds of 2, with two accounts
//             let inital_odds = Uint128::from(2u128);
//
//             ODDS.save(deps.as_mut().storage, (1, account_id.clone()), &Decimal::new(inital_odds)).unwrap();
//             ODDS.save(deps.as_mut().storage, (2, account_id.clone()), &Decimal::new(inital_odds)).unwrap();
//
//             BETS.save(deps.as_mut().storage, (1, account_id.clone()), &vec![
//                 (better.clone(), Uint128::from(100u128)),
//                 (better.clone(), Uint128::from(100u128)),
//             ]).unwrap();
//
//             // Call the function
//             let new_odds = adjust_odds_for_account(&mut storage, round_id, account_id.clone()).unwrap();
//
//             // Check the result
//             assert_eq!(new_odds, expected_odds);
//         }
//     }
// }




/*
pub fn validate_bets(bets: &[NewBet], deps: Deps, ans_host: &AnsHost) -> EtfResult<()> {
    // Cache for accounts registered to a round
    let mut cache: HashMap<RoundId, HashSet<AccountId>> = HashMap::new();

    for bet in bets {
        if bet.asset.amount.is_zero() {
            return Err(BetError::InvalidBet {});
        }

        // ensure that the asset exists
        bet.asset.resolve(&deps.querier, ans_host)?;

        // Load the accounts for the round if not in cache
        if !cache.contains_key(&bet.round_id) {
            let round = Round::new(bet.round_id);
            let accounts = round.accounts(deps.storage)?;
            let mut set = HashSet::new();
            set.extend(accounts);
            cache.insert(bet.round_id, set);
        }

        // Validate bet with cache
        if let Some(accounts) = cache.get(&bet.round_id) {
            if !accounts.contains(&bet.account_id) {
                return Err(BetError::AccountNotParticipating {
                    account_id: bet.account_id.clone(),
                    round_id: bet.round_id,
                });
            }
        }
    }

    Ok(())
}

 */


// /// Called when either providing liquidity with a native token or when providing liquidity
// /// with a CW20.
// pub fn try_provide_liquidity(
//     deps: DepsMut,
//     msg_info: MessageInfo,
//     app: BetApp,
//     asset: Asset,
//     // optional sender address
//     // set if called from CW20 hook
//     sender: Option<String>,
// ) -> EtfResult {
//     let state = STATE.load(deps.storage)?;
//     // Get the depositor address
//     let depositor = match sender {
//         Some(addr) => deps.api.addr_validate(&addr)?,
//         None => {
//             // Check if deposit matches claimed deposit.
//             match asset.info {
//                 AssetInfo::Native(..) => {
//                     // If native token, assert claimed amount is correct
//                     let coin = msg_info.funds.last();
//                     if coin.is_none() {
//                         return Err(BetError::WrongNative {});
//                     }
//
//                     let coin = coin.unwrap().clone();
//                     if Asset::native(coin.denom, coin.amount) != asset {
//                         return Err(BetError::WrongNative {});
//                     }
//                     msg_info.sender
//                 }
//                 AssetInfo::Cw20(_) => return Err(BetError::NotUsingCW20Hook {}),
//                 _ => return Err(BetError::UnsupportedAssetType(asset.info.to_string())),
//             }
//         }
//     };
//     // Get vault API for the account
//     let vault = app.accountant(deps.as_ref());
//     // Construct deposit info
//     let deposit_info = DepositInfo {
//         asset_info: vault.base_asset()?.base_asset,
//     };
//
//     // Assert deposited info and claimed asset info are the same
//     deposit_info.assert(&asset.info)?;
//
//     // Init vector for logging
//     let attrs = vec![
//         ("action", String::from("deposit_to_vault")),
//         ("Received funds:", asset.to_string()),
//     ];
//
//     // Received deposit to vault
//     let deposit: Uint128 = asset.amount;
//
//     // Get total value in Vault
//     let account_value = vault.query_total_value()?;
//     let total_value = account_value.total_value.amount;
//     // Get total supply of LP tokens and calculate share
//     let total_share = query_supply(&deps.querier, state.share_token_address.clone())?;
//
//     let share = if total_share == Uint128::zero() || total_value.is_zero() {
//         // Initial share = deposit amount
//         deposit
//     } else {
//         // lt: liquidity token
//         // lt_to_receive = deposit * lt_price
//         // lt_to_receive = deposit * lt_supply / previous_total_vault_value )
//         // lt_to_receive = deposit * ( lt_supply / ( current_total_vault_value - deposit ) )
//         let value_increase = Decimal::from_ratio(total_value + deposit, total_value);
//         (total_share * value_increase) - total_share
//     };
//
//     // mint LP token to depositor
//     let mint_lp = CosmosMsg::Wasm(WasmMsg::Execute {
//         contract_addr: state.share_token_address.to_string(),
//         msg: to_binary(&Cw20ExecuteMsg::Mint {
//             recipient: depositor.to_string(),
//             amount: share,
//         })?,
//         funds: vec![],
//     });
//
//     // Send received asset to the vault.
//     let send_to_vault = app.bank(deps.as_ref()).deposit(vec![asset])?;
//
//     let response = app
//         .custom_tag_response(Response::default(), "provide_liquidity", attrs)
//         .add_message(mint_lp)
//         .add_messages(send_to_vault);
//
//     Ok(response)
// }
//
// /// Attempt to withdraw deposits. Fees are calculated and deducted in liquidity tokens.
// /// This allows the owner to accumulate a stake in the vault.
// pub fn try_withdraw_liquidity(
//     deps: DepsMut,
//     _env: Env,
//     app: BetApp,
//     sender: Addr,
//     amount: Uint128,
// ) -> EtfResult {
//     let state: State = STATE.load(deps.storage)?;
//     let base_state: AppState = app.load_state(deps.storage)?;
//     let fee: Fee = RAKE.load(deps.storage)?;
//     let bank = app.bank(deps.as_ref());
//     // Get assets
//     let assets: AssetsInfoResponse = app.accountant(deps.as_ref()).assets_list()?;
//
//     // Logging var
//     let mut attrs = vec![("liquidity_tokens", amount.to_string())];
//
//     // Calculate share of pool and requested pool value
//     let total_share: Uint128 = query_supply(&deps.querier, state.share_token_address.clone())?;
//
//     // Get manager fee in LP tokens
//     let manager_fee = fee.compute(amount);
//
//     // Share with fee deducted.
//     let share_ratio: Decimal = Decimal::from_ratio(amount - manager_fee, total_share);
//
//     let mut msgs: Vec<CosmosMsg> = vec![];
//     if !manager_fee.is_zero() {
//         // LP token fee
//         let lp_token_manager_fee = Asset {
//             info: AssetInfo::Cw20(state.share_token_address.clone()),
//             amount: manager_fee,
//         };
//         // Construct manager fee msg
//         let manager_fee_msg = fee.msg(lp_token_manager_fee, state.manager_addr.clone())?;
//
//         // Transfer fee
//         msgs.push(manager_fee_msg);
//     }
//     attrs.push(("treasury_fee", manager_fee.to_string()));
//
//     // Get asset holdings of vault and calculate amount to return
//     let mut shares_assets: Vec<Asset> = vec![];
//     for (info, _) in assets.assets.into_iter() {
//         // query asset held in proxy
//         let asset_balance = info.query_balance(&deps.querier, base_state.proxy_address.clone())?;
//         shares_assets.push(Asset {
//             info: info.clone(),
//             amount: share_ratio * asset_balance,
//         });
//     }
//
//     // Construct repay msg by transferring the assets back to the sender
//     let refund_msg = app
//         .executor(deps.as_ref())
//         .execute(vec![bank.transfer(shares_assets, &sender)?])?;
//
//     // LP burn msg
//     let burn_msg: CosmosMsg = wasm_execute(
//         state.share_token_address,
//         // Burn excludes fee
//         &Cw20ExecuteMsg::Burn {
//             amount: (amount - manager_fee),
//         },
//         vec![],
//     )?
//     .into();
//
//     Ok(app
//         .custom_tag_response(Response::default(), "withdraw_liquidity", attrs)
//         // Burn LP tokens
//         .add_message(burn_msg)
//         // Send proxy funds to owner
//         .add_message(refund_msg))
// }


// /// helper for CW20 supply query
// fn query_supply(querier: &QuerierWrapper, contract_addr: Addr) -> StdResult<Uint128> {
//     let res: TokenInfoResponse = querier.query(&wasm_smart_query(
//         String::from(contract_addr),
//         &Cw20QueryMsg::TokenInfo {},
//     )?)?;
//     Ok(res.total_supply)
// }
