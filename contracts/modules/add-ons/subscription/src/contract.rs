#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, Uint128, Uint64, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw_storage_plus::Map;
use pandora_os::registery::SUBSCRIPTION;
use protobuf::Message;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use semver::{Version};

use pandora_os::modules::dapp_base::commands as dapp_base_commands;
use pandora_os::util::fee::Fee;

use pandora_os::modules::dapp_base::common::BaseDAppResult;
use pandora_os::modules::dapp_base::msg::BaseInstantiateMsg;
use pandora_os::modules::dapp_base::queries as dapp_base_queries;
use pandora_os::modules::dapp_base::state::{BaseState, ADMIN, BASESTATE};

use crate::error::SubscriptionError;
use pandora_os::modules::add_ons::subscription::state::{Config, State, CLIENTS, CONFIG, MONTH, STATE};
use crate::{commands, queries};
use pandora_os::modules::add_ons::subscription::msg::{
    MigrateMsg, QueryMsg, StateResponse,ExecuteMsg, InstantiateMsg,
};
pub type SubscriptionResult = Result<Response, SubscriptionError>;

const INSTANTIATE_REPLY_ID: u8 = 1u8;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> SubscriptionResult {
    let version = CONTRACT_VERSION.parse::<Version>()?;
    let storage_version = get_contract_version(deps.storage)?.version.parse::<Version>()?;
    if storage_version < version {
        set_contract_version(deps.storage, SUBSCRIPTION, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> SubscriptionResult {
    set_contract_version(deps.storage, SUBSCRIPTION, CONTRACT_VERSION)?;
    let base_state: BaseState = dapp_base_commands::handle_base_init(deps.as_ref(), msg.base)?;

    let config: Config = Config {
        payment_asset: msg.payment_asset.check(deps.api, None)?,
        subscription_cost: msg.subscription_cost,
        version_control_address: deps.api.addr_validate(&msg.version_control_addr)?,
    };

    let state: State = State {
        income: Uint64::zero(),
        next_pay_day: Uint64::from(env.block.time.seconds() + MONTH),
        debtors: vec![],
    };

    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;
    BASESTATE.save(deps.storage, &base_state)?;
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> SubscriptionResult {
    match msg {
        ExecuteMsg::Base(message) => {
            dapp_base_commands::handle_base_message(deps, info, message).map_err(|e| e.into())
        }
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::Pay { asset, os_id } => commands::try_pay(deps, info, asset, None, os_id),
        ExecuteMsg::PurgeDebtors {page_limit} => commands::purge_debtors(deps, env, page_limit),
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
                next_pay_day: state.next_pay_day,
                debtors: state.debtors,
            })
        }
    }
}
