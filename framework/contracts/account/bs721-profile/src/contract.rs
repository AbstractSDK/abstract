use crate::{
    state::{PROFILE_MARKETPLACE, SUDO_PARAMS, VERIFIER},
    InstantiateMsg, SudoParams,
};
use abstract_std::PROFILE;
use semver::Version;

use crate::{commands::*, ContractError};
use bs721_base::{ContractError as Bs721ContractError, MintMsg};
use bs_profile::Metadata;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;
use cw_utils::maybe_addr;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Bs721NameContract<'a> = bs721_base::Bs721Contract<'a, Metadata, Empty, Empty, Empty>;
pub type ExecuteMsg = crate::ExecuteMsg<Metadata>;
pub type QueryMsg = crate::QueryMsg;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Bs721ContractError> {
    cw2::set_contract_version(deps.storage, PROFILE, CONTRACT_VERSION)?;

    // Initialize max record count to 10, can be changed by sudo params
    SUDO_PARAMS.save(
        deps.storage,
        &SudoParams {
            max_record_count: 10,
        },
    )?;

    let api = deps.api;
    VERIFIER.set(deps.branch(), maybe_addr(api, msg.verifier)?)?;
    PROFILE_MARKETPLACE.save(deps.storage, &msg.marketplace)?;

    let res =
        Bs721NameContract::default().instantiate(deps, env.clone(), info, msg.base_init_msg)?;

    Ok(res
        .add_attribute("action", "instantiate")
        .add_attribute("bs721_profile_address", env.contract.address.to_string()))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::AssociateAddress { name, address } => {
            execute_associate_address(deps, info, name, address)
        }
        ExecuteMsg::UpdateImageNft { name, nft } => execute_update_image_nft(deps, info, name, nft),
        ExecuteMsg::AddTextRecord { name, record } => {
            execute_add_text_record(deps, info, name, record)
        }
        ExecuteMsg::RemoveTextRecord { name, record_name } => {
            execute_remove_text_record(deps, info, name, record_name)
        }
        ExecuteMsg::UpdateTextRecord { name, record } => {
            execute_update_text_record(deps, info, name, record)
        }
        ExecuteMsg::VerifyTextRecord {
            name,
            record_name,
            result,
        } => execute_verify_text_record(deps, info, name, record_name, result),
        ExecuteMsg::UpdateVerifier { verifier } => {
            Ok(VERIFIER.execute_update_admin(deps, info, maybe_addr(api, verifier)?)?)
        }
        ExecuteMsg::SetNameMarketplace { address } => {
            execute_set_profile_marketplace(deps, info, address)
        }
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => execute_transfer_nft(deps, env, info, recipient, token_id),
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => execute_send_nft(deps, env, info, contract, token_id, msg),
        ExecuteMsg::Mint(msg) => execute_mint(
            deps,
            info,
            bs721_base::ExecuteMsg::Mint(MintMsg::<Metadata> {
                token_id: msg.token_id,
                owner: msg.owner,
                token_uri: None,
                extension: msg.extension,
                seller_fee_bps: msg.seller_fee_bps,
                payment_addr: msg.payment_addr,
            }),
        ),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
        _ => Bs721NameContract::default()
            .execute(deps, env, info, msg.into())
            .map_err(|e| e.into()),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
        QueryMsg::NameMarketplace {} => to_json_binary(&query_profile_marketplace(deps)?),
        QueryMsg::Name { address } => to_json_binary(&query_name(deps, address)?),
        QueryMsg::Verifier {} => to_json_binary(&VERIFIER.query_admin(deps)?),
        QueryMsg::AssociatedAddress { name } => {
            to_json_binary(&query_associated_address(deps, &name)?)
        }
        QueryMsg::ImageNFT { name } => to_json_binary(&query_image_nft(deps, &name)?),
        QueryMsg::TextRecords { name } => to_json_binary(&query_text_records(deps, &name)?),
        QueryMsg::IsTwitterVerified { name } => {
            to_json_binary(&query_is_twitter_verified(deps, &name)?)
        }
        _ => Bs721NameContract::default().query(deps, env, msg.into()),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    let current_version = cw2::get_contract_version(deps.storage)?;
    if current_version.contract != PROFILE {
        return Err(StdError::generic_err("Cannot upgrade to a different contract").into());
    }
    let version: Version = current_version
        .version
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;
    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;

    if version > new_version {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    // if same version return
    if version == new_version {
        return Ok(Response::new());
    }

    // set new contract version
    set_contract_version(deps.storage, PROFILE, CONTRACT_VERSION)?;
    Ok(Response::new())
}
