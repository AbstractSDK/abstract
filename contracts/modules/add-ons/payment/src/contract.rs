#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Decimal, Deps, DepsMut, Empty, Env, MessageInfo, Reply,
    ReplyOn, Response, StdError, StdResult, SubMsg, Uint128, Uint64, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use cw_storage_plus::Map;
use protobuf::Message;
use semver::Version;
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;

use pandora_dapp_base::{DappContract, DappResult};
use pandora_os::modules::add_ons::payout::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StateResponse,
};
use pandora_os::registery::PAYMENT;
use pandora_os::util::fee::Fee;

use crate::error::PaymentError;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{Config, State, CLIENTS, CONFIG, MONTH, STATE};
use crate::{commands, queries};

type PaymentExtension = Option<Empty>;
pub type PaymentDapp<'a> = DappContract<'a, PaymentExtension, Empty>;
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

    PaymentDapp::default().instantiate(deps, env, info, msg.base)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> PaymentResult {
    let dapp = PaymentDapp::default();
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, dapp, msg),
        ExecuteMsg::Pay { asset, os_id } => commands::try_pay(deps, info, dapp, asset, None, os_id),
        ExecuteMsg::Claim { page_limit } => commands::try_claim(deps, env, info, dapp, page_limit),
        ExecuteMsg::UpdateContributor {
            contributor_addr,
            compensation,
        } => commands::update_contributor(deps, info, dapp, contributor_addr, compensation),
        ExecuteMsg::RemoveContributor { contributor_addr } => {
            commands::remove_contributor(deps, info, dapp, contributor_addr)
        }
        ExecuteMsg::Base(dapp_msg) => {
            from_base_dapp_result(dapp.execute(deps, env, info, dapp_msg))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // handle dapp-specific queries here
        QueryMsg::State {} => {
            let state = STATE.load(deps.storage)?;
            to_binary(&StateResponse {
                income: state.income,
                total_weight: state.total_weight,
                next_pay_day: state.next_pay_day,
            })
        }
        QueryMsg::Base(message) => PaymentDapp::default().query(deps, env, message),
    }
}

/// Required to convert DappResult into PaymentResult
/// Can't implement the From trait directly
fn from_base_dapp_result(result: DappResult) -> PaymentResult {
    match result {
        Err(e) => Err(e.into()),
        Ok(r) => Ok(r),
    }
}
