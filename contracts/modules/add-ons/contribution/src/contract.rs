#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, Uint128, Uint64, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw_storage_plus::Map;
use pandora_os::registery::PAYMENT;
use protobuf::Message;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use semver::Version;
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;

use pandora_os::modules::dapp_base::commands as dapp_base_commands;
use pandora_os::util::fee::Fee;

use pandora_os::modules::dapp_base::common::BaseDAppResult;
use pandora_os::modules::dapp_base::msg::BaseInstantiateMsg;
use pandora_os::modules::dapp_base::queries as dapp_base_queries;
use pandora_os::modules::dapp_base::state::{BaseState, ADMIN, BASESTATE};

use crate::response::MsgInstantiateContractResponse;

use crate::error::PaymentError;
use crate::state::{Config, State, CLIENTS, CONFIG, MONTH, STATE};
use crate::{commands, queries};
use pandora_os::modules::add_ons::payout::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StateResponse,
};
pub type PaymentResult = Result<Response, PaymentError>;

const INSTANTIATE_REPLY_ID: u8 = 1u8;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> PaymentResult {
    let version: Version = CONTRACT_VERSION.parse()?;
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;
    if storage_version < version {
        set_contract_version(deps.storage, PAYMENT, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> PaymentResult {
    set_contract_version(deps.storage, PAYMENT, CONTRACT_VERSION)?;
    let base_state: BaseState = dapp_base_commands::handle_base_init(deps.as_ref(), msg.base)?;

    let config: Config = Config {
        payment_asset: msg.payment_asset,
        ratio: msg.ratio,
        subscription_cost: msg.subscription_cost,
        project_token: deps.api.addr_validate(&msg.project_token)?,
    };

    let state: State = State {
        token_cap: msg.token_cap,
        target: Uint64::zero(),
        income: Uint64::zero(),
        expense: Uint64::zero(),
        total_weight: Uint128::zero(),
        next_pay_day: Uint64::from(env.block.time.seconds() + MONTH),
        debtors: vec![],
        expense_ratio: Decimal::zero(),
    };

    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;
    BASESTATE.save(deps.storage, &base_state)?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> PaymentResult {
    match msg {
        ExecuteMsg::Base(message) => {
            from_base_dapp_result(dapp_base_commands::handle_base_message(deps, info, message))
        }
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::Pay { asset, os_id } => commands::try_pay(deps, info, asset, None, os_id),
        ExecuteMsg::Claim { page_limit } => commands::try_claim(deps, env, info, page_limit),
        ExecuteMsg::UpdateContributor {
            contributor_addr,
            compensation,
        } => commands::update_contributor(deps, info, contributor_addr, compensation),
        ExecuteMsg::RemoveContributor { contributor_addr } => {
            commands::remove_contributor(deps, info, contributor_addr)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(message) => dapp_base_queries::handle_base_query(deps, message),
        // handle dapp-specific queries here
        QueryMsg::State {} => {
            let state = STATE.load(deps.storage)?;
            to_binary(&StateResponse {
                income: state.income,
                total_weight: state.total_weight,
                next_pay_day: state.next_pay_day,
            })
        }
    }
}

/// Required to convert BaseDAppResult into TerraswapResult
/// Can't implement the From trait directly
fn from_base_dapp_result(result: BaseDAppResult) -> PaymentResult {
    match result {
        Err(e) => Err(e.into()),
        Ok(r) => Ok(r),
    }
}
