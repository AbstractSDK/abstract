use abstract_ica::{
    msg::{
        state::{Config, CONFIG},
        ConfigResponse,
    },
    ChainType, IcaAction, IcaActionResponse,
};
use abstract_std::objects::TruncatedChainId;
use cosmwasm_std::{ensure_eq, CosmosMsg, Deps, Env};

use crate::{chain_types::evm, contract::IcaClientResult, error::IcaClientError};

/// Timeout in seconds
pub const PACKET_LIFETIME: u64 = 60 * 60;

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
    env: Env,
    _proxy_address: String,
    chain: TruncatedChainId,
    mut actions: Vec<IcaAction>,
) -> IcaClientResult<IcaActionResponse> {
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

    let cfg = CONFIG.load(deps.storage)?;

    let process_action = |action: IcaAction| -> IcaClientResult<Vec<CosmosMsg>> {
        match action {
            IcaAction::Execute(ica_exec) => match ica_exec {
                abstract_ica::IcaExecute::Evm { msgs, callback } => {
                    ensure_eq!(
                        chain_type,
                        ChainType::Evm,
                        IcaClientError::WrongChainType {
                            chain: chain.to_string(),
                            ty: chain_type.to_string()
                        }
                    );

                    let msg = evm::execute(&deps.querier, &cfg.version_control, msgs, callback)?;

                    Ok(vec![msg.into()])
                }
                _ => unimplemented!(),
            },
            IcaAction::Fund {
                funds,
                receiver,
                memo,
            } => match chain_type {
                ChainType::Evm => {
                    Ok(vec![evm::send_funds(
                        deps, &env, &chain, &cfg, funds, receiver, memo,
                    )?])
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    };

    // TODO: can we use `flat_map` here?
    let maybe_msgs: Result<Vec<Vec<CosmosMsg>>, _> =
        actions.into_iter().map(process_action).collect();
    let msgs = maybe_msgs?.into_iter().flatten().collect();

    Ok(IcaActionResponse { msgs })
}
