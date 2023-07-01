use std::collections::HashSet;

use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Api, Binary, Coin, CosmosMsg, Decimal, Deps,
    DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_utils::ensure_from_older_version;

use crate::msg::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    SimulateSwapOperationsResponse, SwapOperation, MAX_SWAP_OPERATIONS,
};
use wyndex::asset::{addr_opt_validate, Asset, AssetInfo, AssetInfoExt};
use wyndex::pair::{ExecuteMsg as PairExecuteMsg, QueryMsg as PairQueryMsg, SimulationResponse};
use wyndex::querier::{query_balance, query_pair_info, query_token_balance};

use crate::error::ContractError;
use crate::state::{Config, CONFIG};

/// Version info for migration
const CONTRACT_NAME: &str = "wynd-multi-hop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            wyndex_factory: deps.api.addr_validate(&msg.wyndex_factory)?,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, msg),
        ExecuteMsg::ExecuteSwapOperations {
            operations,
            minimum_receive,
            receiver,
            max_spread,
            referral_address,
            referral_commission,
        } => execute::swap_operations(
            deps,
            env,
            info.sender,
            operations,
            minimum_receive,
            receiver,
            max_spread,
            referral_address,
            referral_commission,
        ),
        ExecuteMsg::ExecuteSwapOperation {
            operation,
            receiver,
            max_spread,
            single,
            referral_address,
            referral_commission,
        } => execute::swap_operation(
            deps,
            env,
            info,
            operation,
            receiver,
            max_spread,
            single,
            referral_address,
            referral_commission,
        ),
        ExecuteMsg::AssertMinimumReceive {
            asset_info,
            prev_balance,
            minimum_receive,
            receiver,
        } => execute::assert_minimum_receive(
            deps.as_ref(),
            asset_info,
            prev_balance,
            minimum_receive,
            deps.api.addr_validate(&receiver)?,
        ),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let sender = deps.api.addr_validate(&cw20_msg.sender)?;
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::ExecuteSwapOperations {
            operations,
            minimum_receive,
            receiver,
            max_spread,
            referral_address,
            referral_commission,
        } => execute::swap_operations(
            deps,
            env,
            sender,
            operations,
            minimum_receive,
            receiver,
            max_spread,
            referral_address,
            referral_commission,
        ),
    }
}

mod execute {
    use super::*;

    #[allow(clippy::too_many_arguments)]
    pub fn swap_operation(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        operation: SwapOperation,
        receiver: Option<String>,
        max_spread: Option<Decimal>,
        single: bool,
        referral_address: Option<String>,
        referral_commission: Option<Decimal>,
    ) -> Result<Response, ContractError> {
        if env.contract.address != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        let message = match operation {
            SwapOperation::WyndexSwap {
                offer_asset_info,
                ask_asset_info,
            } => {
                let config = CONFIG.load(deps.storage)?;
                let pair_info = query_pair_info(
                    &deps.querier,
                    &config.wyndex_factory,
                    &[offer_asset_info.clone(), ask_asset_info.clone()],
                )?;

                let amount = match &offer_asset_info {
                    AssetInfo::Native(denom) => {
                        query_balance(&deps.querier, env.contract.address, denom)?
                    }
                    AssetInfo::Token(contract_addr) => {
                        query_token_balance(&deps.querier, contract_addr, env.contract.address)?
                    }
                };
                let offer_asset = Asset {
                    info: offer_asset_info,
                    amount,
                };

                asset_into_swap_msg(
                    pair_info.contract_addr.to_string(),
                    offer_asset,
                    ask_asset_info,
                    max_spread,
                    receiver,
                    single,
                    referral_address,
                    referral_commission,
                )?
            }
        };

        Ok(Response::new().add_message(message))
    }

    #[allow(clippy::too_many_arguments)]
    fn asset_into_swap_msg(
        pair_contract: String,
        offer_asset: Asset,
        ask_asset_info: AssetInfo,
        max_spread: Option<Decimal>,
        receiver: Option<String>,
        single: bool,
        referral_address: Option<String>,
        referral_commission: Option<Decimal>,
    ) -> StdResult<CosmosMsg> {
        // Disabling spread assertion if this swap is part of a multi hop route
        let belief_price = if single { None } else { Some(Decimal::MAX) };

        match &offer_asset.info {
            AssetInfo::Native(denom) => {
                // Deduct tax first
                let amount = offer_asset.amount;
                Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: pair_contract,
                    funds: vec![Coin {
                        denom: denom.to_string(),
                        amount,
                    }],
                    msg: to_binary(&PairExecuteMsg::Swap {
                        offer_asset: Asset {
                            amount,
                            ..offer_asset
                        },
                        ask_asset_info: Some(ask_asset_info),
                        belief_price,
                        max_spread,
                        to: receiver,
                        referral_address,
                        referral_commission,
                    })?,
                }))
            }
            AssetInfo::Token(contract_addr) => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: pair_contract,
                    amount: offer_asset.amount,
                    msg: to_binary(&wyndex::pair::Cw20HookMsg::Swap {
                        ask_asset_info: Some(ask_asset_info),
                        belief_price,
                        max_spread,
                        to: receiver,
                        referral_address,
                        referral_commission,
                    })?,
                })?,
            })),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn swap_operations(
        deps: DepsMut,
        env: Env,
        sender: Addr,
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        receiver: Option<String>,
        max_spread: Option<Decimal>,
        referral_address: Option<String>,
        referral_commission: Option<Decimal>,
    ) -> Result<Response, ContractError> {
        if operations.is_empty() {
            return Err(ContractError::MustProvideOperations {});
        }

        let operations_len = operations.len();
        if operations_len > MAX_SWAP_OPERATIONS {
            return Err(ContractError::SwapLimitExceeded {});
        }

        // Assert the operations are properly set
        assert_operations(deps.api, &operations)?;

        let receiver = addr_opt_validate(deps.api, &receiver)?.unwrap_or(sender);

        let target_asset_info = operations
            .last()
            .unwrap()
            .get_target_asset_info()
            .validate(deps.api)?;

        let mut messages = operations
            .into_iter()
            .enumerate()
            .map(|(operation_index, op)| {
                Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    funds: vec![],
                    msg: to_binary(&ExecuteMsg::ExecuteSwapOperation {
                        operation: op,
                        receiver: if operation_index == operations_len - 1 {
                            Some(receiver.to_string())
                        } else {
                            None
                        },
                        max_spread,
                        single: operations_len == 1,
                        referral_address: if operation_index == 0 {
                            referral_address.clone()
                        } else {
                            None
                        },
                        referral_commission: if operation_index == 0 {
                            referral_commission
                        } else {
                            None
                        },
                    })?,
                }))
            })
            .collect::<StdResult<Vec<CosmosMsg>>>()?;

        // Execute minimum amount assertion
        if let Some(minimum_receive) = minimum_receive {
            let receiver_balance = target_asset_info.query_balance(&deps.querier, &receiver)?;
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::AssertMinimumReceive {
                    asset_info: target_asset_info.into(),
                    prev_balance: receiver_balance,
                    minimum_receive,
                    receiver: receiver.to_string(),
                })?,
            }));
        }

        Ok(Response::new().add_messages(messages))
    }

    pub fn assert_minimum_receive(
        deps: Deps,
        asset_info: AssetInfo,
        prev_balance: Uint128,
        minimum_receive: Uint128,
        receiver: Addr,
    ) -> Result<Response, ContractError> {
        let asset_info = asset_info.validate(deps.api)?;
        let receiver_balance = asset_info.query_balance(&deps.querier, receiver)?;
        let swap_amount = receiver_balance.checked_sub(prev_balance)?;

        if swap_amount < minimum_receive {
            Err(ContractError::AssertionMinimumReceive {
                receive: minimum_receive,
                amount: swap_amount,
            })
        } else {
            Ok(Response::default())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query::config(deps)?)?),
        QueryMsg::SimulateSwapOperations {
            offer_amount,
            operations,
            referral,
            referral_commission,
        } => Ok(to_binary(&query::simulate_swap_operations(
            deps,
            offer_amount,
            referral,
            referral_commission,
            operations,
        )?)?),
        QueryMsg::SimulateReverseSwapOperations {
            ask_amount,
            operations,
            referral,
            referral_commission,
        } => Ok(to_binary(&query::simulate_reverse_swap_operations(
            deps,
            ask_amount,
            referral,
            referral_commission,
            operations,
        )?)?),
    }
}

mod query {
    use wyndex::pair::ReverseSimulationResponse;

    use super::*;

    /// Returns general contract settings in a [`ConfigResponse`] object.
    pub fn config(deps: Deps) -> Result<ConfigResponse, ContractError> {
        let state = CONFIG.load(deps.storage)?;
        let resp = ConfigResponse {
            wyndex_factory: state.wyndex_factory.into_string(),
        };

        Ok(resp)
    }

    /// Returns the end result of a simulation for one or multiple swap
    /// operations using a [`SimulateSwapOperationsResponse`] object.
    ///
    /// * **offer_amount** amount of offer assets being swapped.
    ///
    /// * **operations** is a vector that contains objects of type [`SwapOperation`].
    /// These are all the swap operations for which we perform a simulation.
    pub fn simulate_swap_operations(
        deps: Deps,
        offer_amount: Uint128,
        referral: bool,
        referral_commission: Option<Decimal>,
        operations: Vec<SwapOperation>,
    ) -> Result<SimulateSwapOperationsResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        let wyndex_factory = config.wyndex_factory;

        let operations_len = operations.len();
        if operations_len == 0 {
            return Err(ContractError::MustProvideOperations {});
        }

        if operations_len > MAX_SWAP_OPERATIONS {
            return Err(ContractError::SwapLimitExceeded {});
        }

        assert_operations(deps.api, &operations)?;

        let mut offer_amount = offer_amount;
        let mut spread_amounts = Vec::with_capacity(operations_len);
        let mut commission_amounts = Vec::with_capacity(operations_len);
        let mut referral_amount = None;
        // the ratio of swap result to ideal swap result (= 1 - spread percentage)
        let mut percent_of_ideal = Decimal::one();
        for (idx, operation) in operations.into_iter().enumerate() {
            match operation {
                SwapOperation::WyndexSwap {
                    offer_asset_info,
                    ask_asset_info,
                } => {
                    let pair_info = query_pair_info(
                        &deps.querier,
                        wyndex_factory.clone(),
                        &[offer_asset_info.clone(), ask_asset_info.clone()],
                    )?;

                    let res: SimulationResponse = deps.querier.query_wasm_smart(
                        pair_info.contract_addr,
                        &PairQueryMsg::Simulation {
                            offer_asset: Asset {
                                info: offer_asset_info.clone(),
                                amount: offer_amount,
                            },
                            ask_asset_info: Some(ask_asset_info.clone()),
                            referral: if idx == 0 { referral } else { false },
                            referral_commission: if idx == 0 { referral_commission } else { None },
                        },
                    )?;
                    offer_amount = res.return_amount;
                    // to calculate the percentage of ideal amount for one operation,
                    // we use the formula `(return_amount + commission) / (return_amount + commission + spread_amount)`
                    // (essentially: what we got from swapping, divided by what we would have gotten if there was no price impact).
                    // The commission needs to be part of that calculation, because it is also part of the swap.
                    // Otherwise it would be counted as spread.
                    // This then needs to be multiplied by the percentage of the previous swap operation to
                    // get the percentage with regards to the whole swap.
                    percent_of_ideal *= Decimal::from_ratio(
                        res.return_amount + res.commission_amount,
                        res.return_amount + res.commission_amount + res.spread_amount,
                    );

                    let ask_asset_info = ask_asset_info.validate(deps.api)?;
                    spread_amounts.push(ask_asset_info.with_balance(res.spread_amount));
                    commission_amounts.push(ask_asset_info.with_balance(res.commission_amount));
                    if idx == 0 {
                        // only first operation can have a referral commission
                        let offer_asset_info = offer_asset_info.validate(deps.api)?;
                        referral_amount = Some(offer_asset_info.with_balance(res.referral_amount));
                    }
                }
            }
        }

        Ok(SimulateSwapOperationsResponse {
            amount: offer_amount,
            spread: Decimal::one() - percent_of_ideal,
            spread_amounts,
            commission_amounts,
            referral_amount: referral_amount
                .expect("referral_amount must be set for first operation"),
        })
    }

    /// Returns the offer asset needed and the result of a simulation for one or multiple swap
    /// operations using a [`SimulateSwapOperationsResponse`] object.
    ///
    /// * **ask_amount** amount of offer assets being swapped.
    ///
    /// * **operations** is a vector that contains objects of type [`SwapOperation`].
    /// These are all the swap operations for which we perform a simulation.
    pub fn simulate_reverse_swap_operations(
        deps: Deps,
        ask_amount: Uint128,
        referral: bool,
        referral_commission: Option<Decimal>,
        operations: Vec<SwapOperation>,
    ) -> Result<SimulateSwapOperationsResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        let wyndex_factory = config.wyndex_factory;

        let operations_len = operations.len();
        if operations_len == 0 {
            return Err(ContractError::MustProvideOperations {});
        }

        if operations_len > MAX_SWAP_OPERATIONS {
            return Err(ContractError::SwapLimitExceeded {});
        }

        assert_operations(deps.api, &operations)?;

        let mut ask_amount = ask_amount;
        let mut spread_amounts = Vec::with_capacity(operations_len);
        let mut commission_amounts = Vec::with_capacity(operations_len);
        let mut referral_amount = None;
        // the ratio of swap result to ideal swap result (= 1 - spread percentage)
        let mut percent_of_ideal = Decimal::one();
        for (idx, operation) in operations.into_iter().enumerate().rev() {
            match operation {
                SwapOperation::WyndexSwap {
                    offer_asset_info,
                    ask_asset_info,
                } => {
                    let pair_info = query_pair_info(
                        &deps.querier,
                        wyndex_factory.clone(),
                        &[offer_asset_info.clone(), ask_asset_info.clone()],
                    )?;

                    let res: ReverseSimulationResponse = deps.querier.query_wasm_smart(
                        pair_info.contract_addr,
                        &PairQueryMsg::ReverseSimulation {
                            offer_asset_info: Some(offer_asset_info.clone()),
                            ask_asset: Asset {
                                info: ask_asset_info.clone(),
                                amount: ask_amount,
                            },
                            referral: if idx == 0 { referral } else { false },
                            referral_commission: if idx == 0 { referral_commission } else { None },
                        },
                    )?;
                    // to calculate the percentage of ideal amount for one operation,
                    // we use the formula `(ask_amount + commission) / (ask_amount + commission + spread_amount)`
                    // (essentially: what we got from swapping, divided by what we would have gotten if there was no price impact).
                    // The commission needs to be part of that calculation, because it is also part of the swap.
                    // Otherwise it would be counted as spread.
                    // This then needs to be multiplied by the percentage of the previous swap operation to
                    // get the percentage with regards to the whole swap.
                    percent_of_ideal *= Decimal::from_ratio(
                        ask_amount + res.commission_amount,
                        ask_amount + res.commission_amount + res.spread_amount,
                    );
                    // previous swap has to return what we need to input into this swap
                    ask_amount = res.offer_amount;

                    let ask_asset_info = ask_asset_info.validate(deps.api)?;
                    spread_amounts.push(ask_asset_info.with_balance(res.spread_amount));
                    commission_amounts.push(ask_asset_info.with_balance(res.commission_amount));
                    if idx == 0 {
                        // only first operation can have a referral commission
                        let offer_asset_info = offer_asset_info.validate(deps.api)?;
                        referral_amount = Some(offer_asset_info.with_balance(res.referral_amount));
                    }
                }
            }
        }

        Ok(SimulateSwapOperationsResponse {
            amount: ask_amount,
            spread: Decimal::one() - percent_of_ideal,
            spread_amounts,
            commission_amounts,
            referral_amount: referral_amount
                .expect("referral_amount must be set for first operation"),
        })
    }
}

/// Validates swap operations.
fn assert_operations(api: &dyn Api, operations: &[SwapOperation]) -> Result<(), ContractError> {
    let mut ask_asset_map: HashSet<String> = HashSet::new();
    for operation in operations {
        let (offer_asset, ask_asset) = match operation {
            SwapOperation::WyndexSwap {
                offer_asset_info,
                ask_asset_info,
            } => (
                offer_asset_info.validate(api)?,
                ask_asset_info.validate(api)?,
            ),
        };

        ask_asset_map.remove(&offer_asset.to_string());
        ask_asset_map.insert(ask_asset.to_string());
    }

    if ask_asset_map.len() != 1 {
        return Err(StdError::generic_err("invalid operations; multiple output token").into());
    }

    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn test_invalid_operations() {
        use cosmwasm_std::testing::mock_dependencies;
        let deps = mock_dependencies();
        // Empty error
        assert!(assert_operations(deps.as_ref().api, &[]).is_err());

        // uluna output
        assert!(assert_operations(
            deps.as_ref().api,
            &[
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Native("ukrw".to_string()),
                    ask_asset_info: AssetInfo::Token("asset0001".to_string()),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Token("asset0001".to_string()),
                    ask_asset_info: AssetInfo::Native("uluna".to_string()),
                },
            ],
        )
        .is_ok());

        // asset0002 output
        assert!(assert_operations(
            deps.as_ref().api,
            &[
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Native("ukrw".to_string()),
                    ask_asset_info: AssetInfo::Token("asset0001".to_string()),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Token("asset0001".to_string()),
                    ask_asset_info: AssetInfo::Native("uluna".to_string()),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Native("uluna".to_string()),
                    ask_asset_info: AssetInfo::Token("asset0002".to_string()),
                },
            ],
        )
        .is_ok());

        // Multiple output token type errors
        assert!(assert_operations(
            deps.as_ref().api,
            &[
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Native("ukrw".to_string()),
                    ask_asset_info: AssetInfo::Token("asset0001".to_string()),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Token("asset0001".to_string()),
                    ask_asset_info: AssetInfo::Native("uaud".to_string()),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Native("uluna".to_string()),
                    ask_asset_info: AssetInfo::Token("asset0002".to_string()),
                },
            ],
        )
        .is_err());
    }
}
