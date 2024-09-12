use abstract_std::{
    account::{
        state::{ACCOUNT_ID, CONFIG, SUB_ACCOUNTS},
        ExecuteMsg, ModuleInstallConfig, UpdateSubAccountAction,
    },
    objects::{
        gov_type::GovernanceDetails,
        module::ModuleInfo,
        ownership::{self, GovOwnershipError},
        AccountId,
    },
};
use cosmwasm_std::{wasm_execute, Attribute, CosmosMsg, DepsMut, Empty, Env, MessageInfo};

use crate::{
    contract::{AccountResponse, AccountResult},
    error::AccountError,
    modules::query_module,
};
#[allow(clippy::too_many_arguments)]
/// Creates a sub-account for this account,
pub fn create_sub_account(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    name: String,
    description: Option<String>,
    link: Option<String>,
    namespace: Option<String>,
    install_modules: Vec<ModuleInstallConfig>,
    account_id: Option<u32>,
) -> AccountResult {
    // only owner can create a subaccount
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    let create_account_msg = &abstract_std::account_factory::ExecuteMsg::CreateAccount {
        // proxy of this manager will be the account owner
        governance: GovernanceDetails::SubAccount {
            account: env.contract.address.into_string(),
        },
        name,
        description,
        link,
        namespace,
        install_modules,
        account_id: account_id.map(AccountId::local),
    };

    let account_factory_addr = query_module(
        deps.as_ref(),
        ModuleInfo::from_id_latest(abstract_std::ACCOUNT_FACTORY)?,
        None,
    )?
    .module
    .reference
    .unwrap_native()?;

    // Call factory and attach all funds that were provided.
    let account_creation_message =
        wasm_execute(account_factory_addr, create_account_msg, info.funds)?;

    let response = AccountResponse::new::<_, Attribute>("create_sub_account", vec![])
        .add_message(account_creation_message);

    Ok(response)
}

pub fn handle_sub_account_action(
    deps: DepsMut,
    info: MessageInfo,
    action: UpdateSubAccountAction,
) -> AccountResult {
    match action {
        UpdateSubAccountAction::UnregisterSubAccount { id } => {
            unregister_sub_account(deps, info, id)
        }
        UpdateSubAccountAction::RegisterSubAccount { id } => register_sub_account(deps, info, id),
        _ => unimplemented!(),
    }
}

// Unregister sub-account from the state
fn unregister_sub_account(deps: DepsMut, info: MessageInfo, id: u32) -> AccountResult {
    let config = CONFIG.load(deps.storage)?;

    let account = abstract_std::version_control::state::ACCOUNT_ADDRESSES.query(
        &deps.querier,
        config.version_control_address,
        &AccountId::local(id),
    )?;

    if account.is_some_and(|a| a.addr() == info.sender) {
        SUB_ACCOUNTS.remove(deps.storage, id);

        Ok(AccountResponse::new(
            "unregister_sub_account",
            vec![("sub_account_removed", id.to_string())],
        ))
    } else {
        Err(AccountError::SubAccountRemovalFailed {})
    }
}

// Register sub-account to the state
fn register_sub_account(deps: DepsMut, info: MessageInfo, id: u32) -> AccountResult {
    let config = CONFIG.load(deps.storage)?;

    let account = abstract_std::version_control::state::ACCOUNT_ADDRESSES.query(
        &deps.querier,
        config.version_control_address,
        &AccountId::local(id),
    )?;

    if account.is_some_and(|a| a.addr() == info.sender) {
        SUB_ACCOUNTS.save(deps.storage, id, &Empty {})?;

        Ok(AccountResponse::new(
            "register_sub_account",
            vec![("sub_account_added", id.to_string())],
        ))
    } else {
        Err(AccountError::SubAccountRegisterFailed {})
    }
}

/// Update governance of sub_accounts account after claim
pub fn maybe_update_sub_account_governance(deps: DepsMut) -> AccountResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];
    let mut account_id = None;
    let ownership = ownership::get_ownership(deps.storage)?;
    // Get pending governance
    let pending_governance = ownership
        .pending_owner
        .ok_or(GovOwnershipError::TransferNotFound)?;

    // Clear state for previous manager if it was sub-account
    if let GovernanceDetails::SubAccount { account } = ownership.owner {
        let id = ACCOUNT_ID.load(deps.storage)?;
        let unregister_message = wasm_execute(
            account,
            &ExecuteMsg::UpdateSubAccount(UpdateSubAccountAction::UnregisterSubAccount {
                id: id.seq(),
            }),
            vec![],
        )?;
        // For optimizing the gas we save it, in case new owner is sub-account as well
        account_id = Some(id);
        msgs.push(unregister_message.into());
    }

    // Update state for new manager if owner will be the sub-account
    if let GovernanceDetails::SubAccount { account } = &pending_governance {
        let id = if let Some(id) = account_id {
            id
        } else {
            ACCOUNT_ID.load(deps.storage)?
        };
        let register_message = wasm_execute(
            account,
            &ExecuteMsg::UpdateSubAccount(UpdateSubAccountAction::RegisterSubAccount {
                id: id.seq(),
            }),
            vec![],
        )?;
        msgs.push(register_message.into());
    }

    Ok(msgs)
}
