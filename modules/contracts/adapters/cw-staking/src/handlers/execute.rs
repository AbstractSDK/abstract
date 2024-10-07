use abstract_adapter::sdk::{
    feature_objects::AnsHost,
    features::{AbstractNameService, AbstractResponse, AccountIdentification},
    IbcInterface, Resolve,
};
use abstract_adapter::std::ibc::Callback;
use abstract_adapter::std::objects::TruncatedChainId;
use abstract_staking_standard::msg::{ExecuteMsg, ProviderName, StakingAction, StakingExecuteMsg};
use cosmwasm_std::{to_json_binary, Coin, Deps, DepsMut, Env, MessageInfo};

use crate::{
    adapter::CwStakingAdapter,
    contract::{CwStakingAdapter as CwStakingContract, StakingResult},
    resolver::{self, is_over_ibc},
    CW_STAKING_ADAPTER_ID,
};

/// Execute staking operation locally or over IBC
pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: CwStakingContract,
    msg: StakingExecuteMsg,
) -> StakingResult {
    let StakingExecuteMsg {
        provider: provider_name,
        action,
    } = msg;
    // if provider is on an app-chain, execute the action on the app-chain
    let (local_provider_name, is_over_ibc) = is_over_ibc(&env, &provider_name)?;
    if is_over_ibc {
        handle_ibc_request(&deps, info, &module, local_provider_name, &action)
    } else {
        // the action can be executed on the local chain
        handle_local_request(deps, env, info, module, action, local_provider_name)
    }
}

/// Handle an adapter request that can be executed on the local chain
fn handle_local_request(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    module: CwStakingContract,
    action: StakingAction,
    provider_name: String,
) -> StakingResult {
    let provider = resolver::resolve_local_provider(&provider_name)?;
    let target_account = module.account(deps.as_ref())?;
    Ok(module
        .custom_response("handle_local_request", vec![("provider", provider_name)])
        .add_submessage(module.resolve_staking_action(
            deps,
            env,
            target_account,
            action,
            provider,
        )?))
}

/// Handle a request that needs to be executed on a remote chain
/// TODO, this doesn't work as is. This should be corrected when working with ibc hooks ?
fn handle_ibc_request(
    deps: &DepsMut,
    info: MessageInfo,
    module: &CwStakingContract,
    provider_name: ProviderName,
    action: &StakingAction,
) -> StakingResult {
    let host_chain = TruncatedChainId::from_string(provider_name.clone())?; // TODO : Especially this line is faulty
    let ans = module.name_service(deps.as_ref());
    let ibc_client = module.ibc_client(deps.as_ref());
    // get the to-be-sent assets from the action
    let coins = resolve_assets_to_transfer(deps.as_ref(), action, ans.host())?;
    // construct the ics20 call(s)
    let ics20_transfer_msg = ibc_client.ics20_transfer(host_chain.clone(), coins, None)?;
    // construct the action to be called on the host
    // construct the action to be called on the host
    let host_action = abstract_adapter::std::ibc_host::HostAction::Dispatch {
        account_msgs: vec![
            abstract_adapter::std::account::ExecuteMsg::ExecuteOnModule {
                module_id: CW_STAKING_ADAPTER_ID.to_string(),
                exec_msg: to_json_binary::<ExecuteMsg>(
                    &StakingExecuteMsg {
                        provider: provider_name.clone(),
                        action: action.clone(),
                    }
                    .into(),
                )?,
            },
        ],
    };

    // If the calling entity is a contract, we provide a callback on successful cross-chain-staking
    let maybe_contract_info = deps.querier.query_wasm_contract_info(info.sender.clone());
    let _callback = if maybe_contract_info.is_err() {
        None
    } else {
        Some(Callback {
            msg: to_json_binary(&StakingExecuteMsg {
                provider: provider_name.clone(),
                action: action.clone(),
            })?,
        })
    };
    let ibc_action_msg = ibc_client.host_action(host_chain, host_action)?;

    Ok(module
        .custom_response("handle_ibc_request", vec![("provider", provider_name)])
        // call both messages on the account
        .add_messages(vec![ics20_transfer_msg, ibc_action_msg]))
}

/// Resolve the assets to be transferred to the host chain for the given action
fn resolve_assets_to_transfer(
    deps: Deps,
    dex_action: &StakingAction,
    ans_host: &AnsHost,
) -> StakingResult<Vec<Coin>> {
    match dex_action {
        StakingAction::Stake { assets, .. } => {
            let resolved: Vec<Coin> = assets
                .resolve(&deps.querier, ans_host)?
                .into_iter()
                .map(Coin::try_from)
                .collect::<Result<_, cw_asset::AssetError>>()?;
            Ok(resolved)
        }
        _ => Ok(vec![]),
    }
}
