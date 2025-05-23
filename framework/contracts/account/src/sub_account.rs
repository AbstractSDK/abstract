use abstract_sdk::feature_objects::RegistryContract;
use abstract_std::{
    account::{
        state::{ACCOUNT_ID, SUB_ACCOUNTS},
        ExecuteMsg, ModuleInstallConfig, UpdateSubAccountAction,
    },
    native_addrs,
    objects::{
        gov_type::GovernanceDetails,
        ownership::{self, GovOwnershipError},
        salt, AccountId,
    },
};
use cosmwasm_std::{
    ensure, instantiate2_address, to_json_binary, wasm_execute, Attribute, CosmosMsg, DepsMut,
    Empty, Env, MessageInfo, WasmMsg,
};

use crate::{
    contract::{AccountResponse, AccountResult},
    error::AccountError,
};
#[allow(clippy::too_many_arguments)]
/// Creates a sub-account for this account,
pub fn create_sub_account(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    name: Option<String>,
    description: Option<String>,
    link: Option<String>,
    namespace: Option<String>,
    install_modules: Vec<ModuleInstallConfig>,
    account_id: Option<u32>,
) -> AccountResult {
    // only owner can create a subaccount
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;
    let self_code_id = deps
        .querier
        .query_wasm_contract_info(env.contract.address.clone())?
        .code_id;
    let registry = RegistryContract::new(deps.as_ref(), self_code_id)?;
    let seq = account_id.unwrap_or(
        abstract_std::registry::state::LOCAL_ACCOUNT_SEQUENCE
            .query(&deps.querier, registry.address.clone())?,
    );
    let account_id = AccountId::local(seq);
    let salt = salt::generate_instantiate_salt(&account_id);

    let checksum = deps.querier.query_wasm_code_info(self_code_id)?.checksum;
    let self_canon_addr = deps.api.addr_canonicalize(env.contract.address.as_str())?;

    let create_account_msg = abstract_std::account::InstantiateMsg {
        code_id: self_code_id,
        account_id: Some(account_id.clone()),
        owner: Some(GovernanceDetails::SubAccount {
            account: env.contract.address.into_string(),
        }),
        namespace,
        install_modules,
        name,
        description,
        link,
        authenticator: None::<Empty>,
    };

    let account_canon_addr =
        instantiate2_address(checksum.as_slice(), &self_canon_addr, salt.as_slice())?;
    let account_addr = deps.api.addr_humanize(&account_canon_addr)?;

    // Call factory and attach all funds that were provided.
    let account_creation_message = WasmMsg::Instantiate2 {
        admin: Some(account_addr.to_string()),
        code_id: self_code_id,
        label: account_id.to_string(),
        msg: to_json_binary(&create_account_msg)?,
        funds: info.funds,
        salt,
    };

    let response = AccountResponse::new::<_, Attribute>("create_sub_account", vec![])
        .add_message(account_creation_message);

    Ok(response)
}

pub fn handle_sub_account_action(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    action: UpdateSubAccountAction,
) -> AccountResult {
    match action {
        UpdateSubAccountAction::UnregisterSubAccount { id } => {
            unregister_sub_account(deps, env, info, id)
        }
        UpdateSubAccountAction::RegisterSubAccount { id } => {
            register_sub_account(deps, env, info, id)
        }
        _ => unimplemented!(),
    }
}

// Unregister sub-account from the state
fn unregister_sub_account(deps: DepsMut, env: &Env, info: MessageInfo, id: u32) -> AccountResult {
    let abstract_code_id =
        native_addrs::abstract_code_id(&deps.querier, env.contract.address.clone())?;
    let registry = RegistryContract::new(deps.as_ref(), abstract_code_id)?;

    let account = abstract_std::registry::state::ACCOUNT_ADDRESSES.query(
        &deps.querier,
        registry.address,
        &AccountId::local(id),
    )?;

    if let Some(account) = account {
        if account.addr() == info.sender {
            SUB_ACCOUNTS.remove(deps.storage, id);

            Ok(AccountResponse::new(
                "unregister_sub_account",
                vec![("sub_account_removed", id.to_string())],
            ))
        } else {
            Err(AccountError::SubAccountIsNotCaller {})
        }
    } else {
        Err(AccountError::SubAccountDoesntExist {})
    }
}

// Register sub-account to the state
fn register_sub_account(deps: DepsMut, env: &Env, info: MessageInfo, id: u32) -> AccountResult {
    let abstract_code_id =
        native_addrs::abstract_code_id(&deps.querier, env.contract.address.clone())?;
    let registry = RegistryContract::new(deps.as_ref(), abstract_code_id)?;

    let account = abstract_std::registry::state::ACCOUNT_ADDRESSES.query(
        &deps.querier,
        registry.address,
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

    // Clear state for previous account if it was sub-account
    if let GovernanceDetails::SubAccount { account } = ownership.owner {
        let id = ACCOUNT_ID.load(deps.storage)?;
        let unregister_message = wasm_execute(
            account,
            &ExecuteMsg::UpdateSubAccount::<cosmwasm_std::Empty>(
                UpdateSubAccountAction::UnregisterSubAccount { id: id.seq() },
            ),
            vec![],
        )?;
        // For optimizing the gas we save it, in case new owner is sub-account as well
        account_id = Some(id);
        msgs.push(unregister_message.into());
    }

    // Update state for new account if owner will be the sub-account
    if let GovernanceDetails::SubAccount { account } = &pending_governance {
        let id = if let Some(id) = account_id {
            id
        } else {
            ACCOUNT_ID.load(deps.storage)?
        };
        let register_message = wasm_execute(
            account,
            &ExecuteMsg::UpdateSubAccount::<cosmwasm_std::Empty>(
                UpdateSubAccountAction::RegisterSubAccount { id: id.seq() },
            ),
            vec![],
        )?;
        msgs.push(register_message.into());
    }

    Ok(msgs)
}

/// Renounce ownership of this account \
/// **WARNING**: This will lock the account, making it unusable.
pub fn remove_account_from_contracts(deps: DepsMut, env: &Env) -> AccountResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];

    let account_id = ACCOUNT_ID.load(deps.storage)?;
    // Check for any sub accounts
    let sub_account = SUB_ACCOUNTS
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .next()
        .transpose()?;
    ensure!(
        sub_account.is_none(),
        AccountError::RenounceWithSubAccount {}
    );

    let ownership = ownership::get_ownership(deps.storage)?;
    if let GovernanceDetails::SubAccount { account } = ownership.owner {
        // Unregister itself (sub-account) from the owning account.
        msgs.push(
            wasm_execute(
                account,
                &ExecuteMsg::UpdateSubAccount::<cosmwasm_std::Empty>(
                    UpdateSubAccountAction::UnregisterSubAccount {
                        id: account_id.seq(),
                    },
                ),
                vec![],
            )?
            .into(),
        );
    }

    let abstract_code_id =
        native_addrs::abstract_code_id(&deps.querier, env.contract.address.clone())?;
    let registry = RegistryContract::new(deps.as_ref(), abstract_code_id)?;
    let mut namespaces = registry
        .query_namespaces(vec![account_id], &deps.querier)?
        .namespaces;
    let namespace = namespaces.pop();
    if let Some((namespace, _)) = namespace {
        // Remove the namespace that this account holds.
        msgs.push(
            wasm_execute(
                registry.address,
                &abstract_std::registry::ExecuteMsg::ForgoNamespace {
                    namespaces: vec![namespace.to_string()],
                },
                vec![],
            )?
            .into(),
        )
    };
    Ok(msgs)
}
