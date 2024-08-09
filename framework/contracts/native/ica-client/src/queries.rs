use std::str::FromStr;

use abstract_ica::{IcaAction, IcaActionResponse, EVM_NOTE_ID};
use abstract_std::{
    ibc_client::{
        state::{Config, ACCOUNTS, CONFIG, IBC_INFRA},
        AccountResponse, ConfigResponse, HostResponse, ListAccountsResponse,
        ListIbcInfrastructureResponse, ListRemoteHostsResponse, ListRemoteProxiesResponse,
    },
    objects::{
        account::{AccountSequence, AccountTrace}, module::ModuleInfo, AccountId, ContractEntry, TruncatedChainId
    },
    AbstractError,
};
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps, Order, StdError, StdResult};
use cw_storage_plus::Bound;

use crate::{contract::IcaClientResult, error::IcaClientError};

/// Timeout in seconds
const DEFAULT_TIMEOUT: u64 = 3600;

pub fn config(deps: Deps) -> IcaClientResult<ConfigResponse> {
    let Config {
        version_control,
        ans_host,
    } = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        ans_host: ans_host.address.to_string(),
        version_control_address: version_control.address.into_string(),
    })
}

pub(crate) fn ica_action(
    deps: Deps,
    proxy_address: String,
    chain: TruncatedChainId,
    mut actions: Vec<IcaAction>,
) -> IcaClientResult<IcaActionResponse> {
    let proxy_addr = deps.api.addr_validate(&proxy_address)?;

    // match chain-id with cosmos or EVM
    use abstract_ica::CastChainType;
    let chain_type = chain.chain_type().ok_or(IcaClientError::NoChainType {
        chain: chain.to_string(),
    })?;

    // todo: what do we do for msgs that contain both cosmos and EVM messages?
    // Best to err if there's conflict.

    // sort actions
    // 1) Transfers
    // 2) Calls
    // 3) Queries
    actions.sort_unstable();

    let 


    let cfg = CONFIG.load(deps.storage)?;
    let process_action = |action: IcaAction| -> IcaClientResult<Vec<CosmosMsg>> {
        match action {
            IcaAction::Execute(ica_exec) => {
                match ica_exec {
                    abstract_ica::IcaExecute::Evm { msgs, callback } => {
                        let evm_note_entry = ModuleInfo::from_id(EVM_NOTE_ID, abstract_ica::EVM_NOTE_VERSION.parse()?)?;
                        // TODO: query VC for native contract
                        let note_addr = cfg.version_control.query_module(evm_note_entry, &deps.querier)?.reference.unwrap_native()?;

                        let msg = wasm_execute(note_addr, &evm_note::msg::ExecuteMsg::Execute {
                            msgs,
                            callback,
                            timeout_seconds: DEFAULT_TIMEOUT.into(),
                        }, vec![])?;

                    },
                    _ => unimplemented!(),
                }
            },
            IcaAction::Fund(funds) => {

            },
            _ => unimplemented!(),
        }
    };


    let mut msgs = Vec::new();

    actions.into_iter().map(process_action)
}
