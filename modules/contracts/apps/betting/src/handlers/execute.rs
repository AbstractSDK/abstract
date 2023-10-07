use std::str::FromStr;
use abstract_core::objects::{AccountId, AnsAsset, AssetEntry};
use abstract_sdk::{
    *,
    core::objects::fee::Fee, features::AbstractResponse,
};
use cosmwasm_std::{Addr, coins, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult, Storage, Uint128};
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
                bet_asset: base_bet_token.clone(),
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

            update_config(deps, app, rake)
        }
        BetExecuteMsg::PlaceBet {
            bet,
            round_id
        } => {
            place_bet(deps, info, app, round_id, bet)
        }
        BetExecuteMsg::CloseRound {
            round_id, winner
        } => {
            app.admin.assert_admin(deps.as_ref(), &info.sender)?;
            let round = Round::new(round_id);

            close_round(deps, &app, winner, round)?
        }
        BetExecuteMsg::DistributeWinnings {
            round_id
        } => distribute_winnings(deps, app, round_id),
        BetExecuteMsg::Register { round_id } => {
            let round = Round::new(round_id);
            round.assert_not_closed(deps.storage)?;
            register_for_round(deps, info, app, round)
        }
    }
}

fn update_config(deps: DepsMut, app: BetApp, rake: Option<Decimal>) -> BetResult {
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

fn register_for_round(deps: DepsMut, info: MessageInfo, app: BetApp, round: Round) -> BetResult {
    let account_id = app.account_registry(deps.as_ref()).account_id(&info.sender)?;
    // default odds
    let odds = Decimal::one();
    let to_add = vec![AccountOdds {
        account_id,
        odds,
    }];
    let to_remove = vec![];
    update_accounts(deps, info, app, round, to_add, to_remove)
}

fn close_round(deps: DepsMut, app: &BetApp, winner: Option<AccountId>, round: Round) -> Result<Result<Response, BetError>, BetError> {
    let current_status = round.status(deps.storage)?;

    Ok(match current_status {
        RoundStatus::Open => {
            let accounts = round.accounts(deps.storage)?;
            if let Some(winner) = winner.clone() {
                if !accounts.contains(&winner) {
                    return Err(BetError::AccountNotFound(winner));
                }
            }
            round.set_status(deps.storage, RoundStatus::Closed { winning_team: winner })?;

            Ok(app.custom_tag_response(
                Response::default(),
                "update_round_status",
                vec![("round_id", round.id().to_string()) /*, ("status", new_status.to_string()) */],
            ))
        }
        _ => Err(BetError::RoundAlreadyClosed(round.id())),
    })
}

fn distribute_winnings(deps: DepsMut, app: BetApp, round_id: RoundId) -> BetResult {
    let round = Round::new(round_id);
    let current_status = round.status(deps.storage)?;

    // Set distribution status
    round.set_status(deps.storage, RoundStatus::RewardsDistributed)?;

    let distribution_msgs = match current_status {
        RoundStatus::Closed { winning_team: None } => {
            // No winning team, refund all bets

            let bets = round.bets(deps.storage)?;

            let bank = app.bank(deps.as_ref());
            let round_info = round.info(deps.storage)?.bet_asset;

            let distribution_actions: Result<Vec<AccountAction>, AbstractSdkError> = bets.into_iter()
                .map(|(better_addr, bet_amount)| {
                    let transfer_asset = AnsAsset::new(round_info.clone(), bet_amount);
                    bank.transfer(vec![transfer_asset], &better_addr)
                })
                .collect();

            distribution_actions.map_err(Into::into)
        }
        RoundStatus::Closed { winning_team: Some(winning_team) } => {
            // Distribute the winnings to the winning betters
            let winning_odds = ODDS.load(deps.storage, (round_id, winning_team.clone()))?;
            let winning_bets = BETS.load(deps.storage, (round_id, winning_team.clone()))?;

            let bank = app.bank(deps.as_ref());
            let round_info = round.info(deps.storage)?.bet_asset;

            let distribution_msgs1 = winning_bets.into_iter()
                .map(|(better_addr, bet_amount)| {
                    let winnings = bet_amount * winning_odds;
                    let transfer_asset = AnsAsset::new(round_info.clone(), winnings);
                    bank.transfer(vec![transfer_asset], &better_addr)
                })
                .collect::<Result<Vec<AccountAction>, AbstractSdkError>>();

            distribution_msgs1.map_err(Into::into)
        }
        _ => Err(BetError::RoundNotClosed(round_id))
    }?;

    let executor = app.executor(deps.as_ref());


    let distribution_msg = executor.execute(distribution_msgs)?;

    Ok(app.tag_response(Response::default().add_message(distribution_msg), "distribute_draw"))

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

fn place_bet(deps: DepsMut, info: MessageInfo, app: BetApp, round_id: RoundId, bet: Bet) -> BetResult {
    let mut messages: Vec<CosmosMsg> = vec![];

    let bank = app.bank(deps.as_ref());

    // TODO: move into Round
    // Validate round exists
    let round = ROUNDS.may_load(deps.storage, round_id)?;
    if round.is_none() {
        return Err(BetError::RoundNotFound(round_id));
    }

    let round = Round::new(round_id);

    // Ensure the account placing the bet exists
     bet.validate(deps.as_ref(), &round)?;

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
