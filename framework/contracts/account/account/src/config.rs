use crate::{
    contract::{AccountResponse, AccountResult},
    error::AccountError,
    modules::update_module_addresses,
};
use abstract_sdk::{cw_helpers::AbstractAttributes, feature_objects::VersionControlContract};
use abstract_std::{
    account::{
        state::{
            AccountInfo, SuspensionStatus, ACCOUNT_ID, CONFIG, INFO, SUB_ACCOUNTS,
            SUSPENSION_STATUS,
        },
        types::{InternalConfigAction, UpdateSubAccountAction}, ExecuteMsg,
    },
    objects::{
        gov_type::GovernanceDetails,
        ownership,
        validation::{validate_description, validate_link, validate_name},
    },
};
use cosmwasm_std::{
    ensure, from_json, wasm_execute, Binary, CosmosMsg, DepsMut, MessageInfo, Response, StdError,
};

pub fn update_account_status(
    deps: DepsMut,
    info: MessageInfo,
    suspension_status: Option<bool>,
) -> Result<Response, AccountError> {
    let mut response = AccountResponse::action("update_status");

    if let Some(suspension_status) = suspension_status {
        response = update_suspension_status(deps, info, suspension_status, response)?;
    } else {
        return Err(AccountError::NoUpdates {});
    }

    Ok(response)
}

pub fn update_suspension_status(
    deps: DepsMut,
    info: MessageInfo,
    is_suspended: SuspensionStatus,
    response: Response,
) -> AccountResult {
    // only owner can update suspension status
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    SUSPENSION_STATUS.save(deps.storage, &is_suspended)?;

    Ok(response.add_abstract_attributes(vec![("is_suspended", is_suspended.to_string())]))
}

/// Allows the owner to manually update the internal configuration of the account.
/// This can be used to unblock the account and its modules in case of a bug/lock on the account.
pub fn update_internal_config(deps: DepsMut, info: MessageInfo, config: Binary) -> AccountResult {
    // deserialize the config action
    let action: InternalConfigAction =
        from_json(config).map_err(|error| AccountError::InvalidConfigAction { error })?;

    let (add, remove) = match action {
        InternalConfigAction::UpdateModuleAddresses { to_add, to_remove } => (to_add, to_remove),
        _ => {
            return Err(AccountError::InvalidConfigAction {
                error: StdError::generic_err("Unknown config action"),
            })
        }
    };

    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;
    update_module_addresses(deps, add, remove)
}

/// Update the Account information
pub fn update_info(
    deps: DepsMut,
    info: MessageInfo,
    name: Option<String>,
    description: Option<String>,
    link: Option<String>,
) -> AccountResult {
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    let mut info: AccountInfo = INFO.load(deps.storage)?;
    if let Some(name) = name {
        validate_name(&name)?;
        info.name = name;
    }
    validate_description(description.as_deref())?;
    info.description = description;
    validate_link(link.as_deref())?;
    info.link = link;
    INFO.save(deps.storage, &info)?;

    Ok(AccountResponse::action("update_info"))
}

/// Renounce ownership of this account \
/// **WARNING**: This will lock the account, making it unusable.
pub fn remove_account_from_contracts(deps: DepsMut) -> AccountResult<Vec<CosmosMsg>> {
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
                &ExecuteMsg::UpdateSubAccount(UpdateSubAccountAction::UnregisterSubAccount {
                    id: account_id.seq(),
                }),
                vec![],
            )?
            .into(),
        );
    }

    let config = CONFIG.load(deps.storage)?;
    let vc = VersionControlContract::new(config.version_control_address);
    let mut namespaces = vc
        .query_namespaces(vec![account_id], &deps.querier)?
        .namespaces;
    let namespace = namespaces.pop();
    if let Some((namespace, _)) = namespace {
        // Remove the namespace that this account holds.
        msgs.push(
            wasm_execute(
                vc.address,
                &abstract_std::version_control::ExecuteMsg::RemoveNamespaces {
                    namespaces: vec![namespace.to_string()],
                },
                vec![],
            )?
            .into(),
        )
    };
    Ok(msgs)
}
