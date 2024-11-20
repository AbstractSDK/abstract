use abstract_sdk::{
    cw_helpers::Clearable,
    std::{
        objects::{
            module::{ModuleInfo, ModuleVersion},
            module_reference::ModuleReference,
            namespace::Namespace,
            AccountId,
        },
        registry::{state::*, Account, Config},
    },
};
use abstract_std::{
    account::state::ACCOUNT_ID,
    objects::{
        fee::FixedFee,
        module::{self, Module},
        ownership,
        validation::validate_link,
        ABSTRACT_ACCOUNT_ID,
    },
    registry::{state::LOCAL_ACCOUNT_SEQUENCE, ModuleDefaultConfiguration, UpdateModule},
    ACCOUNT, IBC_HOST,
};
use cosmwasm_std::{
    ensure, ensure_eq, Addr, Attribute, BankMsg, Coin, CosmosMsg, Deps, DepsMut, MessageInfo,
    Order, QuerierWrapper, StdResult, Storage,
};

use crate::{
    contract::{VCResult, VcResponse, ABSTRACT_NAMESPACE},
    error::RegistryError,
};

/// Add new Account to registry contract
/// Only Account can add itself.
pub fn add_account(
    deps: DepsMut,
    msg_info: MessageInfo,
    namespace: Option<String>,
    creator: String,
) -> VCResult {
    let config = CONFIG.load(deps.storage)?;

    // Check that sender is account
    let contract_info = cw2::query_contract_info(&deps.querier, &msg_info.sender)?;
    let maybe_acc_module_info = ModuleInfo::try_from(contract_info)?;

    ensure!(
        maybe_acc_module_info.id() == ACCOUNT,
        RegistryError::NotAccountInfo {
            caller_info: maybe_acc_module_info
        }
    );
    let acc_module_info = maybe_acc_module_info;

    // Ensure account isn't already registered
    let account_id = ACCOUNT_ID.query(&deps.querier, msg_info.sender.clone())?;
    ensure!(
        !ACCOUNT_ADDRESSES.has(deps.storage, &account_id),
        RegistryError::AccountAlreadyExists(account_id)
    );

    // verify code-id of sender
    let sender_contract_info = deps.querier.query_wasm_contract_info(&msg_info.sender)?;

    let account_code_id = REGISTERED_MODULES
        .load(deps.storage, &acc_module_info)?
        .unwrap_account()?;

    ensure_eq!(
        account_code_id,
        sender_contract_info.code_id,
        RegistryError::NotAccountCodeId {
            account_info: acc_module_info,
            expected_code_id: account_code_id,
            actual_code_id: sender_contract_info.code_id
        }
    );

    // provided and smaller, assert is eq to sequence
    // provided and larger, just register
    // Remote account_id is provided, assert the creator is the ibc host.
    if account_id.is_local() {
        // Predictable Account Id Sequence have to be >= 2147483648
        if account_id.seq() < 2147483648 {
            let next_sequence = LOCAL_ACCOUNT_SEQUENCE.may_load(deps.storage)?.unwrap_or(0);

            ensure_eq!(
                next_sequence,
                account_id.seq(),
                RegistryError::InvalidAccountSequence {
                    expected: next_sequence,
                    actual: account_id.seq(),
                }
            );

            LOCAL_ACCOUNT_SEQUENCE.save(deps.storage, &next_sequence.checked_add(1).unwrap())?;
        }
    } else {
        // If a non-local account_id is provided, assert that the creator is the ibc host
        let creator_addr = deps.api.addr_validate(&creator)?;
        let sender_cw2_info = cw2::query_contract_info(&deps.querier, &creator_addr)?;
        let ibc_host_addr = REGISTERED_MODULES
            .load(
                deps.storage,
                &ModuleInfo::from_id(IBC_HOST, sender_cw2_info.version.into())?,
            )?
            .unwrap_native()?;

        ensure_eq!(
            creator_addr,
            ibc_host_addr,
            RegistryError::SenderNotIbcHost(creator_addr.into_string(), ibc_host_addr.into())
        );
        // then assert that the account trace is remote and properly formatted
        account_id.trace().verify_remote()?;
    }

    // Now we're sure that the account is valid.
    let account = Account::new(msg_info.sender.clone());

    ACCOUNT_ADDRESSES.save(deps.storage, &account_id, &account)?;

    let fee_msg = if let Some(namespace) = &namespace {
        claim_namespace_internal(
            deps.storage,
            config.namespace_registration_fee,
            msg_info,
            account_id.clone(),
            namespace,
        )?
    } else {
        None
    };

    let mut response = VcResponse::new(
        "add_account",
        vec![
            ("account_id", account_id.to_string().as_str()),
            ("account_address", account.addr().as_str()),
            ("namespace", &format!("{namespace:?}")),
        ],
    );

    if let Some(msg) = fee_msg {
        response = response.add_message(msg);
    }
    Ok(response)
}

/// Here we can add logic to allow subscribers to claim a namespace and upload contracts to that namespace
pub fn propose_modules(
    deps: DepsMut,
    msg_info: MessageInfo,
    modules: Vec<(ModuleInfo, ModuleReference)>,
) -> VCResult {
    let config = CONFIG.load(deps.storage)?;

    for (module, mod_ref) in modules {
        let store_has_module = PENDING_MODULES.has(deps.storage, &module)
            || REGISTERED_MODULES.has(deps.storage, &module)
            || YANKED_MODULES.has(deps.storage, &module);
        if config.security_enabled && store_has_module {
            return Err(RegistryError::NotUpdateableModule(module));
        }

        module.validate()?;

        mod_ref.validate(deps.as_ref())?;

        // version must be set in order to add the new version
        module.assert_version_variant()?;

        if module.namespace == Namespace::unchecked(ABSTRACT_NAMESPACE) {
            // Only Admin can update abstract contracts
            cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;
        } else {
            // Only owner can add modules
            validate_account_owner(deps.as_ref(), &module.namespace, &msg_info.sender)?;
        }

        // verify contract admin is None if module is Adapter
        if let ModuleReference::Adapter(ref addr) = mod_ref {
            if deps.querier.query_wasm_contract_info(addr)?.admin.is_some() {
                return Err(RegistryError::AdminMustBeNone);
            }
        }

        if config.security_enabled {
            PENDING_MODULES.save(deps.storage, &module, &mod_ref)?;
        } else {
            // assert that its data is equal to what it wants to be registered under.
            module::assert_module_data_validity(
                &deps.querier,
                &Module {
                    info: module.clone(),
                    reference: mod_ref.clone(),
                },
                None,
            )?;
            REGISTERED_MODULES.save(deps.storage, &module, &mod_ref)?;
            // Help querying version for cw-2-less or mis-formatted contracts
            match mod_ref {
                // Save module info of standalone contracts,
                ModuleReference::Standalone(id) => {
                    STANDALONE_INFOS.save(deps.storage, id, &module)?;
                }
                // Save module info of service contracts,
                ModuleReference::Service(addr) => {
                    SERVICE_INFOS.save(deps.storage, &addr, &module)?;
                }
                _ => (),
            }
        }
    }

    Ok(VcResponse::action("propose_modules"))
}

/// Approve and reject modules
pub fn approve_or_reject_modules(
    deps: DepsMut,
    msg_info: MessageInfo,
    approves: Vec<ModuleInfo>,
    rejects: Vec<ModuleInfo>,
) -> VCResult {
    // Only Admin can approve or rejects a module
    cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;

    let mut attributes = vec![];
    if !approves.is_empty() {
        attributes.push(approve_modules(deps.storage, approves)?);
    }
    if !rejects.is_empty() {
        attributes.push(reject_modules(deps.storage, rejects)?);
    }
    if attributes.is_empty() {
        return Err(RegistryError::NoAction);
    }

    Ok(VcResponse::new("approve_or_reject_modules", attributes))
}

/// Admin approve modules
fn approve_modules(storage: &mut dyn Storage, approves: Vec<ModuleInfo>) -> VCResult<Attribute> {
    for module in &approves {
        let mod_ref = PENDING_MODULES
            .may_load(storage, module)?
            .ok_or_else(|| RegistryError::ModuleNotFound(module.clone()))?;
        // Register the module
        REGISTERED_MODULES.save(storage, module, &mod_ref)?;
        // Remove from pending
        PENDING_MODULES.remove(storage, module);

        // Save module info of standalone contracts,
        // helps querying version for cw-2-less or mis-formatted contracts
        if let ModuleReference::Standalone(id) = mod_ref {
            STANDALONE_INFOS.save(storage, id, module)?;
        }
    }

    let approves: Vec<_> = approves.into_iter().map(|m| m.to_string()).collect();
    Ok(("approves", approves.join(",")).into())
}

/// Admin reject modules
fn reject_modules(storage: &mut dyn Storage, rejects: Vec<ModuleInfo>) -> VCResult<Attribute> {
    for module in &rejects {
        if !PENDING_MODULES.has(storage, module) {
            return Err(RegistryError::ModuleNotFound(module.clone()));
        }
        PENDING_MODULES.remove(storage, module);
    }

    let rejects: Vec<_> = rejects.into_iter().map(|m| m.to_string()).collect();
    Ok(("rejects", rejects.join(",")).into())
}

/// Remove a module from the Registry registry.
pub fn remove_module(deps: DepsMut, msg_info: MessageInfo, module: ModuleInfo) -> VCResult {
    // Only the Registry Admin can remove modules
    cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;

    // Only specific versions may be removed
    module.assert_version_variant()?;

    let module_ref_res = REGISTERED_MODULES.load(deps.storage, &module);

    ensure!(
        module_ref_res.is_ok() || YANKED_MODULES.has(deps.storage, &module),
        RegistryError::ModuleNotFound(module)
    );

    REGISTERED_MODULES.remove(deps.storage, &module);
    YANKED_MODULES.remove(deps.storage, &module);
    MODULE_CONFIG.remove(deps.storage, &module);

    // Remove standalone info
    if let Ok(ModuleReference::Standalone(id)) = module_ref_res {
        STANDALONE_INFOS.remove(deps.storage, id);
    }

    // If this module has no more versions, we also remove default configuration
    if REGISTERED_MODULES
        .prefix((module.namespace.clone(), module.name.clone()))
        .range(deps.storage, None, None, Order::Ascending)
        .next()
        .is_none()
    {
        MODULE_DEFAULT_CONFIG.remove(deps.storage, (&module.namespace, &module.name));
    }
    Ok(VcResponse::new(
        "remove_module",
        vec![("module", &module.to_string())],
    ))
}

/// Yank a module, preventing it from being used.
pub fn yank_module(deps: DepsMut, msg_info: MessageInfo, module: ModuleInfo) -> VCResult {
    // validate the caller is the owner of the namespace
    validate_account_owner(deps.as_ref(), &module.namespace, &msg_info.sender)?;

    // Only specific versions may be yanked
    module.assert_version_variant()?;
    let mod_ref = REGISTERED_MODULES
        .may_load(deps.storage, &module)?
        .ok_or_else(|| RegistryError::ModuleNotFound(module.clone()))?;

    YANKED_MODULES.save(deps.storage, &module, &mod_ref)?;
    REGISTERED_MODULES.remove(deps.storage, &module);

    Ok(VcResponse::new(
        "yank_module",
        vec![("module", &module.to_string())],
    ))
}

/// Updates module configuration
pub fn update_module_config(
    deps: DepsMut,
    msg_info: MessageInfo,
    module_name: String,
    namespace: Namespace,
    update_module: UpdateModule,
) -> VCResult {
    // validate the caller is the owner of the namespace

    if namespace == Namespace::unchecked(ABSTRACT_NAMESPACE) {
        // Only Admin can update abstract contracts
        cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;
    } else {
        // Only owner can add modules
        validate_account_owner(deps.as_ref(), &namespace, &msg_info.sender)?;
    }

    match update_module {
        UpdateModule::Default { metadata } => {
            // Check there is at least one version of this module
            if REGISTERED_MODULES
                .prefix((namespace.clone(), module_name.clone()))
                .range(deps.storage, None, None, Order::Ascending)
                .next()
                .is_none()
            {
                return Err(RegistryError::ModuleNotFound(ModuleInfo {
                    namespace,
                    name: module_name,
                    version: ModuleVersion::Latest,
                }));
            }

            validate_link(Some(&metadata))?;

            MODULE_DEFAULT_CONFIG.save(
                deps.storage,
                (&namespace, &module_name),
                &ModuleDefaultConfiguration::new(metadata),
            )?;
        }
        UpdateModule::Versioned {
            version,
            metadata,
            monetization,
            instantiation_funds,
        } => {
            let module = ModuleInfo {
                namespace: namespace.clone(),
                name: module_name.clone(),
                version: ModuleVersion::Version(version),
            };

            // We verify the module exists before updating the config
            let Some(module_reference) = REGISTERED_MODULES.may_load(deps.storage, &module)? else {
                return Err(RegistryError::ModuleNotFound(module));
            };

            let mut current_cfg = MODULE_CONFIG
                .may_load(deps.storage, &module)?
                .unwrap_or_default();
            // Update metadata
            if let Some(metadata) = metadata {
                current_cfg.metadata = Some(metadata);
            }

            // Update monetization
            if let Some(monetization) = monetization {
                current_cfg.monetization = monetization;
            }

            // Update init funds
            if let Some(init_funds) = instantiation_funds {
                if matches!(
                    module_reference,
                    ModuleReference::App(_) | ModuleReference::Standalone(_)
                ) {
                    current_cfg.instantiation_funds = init_funds
                } else {
                    return Err(RegistryError::RedundantInitFunds {});
                }
            }
            MODULE_CONFIG.save(deps.storage, &module, &current_cfg)?;
        }
        _ => todo!(),
    };

    Ok(VcResponse::new(
        "update_module_config",
        vec![
            ("namespace", &namespace.to_string()),
            ("module_name", &module_name),
        ],
    ))
}

/// Claim namespaces
/// Only the Account Owner can do this
pub fn claim_namespace(
    deps: DepsMut,
    msg_info: MessageInfo,
    account_id: AccountId,
    namespace_to_claim: String,
) -> VCResult {
    let Config {
        namespace_registration_fee: fee,
        security_enabled,
        ..
    } = CONFIG.load(deps.storage)?;

    if security_enabled {
        // When security is enabled, only the contract admin can claim namespaces
        cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;
    } else {
        // If there is no security, only account owner can register a namespace
        let account = ACCOUNT_ADDRESSES.load(deps.storage, &account_id)?;
        let account_owner = query_account_owner(&deps.querier, account.into_addr(), &account_id)?;

        if msg_info.sender != account_owner {
            return Err(RegistryError::AccountOwnerMismatch {
                sender: msg_info.sender,
                owner: account_owner,
            });
        }
    }

    let fee_msg = claim_namespace_internal(
        deps.storage,
        fee,
        msg_info,
        account_id.clone(),
        &namespace_to_claim,
    )?;

    let mut response = VcResponse::new(
        "claim_namespace",
        vec![
            ("account_id", account_id.to_string()),
            ("namespaces", namespace_to_claim),
        ],
    );

    if let Some(msg) = fee_msg {
        response = response.add_message(msg);
    }
    Ok(response)
}

/// Claim namespace internal
fn claim_namespace_internal(
    storage: &mut dyn Storage,
    fee: Option<Coin>,
    msg_info: MessageInfo,
    account_id: AccountId,
    namespace_to_claim: &str,
) -> VCResult<Option<CosmosMsg>> {
    // check if the account already has a namespace
    if REV_NAMESPACES.has(storage, &account_id) {
        return Err(RegistryError::ExceedsNamespaceLimit {
            limit: 1,
            current: 1,
        });
    }

    let fee_msg = if let Some(fee) = fee {
        // assert it is paid
        FixedFee::new(&fee).assert_payment(&msg_info)?;

        // We transfer the namespace fee if necessary
        let admin_account = ACCOUNT_ADDRESSES.load(storage, &ABSTRACT_ACCOUNT_ID)?;
        Some(CosmosMsg::Bank(BankMsg::Send {
            to_address: admin_account.addr().to_string(),
            amount: msg_info.funds,
        }))
    } else {
        None
    };

    let namespace = Namespace::try_from(namespace_to_claim)?;
    if let Some(id) = NAMESPACES.may_load(storage, &namespace)? {
        return Err(RegistryError::NamespaceOccupied {
            namespace: namespace.to_string(),
            id,
        });
    }
    NAMESPACES.save(storage, &namespace, &account_id)?;
    REV_NAMESPACES.save(storage, &account_id, &namespace)?;

    Ok(fee_msg)
}

/// Forgo namespaces
/// Only admin or the account owner can do this
pub fn forgo_namespace(deps: DepsMut, msg_info: MessageInfo, namespaces: Vec<String>) -> VCResult {
    let is_admin = cw_ownable::is_owner(deps.storage, &msg_info.sender)?;

    let mut logs = vec![];
    for namespace in namespaces.iter() {
        let namespace = Namespace::try_from(namespace)?;
        if !NAMESPACES.has(deps.storage, &namespace) {
            return Err(RegistryError::UnknownNamespace { namespace });
        }
        if !is_admin {
            validate_account_owner(deps.as_ref(), &namespace, &msg_info.sender)?;
        }

        for ((name, version), mod_ref) in REGISTERED_MODULES
            .sub_prefix(namespace.clone())
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?
            .into_iter()
        {
            let module = ModuleInfo {
                namespace: namespace.clone(),
                name,
                version,
            };
            REGISTERED_MODULES.remove(deps.storage, &module);
            YANKED_MODULES.save(deps.storage, &module, &mod_ref)?;
        }

        let owner = NAMESPACES.load(deps.storage, &namespace)?;
        logs.push(format!("({namespace}, {owner})"));
        NAMESPACES.remove(deps.storage, &namespace);
        REV_NAMESPACES.remove(deps.storage, &owner);
    }

    Ok(VcResponse::new(
        "forgo_namespace",
        vec![("namespaces", &logs.join(","))],
    ))
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    security_enabled: Option<bool>,
    namespace_registration_fee: Option<Clearable<Coin>>,
) -> VCResult {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut config = CONFIG.load(deps.storage)?;

    let mut attributes = vec![];

    if let Some(security_enabled) = security_enabled {
        let previous_security_enabled = config.security_enabled;
        config.security_enabled = security_enabled;
        attributes.extend(vec![
            (
                "previous_security_enabled",
                previous_security_enabled.to_string(),
            ),
            (
                "allow_direct_module_registration_and_updates",
                security_enabled.to_string(),
            ),
        ])
    }

    if let Some(fee) = namespace_registration_fee {
        let previous_fee = config.namespace_registration_fee;
        let fee: Option<Coin> = fee.into();
        config.namespace_registration_fee = fee;
        attributes.extend(vec![
            (
                "previous_namespace_registration_fee",
                format!("{:?}", previous_fee),
            ),
            (
                "namespace_registration_fee",
                format!("{:?}", config.namespace_registration_fee),
            ),
        ])
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(VcResponse::new("update_config", attributes))
}

pub fn query_account_owner(
    querier: &QuerierWrapper,
    account_addr: Addr,
    account_id: &AccountId,
) -> VCResult<Addr> {
    let ownership::Ownership { owner, .. } = ownership::query_ownership(querier, account_addr)?;

    owner
        .owner_address(querier)
        .ok_or_else(|| RegistryError::NoAccountOwner {
            account_id: account_id.clone(),
        })
}

pub fn validate_account_owner(
    deps: Deps,
    namespace: &Namespace,
    sender: &Addr,
) -> Result<(), RegistryError> {
    let sender = sender.clone();
    let account_id = NAMESPACES
        .may_load(deps.storage, &namespace.clone())?
        .ok_or_else(|| RegistryError::UnknownNamespace {
            namespace: namespace.to_owned(),
        })?;
    let account = ACCOUNT_ADDRESSES.load(deps.storage, &account_id)?;
    let account = account.addr();
    // Check account first, account can call this function to unregister a namespace when renouncing its ownership.
    if sender != account {
        let account_owner = query_account_owner(&deps.querier, account.clone(), &account_id)?;
        if sender != account_owner {
            return Err(RegistryError::AccountOwnerMismatch {
                sender,
                owner: account_owner,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_sdk::namespaces::OWNERSHIP_STORAGE_KEY;
    use abstract_std::{objects::account::AccountTrace, registry::*, ACCOUNT};
    use abstract_testing::{abstract_mock_querier_builder, prelude::*};
    use cosmwasm_std::{
        from_json,
        testing::{message_info, mock_dependencies, MockApi},
        Addr, Coin, OwnedDeps,
    };
    use cw_ownable::OwnershipError;
    use cw_storage_plus::Item;
    use ownership::{GovernanceDetails, Ownership};

    use super::*;
    use crate::contract;

    type RegistryTestResult = Result<(), RegistryError>;

    const TEST_OTHER: &str = "test-other";
    const FIRST_ACCOUNT: &str = "first-account";
    const SECOND_ACCOUNT: &str = "second-account";
    const THIRD_ACCOUNT: &str = "third-account";

    pub const FIRST_TEST_ACCOUNT_ID: AccountId = AccountId::const_new(1, AccountTrace::Local);
    pub const SECOND_TEST_ACCOUNT_ID: AccountId = AccountId::const_new(2, AccountTrace::Local);
    pub const THIRD_TEST_ACCOUNT_ID: AccountId = AccountId::const_new(3, AccountTrace::Local);

    fn registry_mock_deps() -> MockDeps {
        let mut deps = mock_dependencies();

        let querier = registry_mock_querier_builder(deps.api).build();

        deps.querier = querier;

        deps
    }

    fn registry_mock_querier_builder(api: MockApi) -> MockQuerierBuilder {
        let abstr = AbstractMockAddrs::new(api);

        let owner = Ownership {
            owner: GovernanceDetails::Monarchy {
                monarch: abstr.owner.clone(),
            },
            pending_owner: None,
            pending_expiry: None,
        };

        let first_acc_addr = api.addr_make(FIRST_ACCOUNT);
        let second_acc_addr = api.addr_make(SECOND_ACCOUNT);
        let third_acc_addr = api.addr_make(THIRD_ACCOUNT);

        const OWNERSHIP: Item<Ownership<Addr>> = Item::new(OWNERSHIP_STORAGE_KEY);

        abstract_mock_querier_builder(api)
            .with_contract_version(&first_acc_addr, ACCOUNT, TEST_VERSION)
            .with_contract_version(&second_acc_addr, ACCOUNT, TEST_VERSION)
            .with_contract_version(&third_acc_addr, ACCOUNT, TEST_VERSION)
            .with_contract_item(&first_acc_addr, OWNERSHIP, &owner)
            .with_contract_item(&second_acc_addr, OWNERSHIP, &owner)
            .with_contract_item(&third_acc_addr, OWNERSHIP, &owner)
            .with_contract_item(&first_acc_addr, ACCOUNT_ID, &FIRST_TEST_ACCOUNT_ID)
            .with_contract_item(&second_acc_addr, ACCOUNT_ID, &SECOND_TEST_ACCOUNT_ID)
            .with_contract_item(&third_acc_addr, ACCOUNT_ID, &THIRD_TEST_ACCOUNT_ID)
    }

    /// Initialize the registry with admin and updated account_factory
    fn mock_init(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) -> VCResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let env = mock_env_validated(deps.api);
        let info = message_info(&abstr.owner, &[]);
        let admin = info.sender.to_string();

        let resp = contract::instantiate(
            deps.as_mut(),
            env,
            info.clone(),
            InstantiateMsg {
                admin,
                security_enabled: Some(false),
                namespace_registration_fee: None,
            },
        )?;

        REGISTERED_MODULES.save(
            &mut deps.storage,
            &ModuleInfo::from_id(ACCOUNT, ModuleVersion::Version(TEST_VERSION.into())).unwrap(),
            &ModuleReference::Account(1),
        )?;

        // register abstract account
        execute_as(
            deps,
            &abstr.account.addr().clone(),
            ExecuteMsg::AddAccount {
                namespace: None,
                creator: abstr.owner.to_string(),
            },
        )?;

        Ok(resp)
    }

    /// Initialize the registry with admin as creator and test account
    fn mock_init_with_account(
        deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
        security_enabled: bool,
    ) -> VCResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let admin_info = message_info(&abstr.owner, &[]);
        let env = mock_env_validated(deps.api);
        let admin = admin_info.sender.to_string();

        let resp = contract::instantiate(
            deps.as_mut(),
            env,
            admin_info,
            InstantiateMsg {
                admin,
                security_enabled: Some(security_enabled),
                namespace_registration_fee: None,
            },
        )?;

        REGISTERED_MODULES.save(
            &mut deps.storage,
            &ModuleInfo::from_id(ACCOUNT, ModuleVersion::Version(TEST_VERSION.into())).unwrap(),
            &ModuleReference::Account(1),
        )?;

        execute_as(
            deps,
            &abstr.account.addr().clone(),
            ExecuteMsg::AddAccount {
                namespace: None,
                creator: abstr.owner.to_string(),
            },
        )?;

        let first_account = Account::new(deps.api.addr_make(FIRST_ACCOUNT));

        execute_as(
            deps,
            first_account.addr(),
            ExecuteMsg::AddAccount {
                namespace: None,
                creator: abstr.owner.to_string(),
            },
        )?;

        Ok(resp)
    }

    fn create_second_account(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) {
        let abstr = AbstractMockAddrs::new(deps.api);

        let second_account = Account::new(deps.api.addr_make(SECOND_ACCOUNT));

        // create second account
        execute_as(
            deps,
            second_account.addr(),
            ExecuteMsg::AddAccount {
                namespace: None,
                creator: abstr.owner.to_string(),
            },
        )
        .unwrap();
    }

    pub const THIRD_ACC_ID: AccountId = AccountId::const_new(3, AccountTrace::Local);

    fn create_third_account(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) -> Account {
        let abstr = AbstractMockAddrs::new(deps.api);

        let third_account = Account::new(deps.api.addr_make(THIRD_ACCOUNT));
        // create third account
        execute_as(
            deps,
            third_account.addr(),
            ExecuteMsg::AddAccount {
                namespace: None,
                creator: abstr.owner.to_string(),
            },
        )
        .unwrap();
        third_account
    }

    fn execute_as(deps: &mut MockDeps, sender: &Addr, msg: ExecuteMsg) -> VCResult {
        execute_as_with_funds(deps, sender, msg, &[])
    }

    fn execute_as_with_funds(
        deps: &mut MockDeps,
        sender: &Addr,
        msg: ExecuteMsg,
        funds: &[Coin],
    ) -> VCResult {
        let env = mock_env_validated(deps.api);
        contract::execute(deps.as_mut(), env, message_info(sender, funds), msg)
    }

    fn test_only_admin(
        msg: ExecuteMsg,
        deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) -> RegistryTestResult {
        mock_init(deps)?;

        let not_owner = deps.api.addr_make("not_owner");
        let res = execute_as(deps, &not_owner, msg);
        assert_eq!(
            res,
            Err(RegistryError::Ownership(OwnershipError::NotOwner {}))
        );

        Ok(())
    }

    mod set_admin_and_factory {
        use super::*;

        #[coverage_helper::test]
        fn only_admin_admin() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: deps.api.addr_make("new_admin").to_string(),
                expiry: None,
            });

            test_only_admin(msg, &mut deps)
        }

        #[coverage_helper::test]
        fn updates_admin() -> RegistryTestResult {
            let mut deps = registry_mock_deps();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let new_admin = deps.api.addr_make("new_admin");
            // First update to transfer
            let transfer_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: new_admin.to_string(),
                expiry: None,
            });

            let transfer_res = execute_as(&mut deps, &abstr.owner, transfer_msg).unwrap();
            assert_eq!(0, transfer_res.messages.len());

            // Then update and accept as the new owner
            let accept_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership);
            let accept_res = execute_as(&mut deps, &new_admin, accept_msg).unwrap();
            assert_eq!(0, accept_res.messages.len());

            assert_eq!(
                cw_ownable::get_ownership(&deps.storage).unwrap().owner,
                Some(new_admin)
            );

            Ok(())
        }
    }

    mod claim_namespace {
        use super::*;

        use abstract_std::AbstractError;
        use cosmwasm_std::{coins, SubMsg};

        #[coverage_helper::test]
        fn claim_namespaces_by_owner() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let new_namespace1 = Namespace::new("namespace1").unwrap();
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: FIRST_TEST_ACCOUNT_ID,
                namespace: new_namespace1.to_string(),
            };
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());

            create_second_account(&mut deps);

            let new_namespace2 = Namespace::new("namespace2").unwrap();
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: SECOND_TEST_ACCOUNT_ID,
                namespace: new_namespace2.to_string(),
            };
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());

            let account_id = NAMESPACES.load(&deps.storage, &new_namespace1)?;
            assert_eq!(account_id, FIRST_TEST_ACCOUNT_ID);
            let account_id = NAMESPACES.load(&deps.storage, &new_namespace2)?;
            assert_eq!(account_id, SECOND_TEST_ACCOUNT_ID);
            Ok(())
        }

        #[coverage_helper::test]
        fn fail_claim_permissioned_namespaces() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, true)?;
            let new_namespace1 = Namespace::new("namespace1").unwrap();
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: FIRST_TEST_ACCOUNT_ID,
                namespace: new_namespace1.to_string(),
            };
            // OWNER is also admin of the contract so this succeeds
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());

            let account_id = NAMESPACES.load(&deps.storage, &new_namespace1)?;
            assert_eq!(account_id, FIRST_TEST_ACCOUNT_ID);

            create_second_account(&mut deps);

            let account = create_third_account(&mut deps);

            let new_namespace2 = Namespace::new("namespace2").unwrap();

            let msg = ExecuteMsg::ClaimNamespace {
                account_id: THIRD_ACC_ID,
                namespace: new_namespace2.to_string(),
            };

            let res = execute_as(&mut deps, account.addr(), msg);
            assert_eq!(res, Err(RegistryError::Ownership(OwnershipError::NotOwner)));

            Ok(())
        }

        #[coverage_helper::test]
        fn claim_namespaces_with_fee() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);

            mock_init_with_account(&mut deps, false)?;

            let one_namespace_fee = Coin {
                denom: "ujunox".to_string(),
                amount: 6u128.into(),
            };

            execute_as(
                &mut deps,
                &abstr.owner,
                ExecuteMsg::UpdateConfig {
                    security_enabled: None,
                    namespace_registration_fee: Clearable::new_opt(one_namespace_fee.clone()),
                },
            )
            .unwrap();

            let new_namespace1 = Namespace::new("namespace1").unwrap();
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: FIRST_TEST_ACCOUNT_ID,
                namespace: new_namespace1.to_string(),
            };

            // Fail, no fee at all
            let res = execute_as(&mut deps, &abstr.owner, msg.clone());
            assert_eq!(
                res,
                Err(RegistryError::Abstract(AbstractError::Fee(format!(
                    "Invalid fee payment sent. Expected {}, sent {:?}",
                    Coin {
                        denom: one_namespace_fee.denom.clone(),
                        amount: one_namespace_fee.amount,
                    },
                    Vec::<Coin>::new()
                ))))
            );

            // Fail, not enough fee
            let sent_coins = coins(5, "ujunox");
            let res = execute_as_with_funds(&mut deps, &abstr.owner, msg.clone(), &sent_coins);
            assert_eq!(
                res,
                Err(RegistryError::Abstract(AbstractError::Fee(format!(
                    "Invalid fee payment sent. Expected {}, sent {:?}",
                    Coin {
                        denom: one_namespace_fee.denom.clone(),
                        amount: one_namespace_fee.amount,
                    },
                    sent_coins
                ))))
            );

            // Success
            let sent_coins = coins(6, "ujunox");
            let res = execute_as_with_funds(&mut deps, &abstr.owner, msg, &sent_coins);
            assert_eq!(
                res.map(|res| res.messages),
                Ok(vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                    to_address: abstr.account.addr().to_string(),
                    amount: sent_coins,
                }))])
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn claim_namespaces_not_owner() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let new_namespace1 = Namespace::new("namespace1").unwrap();
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: FIRST_TEST_ACCOUNT_ID,
                namespace: new_namespace1.to_string(),
            };

            let other = deps.api.addr_make(TEST_OTHER);
            let res = execute_as(&mut deps, &other, msg);
            assert_eq!(
                res,
                Err(RegistryError::AccountOwnerMismatch {
                    sender: other,
                    owner: abstr.owner,
                })
            );
            Ok(())
        }

        #[coverage_helper::test]
        fn claim_existing_namespaces() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;

            create_second_account(&mut deps);

            let new_namespace1 = Namespace::new("namespace1")?;
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: FIRST_TEST_ACCOUNT_ID,
                namespace: new_namespace1.to_string(),
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            let msg = ExecuteMsg::ClaimNamespace {
                account_id: SECOND_TEST_ACCOUNT_ID,
                namespace: new_namespace1.to_string(),
            };
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert_eq!(
                res,
                Err(RegistryError::NamespaceOccupied {
                    namespace: new_namespace1.to_string(),
                    id: FIRST_TEST_ACCOUNT_ID,
                })
            );
            Ok(())
        }

        #[coverage_helper::test]
        fn cannot_claim_abstract() -> VCResult<()> {
            let mut deps = registry_mock_deps();
            let abstr = AbstractMockAddrs::new(deps.api);

            mock_init_with_account(&mut deps, false)?;

            // Add account 1
            create_second_account(&mut deps);

            // Attempt to claim the abstract namespace with account 1
            let claim_abstract_msg: ExecuteMsg = ExecuteMsg::ClaimNamespace {
                account_id: SECOND_TEST_ACCOUNT_ID,
                namespace: ABSTRACT_NAMESPACE.to_string(),
            };
            let res = execute_as(&mut deps, &abstr.owner, claim_abstract_msg);
            assert_eq!(
                res,
                Err(RegistryError::NamespaceOccupied {
                    namespace: Namespace::try_from("abstract")?.to_string(),
                    id: ABSTRACT_ACCOUNT_ID,
                })
            );
            Ok(())
        }
    }

    mod update_direct_registration {
        use super::*;

        #[coverage_helper::test]
        fn only_admin() -> RegistryTestResult {
            let mut deps = registry_mock_deps();
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateConfig {
                security_enabled: Some(false),
                namespace_registration_fee: None,
            };

            let other = deps.api.addr_make(TEST_OTHER);
            let res = execute_as(&mut deps, &other, msg);
            assert_eq!(res, Err(RegistryError::Ownership(OwnershipError::NotOwner)));

            Ok(())
        }

        #[coverage_helper::test]
        fn direct_registration() -> RegistryTestResult {
            let mut deps = registry_mock_deps();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let msg = ExecuteMsg::UpdateConfig {
                security_enabled: Some(true),
                namespace_registration_fee: None,
            };

            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());

            assert!(CONFIG.load(&deps.storage).unwrap().security_enabled);

            Ok(())
        }
    }

    mod update_namespace_fee {
        use cosmwasm_std::Uint128;

        use super::*;

        #[coverage_helper::test]
        fn only_admin() -> RegistryTestResult {
            let mut deps = registry_mock_deps();
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateConfig {
                security_enabled: None,
                namespace_registration_fee: Clearable::new_opt(Coin {
                    denom: "ujunox".to_string(),
                    amount: Uint128::one(),
                }),
            };

            let other = deps.api.addr_make(TEST_OTHER);
            let res = execute_as(&mut deps, &other, msg);
            assert_eq!(res, Err(RegistryError::Ownership(OwnershipError::NotOwner)));

            Ok(())
        }

        #[coverage_helper::test]
        fn updates_fee() -> RegistryTestResult {
            let mut deps = registry_mock_deps();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let new_fee = Coin {
                denom: "ujunox".to_string(),
                amount: Uint128::one(),
            };

            let msg = ExecuteMsg::UpdateConfig {
                security_enabled: None,
                namespace_registration_fee: Clearable::new_opt(new_fee.clone()),
            };

            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());

            assert_eq!(
                CONFIG
                    .load(&deps.storage)
                    .unwrap()
                    .namespace_registration_fee,
                Some(new_fee)
            );

            Ok(())
        }
    }

    mod forgo_namespace {
        use super::*;

        use cosmwasm_std::attr;

        fn test_module() -> ModuleInfo {
            ModuleInfo::from_id(TEST_MODULE_ID, ModuleVersion::Version(TEST_VERSION.into()))
                .unwrap()
        }

        #[coverage_helper::test]
        fn forgo_namespace_by_admin_or_owner() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let new_namespace1 = Namespace::new("namespace1").unwrap();
            let new_namespace2 = Namespace::new("namespace2").unwrap();

            // add namespaces
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: FIRST_TEST_ACCOUNT_ID,
                namespace: new_namespace1.to_string(),
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            // remove as admin
            let msg = ExecuteMsg::ForgoNamespace {
                namespaces: vec![new_namespace1.to_string()],
            };
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());
            let exists = NAMESPACES.has(&deps.storage, &new_namespace1);
            assert!(!exists);

            let msg = ExecuteMsg::ClaimNamespace {
                account_id: FIRST_TEST_ACCOUNT_ID,
                namespace: new_namespace2.to_string(),
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            // remove as owner
            let msg = ExecuteMsg::ForgoNamespace {
                namespaces: vec![new_namespace2.to_string()],
            };
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());
            let exists = NAMESPACES.has(&deps.storage, &new_namespace2);
            assert!(!exists);
            assert_eq!(
                res.unwrap().events[0].attributes[2],
                attr(
                    "namespaces",
                    format!("({}, {})", new_namespace2, FIRST_TEST_ACCOUNT_ID,),
                )
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn remove_namespaces_as_other() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let new_namespace1 = Namespace::new("namespace1")?;

            // add namespaces
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: FIRST_TEST_ACCOUNT_ID,
                namespace: new_namespace1.to_string(),
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            // remove as other
            let msg = ExecuteMsg::ForgoNamespace {
                namespaces: vec![new_namespace1.to_string()],
            };
            let other = deps.api.addr_make(TEST_OTHER);
            let res = execute_as(&mut deps, &other, msg);
            assert_eq!(
                res,
                Err(RegistryError::AccountOwnerMismatch {
                    sender: other,
                    owner: abstr.owner,
                })
            );
            Ok(())
        }

        #[coverage_helper::test]
        fn remove_not_existing_namespaces() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let new_namespace1 = Namespace::new("namespace1")?;

            // remove as owner
            let msg = ExecuteMsg::ForgoNamespace {
                namespaces: vec![new_namespace1.to_string()],
            };
            let res = execute_as(&mut deps, &abstr.owner, msg.clone());
            assert_eq!(
                res,
                Err(RegistryError::UnknownNamespace {
                    namespace: new_namespace1.clone(),
                })
            );

            // remove as admin
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert_eq!(
                res,
                Err(RegistryError::UnknownNamespace {
                    namespace: new_namespace1,
                })
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn yank_orphaned_modules() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;

            // add namespaces
            let new_namespace1 = Namespace::new("namespace1")?;
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: FIRST_TEST_ACCOUNT_ID,
                namespace: new_namespace1.to_string(),
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            // first add module
            let mut new_module = test_module();
            new_module.namespace = new_namespace1.clone();
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            // remove as admin
            let msg = ExecuteMsg::ForgoNamespace {
                namespaces: vec![new_namespace1.to_string()],
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            let module = REGISTERED_MODULES.load(&deps.storage, &new_module);
            assert!(module.is_err());
            let module = YANKED_MODULES.load(&deps.storage, &new_module)?;
            assert_eq!(module, ModuleReference::App(0));
            Ok(())
        }
    }

    mod propose_modules {
        use super::*;

        use crate::contract::query;
        use abstract_std::{objects::module::Monetization, AbstractError};
        use cosmwasm_std::coin;

        fn test_module() -> ModuleInfo {
            ModuleInfo::from_id(TEST_MODULE_ID, ModuleVersion::Version(TEST_VERSION.into()))
                .unwrap()
        }

        // - Query latest

        #[coverage_helper::test]
        fn add_module_by_admin() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let mut new_module = test_module();
            new_module.namespace = Namespace::new(ABSTRACT_NAMESPACE)?;
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());
            let module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;
            assert_eq!(module, ModuleReference::App(0));
            Ok(())
        }

        #[coverage_helper::test]
        fn add_module_by_account_owner() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let new_module = test_module();

            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };

            // try while no namespace
            let res = execute_as(&mut deps, &abstr.owner, msg.clone());
            assert_eq!(
                res,
                Err(RegistryError::UnknownNamespace {
                    namespace: new_module.namespace.clone(),
                })
            );

            // add namespaces
            execute_as(
                &mut deps,
                &abstr.owner,
                ExecuteMsg::ClaimNamespace {
                    account_id: FIRST_TEST_ACCOUNT_ID,
                    namespace: new_module.namespace.to_string(),
                },
            )?;

            // add modules
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());
            let module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;
            assert_eq!(module, ModuleReference::App(0));
            Ok(())
        }

        #[coverage_helper::test]
        fn update_existing_module() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let new_module = test_module();

            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };

            // add namespaces
            execute_as(
                &mut deps,
                &abstr.owner,
                ExecuteMsg::ClaimNamespace {
                    account_id: FIRST_TEST_ACCOUNT_ID,
                    namespace: new_module.namespace.to_string(),
                },
            )?;

            // add modules
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());
            let module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;
            assert_eq!(module, ModuleReference::App(0));

            // update module code-id without changing version

            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(1))],
            };

            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());
            let module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;
            assert_eq!(module, ModuleReference::App(1));
            Ok(())
        }

        #[coverage_helper::test]
        fn update_existing_module_fails() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, true)?;
            let new_module = test_module();

            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };

            // add namespaces
            execute_as(
                &mut deps,
                &abstr.owner,
                ExecuteMsg::ClaimNamespace {
                    account_id: FIRST_TEST_ACCOUNT_ID,
                    namespace: new_module.namespace.to_string(),
                },
            )?;

            // add modules
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());
            // approve
            let msg = ExecuteMsg::ApproveOrRejectModules {
                approves: vec![new_module.clone()],
                rejects: vec![],
            };

            // approve by admin
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());

            let module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;
            assert_eq!(module, ModuleReference::App(0));

            // try update module code-id without changing version

            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(1))],
            };
            let res = execute_as(&mut deps, &abstr.owner, msg);
            // should error as module is already approved and registered.
            assert!(res.is_err());

            let module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;
            assert_eq!(module, ModuleReference::App(0));
            Ok(())
        }

        #[coverage_helper::test]
        fn try_add_module_to_approval_with_admin() -> RegistryTestResult {
            let mut deps = registry_mock_deps();
            let contract_addr = deps.api.addr_make("contract");
            // create mock with ContractInfo response for contract with admin set
            deps.querier = registry_mock_querier_builder(deps.api)
                .with_contract_admin(&contract_addr, &deps.api.addr_make("admin"))
                .build();

            let abstr = AbstractMockAddrs::new(deps.api);

            mock_init_with_account(&mut deps, true)?;
            let new_module = test_module();

            let mod_ref = ModuleReference::Adapter(contract_addr);

            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), mod_ref)],
            };

            // try while no namespace
            let res = execute_as(&mut deps, &abstr.owner, msg.clone());
            assert_eq!(
                res,
                Err(RegistryError::UnknownNamespace {
                    namespace: new_module.namespace.clone(),
                })
            );

            // add namespaces
            execute_as(
                &mut deps,
                &abstr.owner,
                ExecuteMsg::ClaimNamespace {
                    account_id: FIRST_TEST_ACCOUNT_ID,
                    namespace: new_module.namespace.to_string(),
                },
            )?;

            // assert we got admin must be none error
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert_eq!(res, Err(RegistryError::AdminMustBeNone));

            Ok(())
        }

        #[coverage_helper::test]
        fn add_module_to_approval() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, true)?;
            let new_module = test_module();

            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };

            // try while no namespace
            let res = execute_as(&mut deps, &abstr.owner, msg.clone());
            assert_eq!(
                res,
                Err(RegistryError::UnknownNamespace {
                    namespace: new_module.namespace.clone(),
                })
            );

            // add namespaces
            execute_as(
                &mut deps,
                &abstr.owner,
                ExecuteMsg::ClaimNamespace {
                    account_id: FIRST_TEST_ACCOUNT_ID,
                    namespace: new_module.namespace.to_string(),
                },
            )?;

            // add modules
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());
            let module = PENDING_MODULES.load(&deps.storage, &new_module)?;
            assert_eq!(module, ModuleReference::App(0));
            Ok(())
        }

        #[coverage_helper::test]
        fn approve_modules() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, true)?;
            let new_module = test_module();

            // add namespaces
            execute_as(
                &mut deps,
                &abstr.owner,
                ExecuteMsg::ClaimNamespace {
                    account_id: FIRST_TEST_ACCOUNT_ID,
                    namespace: new_module.namespace.to_string(),
                },
            )?;
            // add modules
            execute_as(
                &mut deps,
                &abstr.owner,
                ExecuteMsg::ProposeModules {
                    modules: vec![(new_module.clone(), ModuleReference::App(0))],
                },
            )?;

            let msg = ExecuteMsg::ApproveOrRejectModules {
                approves: vec![new_module.clone()],
                rejects: vec![],
            };

            // approve by not owner
            let not_owner = deps.api.addr_make("not_owner");
            let res = execute_as(&mut deps, &not_owner, msg.clone());
            assert_eq!(
                res,
                Err(RegistryError::Ownership(OwnershipError::NotOwner {}))
            );

            // approve by admin
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());
            let module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;
            assert_eq!(module, ModuleReference::App(0));
            let pending = PENDING_MODULES.has(&deps.storage, &new_module);
            assert!(!pending);

            Ok(())
        }

        #[coverage_helper::test]
        fn reject_modules() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, true)?;
            let new_module = test_module();

            // add namespaces
            execute_as(
                &mut deps,
                &abstr.owner,
                ExecuteMsg::ClaimNamespace {
                    account_id: FIRST_TEST_ACCOUNT_ID,
                    namespace: new_module.namespace.to_string(),
                },
            )?;
            // add modules
            execute_as(
                &mut deps,
                &abstr.owner,
                ExecuteMsg::ProposeModules {
                    modules: vec![(new_module.clone(), ModuleReference::App(0))],
                },
            )?;

            let msg = ExecuteMsg::ApproveOrRejectModules {
                approves: vec![],
                rejects: vec![new_module.clone()],
            };

            // reject by not owner
            let not_owner = deps.api.addr_make("not_owner");
            let res = execute_as(&mut deps, &not_owner, msg.clone());
            assert_eq!(
                res,
                Err(RegistryError::Ownership(OwnershipError::NotOwner {}))
            );

            // reject by admin
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());
            let exists = REGISTERED_MODULES.has(&deps.storage, &new_module);
            assert!(!exists);
            let pending = PENDING_MODULES.has(&deps.storage, &new_module);
            assert!(!pending);

            Ok(())
        }

        #[coverage_helper::test]
        fn remove_module() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let rm_module = test_module();

            // add namespaces
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: FIRST_TEST_ACCOUNT_ID,
                namespace: rm_module.namespace.to_string(),
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            // first add module
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(rm_module.clone(), ModuleReference::App(0))],
            };
            execute_as(&mut deps, &abstr.owner, msg)?;
            let module = REGISTERED_MODULES.load(&deps.storage, &rm_module)?;
            assert_eq!(module, ModuleReference::App(0));

            // then remove
            let msg = ExecuteMsg::RemoveModule {
                module: rm_module.clone(),
            };
            // as other, should fail
            let other = deps.api.addr_make(TEST_OTHER);
            let res = execute_as(&mut deps, &other, msg.clone());
            assert_eq!(
                res,
                Err(RegistryError::Ownership(OwnershipError::NotOwner {}))
            );

            // only admin can remove modules.
            execute_as(&mut deps, &abstr.owner, msg)?;

            let module = REGISTERED_MODULES.load(&deps.storage, &rm_module);
            assert!(module.is_err());
            Ok(())
        }

        #[coverage_helper::test]
        fn yank_module_only_account_owner() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let rm_module = test_module();

            // add namespaces as the account owner
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: FIRST_TEST_ACCOUNT_ID,
                namespace: rm_module.namespace.to_string(),
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            // first add module as the account owner
            let add_modules_msg = ExecuteMsg::ProposeModules {
                modules: vec![(rm_module.clone(), ModuleReference::App(0))],
            };
            execute_as(&mut deps, &abstr.owner, add_modules_msg)?;
            let added_module = REGISTERED_MODULES.load(&deps.storage, &rm_module)?;
            assert_eq!(added_module, ModuleReference::App(0));

            // then yank the module as the other
            let msg = ExecuteMsg::YankModule { module: rm_module };
            // as other
            let other = deps.api.addr_make(TEST_OTHER);
            let res = execute_as(&mut deps, &other, msg);
            assert_eq!(
                res,
                Err(RegistryError::AccountOwnerMismatch {
                    sender: other,
                    owner: abstr.owner,
                })
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn yank_module() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let rm_module = test_module();

            // add namespaces as the owner
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: FIRST_TEST_ACCOUNT_ID,
                namespace: rm_module.namespace.to_string(),
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            // first add module as the owner
            let add_modules_msg = ExecuteMsg::ProposeModules {
                modules: vec![(rm_module.clone(), ModuleReference::App(0))],
            };
            execute_as(&mut deps, &abstr.owner, add_modules_msg)?;
            let added_module = REGISTERED_MODULES.load(&deps.storage, &rm_module)?;
            assert_eq!(added_module, ModuleReference::App(0));

            // then yank as owner
            let msg = ExecuteMsg::YankModule {
                module: rm_module.clone(),
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            // check that the yanked module is in the yanked modules and no longer in the library
            let module = REGISTERED_MODULES.load(&deps.storage, &rm_module);
            assert!(module.is_err());
            let yanked_module = YANKED_MODULES.load(&deps.storage, &rm_module)?;
            assert_eq!(yanked_module, ModuleReference::App(0));
            Ok(())
        }

        #[coverage_helper::test]
        fn bad_version() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;

            // add namespaces
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: FIRST_TEST_ACCOUNT_ID,
                namespace: "namespace".to_string(),
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            let bad_version_module = ModuleInfo::from_id(
                TEST_MODULE_ID,
                ModuleVersion::Version("non_compliant_version".into()),
            )?;
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(bad_version_module, ModuleReference::App(0))],
            };
            let other = deps.api.addr_make(TEST_OTHER);
            let res = execute_as(&mut deps, &other, msg);
            assert!(res.unwrap_err().to_string().contains("Invalid version"));

            let latest_version_module = ModuleInfo::from_id(TEST_MODULE_ID, ModuleVersion::Latest)?;
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(latest_version_module, ModuleReference::App(0))],
            };
            let res = execute_as(&mut deps, &other, msg);
            assert_eq!(
                res,
                Err(RegistryError::Abstract(AbstractError::Assert(
                    "Module version must be set to a specific version".into(),
                )))
            );
            Ok(())
        }

        #[coverage_helper::test]
        fn abstract_namespace() -> RegistryTestResult {
            let mut deps = registry_mock_deps();
            let abstract_contract_id = format!("{}:{}", ABSTRACT_NAMESPACE, "test-module");

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let new_module = ModuleInfo::from_id(&abstract_contract_id, TEST_VERSION.into())?;

            // let mod_ref = ModuleReference::
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };

            // execute as other
            let other = deps.api.addr_make(TEST_OTHER);
            let res = execute_as(&mut deps, &other, msg.clone());
            assert_eq!(
                res,
                Err(RegistryError::Ownership(OwnershipError::NotOwner {}))
            );

            execute_as(&mut deps, &abstr.owner, msg)?;
            let module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;
            assert_eq!(module, ModuleReference::App(0));
            Ok(())
        }

        #[coverage_helper::test]
        fn validates_module_info() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            mock_init_with_account(&mut deps, false)?;
            let bad_modules = vec![
                ModuleInfo {
                    name: "test-module".to_string(),
                    version: ModuleVersion::Version("0.0.1".to_string()),
                    namespace: Namespace::unchecked(""),
                },
                ModuleInfo {
                    name: "test-module".to_string(),
                    version: ModuleVersion::Version("0.0.1".to_string()),
                    namespace: Namespace::unchecked(""),
                },
                ModuleInfo {
                    name: "".to_string(),
                    version: ModuleVersion::Version("0.0.1".to_string()),
                    namespace: Namespace::unchecked("test"),
                },
                ModuleInfo {
                    name: "test-module".to_string(),
                    version: ModuleVersion::Version("aoeu".to_string()),
                    namespace: Namespace::unchecked(""),
                },
            ];

            for bad_module in bad_modules {
                let msg = ExecuteMsg::ProposeModules {
                    modules: vec![(bad_module.clone(), ModuleReference::App(0))],
                };
                let other = deps.api.addr_make(TEST_OTHER);
                let res = execute_as(&mut deps, &other, msg);
                assert_eq!(
                    res,
                    Err(RegistryError::Abstract(AbstractError::FormattingError {
                        object: "module name".into(),
                        expected: "with content".into(),
                        actual: "empty".into(),
                    }))
                );
            }

            Ok(())
        }

        #[coverage_helper::test]
        fn add_module_monetization() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let mut new_module = test_module();
            new_module.namespace = Namespace::new(ABSTRACT_NAMESPACE)?;
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());
            let _module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;

            let monetization = Monetization::InstallFee(FixedFee::new(&coin(45, "ujuno")));
            let metadata = None;
            let monetization_module_msg = ExecuteMsg::UpdateModuleConfiguration {
                module_name: new_module.name.clone(),
                namespace: new_module.namespace.clone(),
                update_module: UpdateModule::Versioned {
                    version: TEST_VERSION.to_owned(),
                    metadata: None,
                    monetization: Some(monetization.clone()),
                    instantiation_funds: None,
                },
            };
            execute_as(&mut deps, &abstr.owner, monetization_module_msg)?;

            // We query the module to see if the monetization is attached ok
            let query_msg = QueryMsg::Modules {
                infos: vec![new_module.clone()],
            };
            let res = query(deps.as_ref(), mock_env_validated(deps.api), query_msg)?;
            let ser_res = from_json::<ModulesResponse>(&res)?;
            assert_eq!(ser_res.modules.len(), 1);
            assert_eq!(
                ser_res.modules[0],
                ModuleResponse {
                    module: Module {
                        info: new_module,
                        reference: ModuleReference::App(0)
                    },
                    config: ModuleConfiguration::new(monetization, metadata, vec![])
                }
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn add_module_init_funds() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let mut new_module = test_module();
            new_module.namespace = Namespace::new(ABSTRACT_NAMESPACE)?;
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());
            let _module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;

            let instantiation_funds = vec![coin(42, "ujuno"), coin(123, "ujunox")];
            let metadata = None;
            let monetization_module_msg = ExecuteMsg::UpdateModuleConfiguration {
                module_name: new_module.name.clone(),
                namespace: new_module.namespace.clone(),
                update_module: UpdateModule::Versioned {
                    version: TEST_VERSION.to_owned(),
                    metadata: None,
                    monetization: None,
                    instantiation_funds: Some(instantiation_funds.clone()),
                },
            };
            execute_as(&mut deps, &abstr.owner, monetization_module_msg)?;

            // We query the module to see if the monetization is attached ok
            let query_msg = QueryMsg::Modules {
                infos: vec![new_module.clone()],
            };
            let res = query(deps.as_ref(), mock_env_validated(deps.api), query_msg)?;
            let ser_res = from_json::<ModulesResponse>(&res)?;
            assert_eq!(ser_res.modules.len(), 1);
            assert_eq!(
                ser_res.modules[0],
                ModuleResponse {
                    module: Module {
                        info: new_module,
                        reference: ModuleReference::App(0)
                    },
                    config: ModuleConfiguration::new(
                        Monetization::None,
                        metadata,
                        instantiation_funds
                    )
                }
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn add_module_metadata() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            let mut new_module = test_module();
            new_module.namespace = Namespace::new(ABSTRACT_NAMESPACE)?;
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            let res = execute_as(&mut deps, &abstr.owner, msg);
            assert!(res.is_ok());
            let _module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;

            let monetization = Monetization::None;
            let metadata = Some("ipfs://YRUI243876FJHKHV3IY".to_string());
            let metadata_module_msg = ExecuteMsg::UpdateModuleConfiguration {
                module_name: new_module.name.clone(),
                namespace: new_module.namespace.clone(),
                update_module: UpdateModule::Versioned {
                    version: TEST_VERSION.to_owned(),
                    metadata: metadata.clone(),
                    monetization: None,
                    instantiation_funds: None,
                },
            };
            execute_as(&mut deps, &abstr.owner, metadata_module_msg)?;

            // We query the module to see if the monetization is attached ok
            let query_msg = QueryMsg::Modules {
                infos: vec![new_module.clone()],
            };
            let res = query(deps.as_ref(), mock_env_validated(deps.api), query_msg)?;
            let ser_res = from_json::<ModulesResponse>(&res)?;
            assert_eq!(ser_res.modules.len(), 1);
            assert_eq!(
                ser_res.modules[0],
                ModuleResponse {
                    module: Module {
                        info: new_module,
                        reference: ModuleReference::App(0)
                    },
                    config: ModuleConfiguration::new(monetization, metadata, vec![])
                }
            );

            Ok(())
        }
    }

    fn claim_test_namespace_as_owner(deps: &mut MockDeps, owner: &Addr) -> RegistryTestResult {
        let msg = ExecuteMsg::ClaimNamespace {
            account_id: FIRST_TEST_ACCOUNT_ID,
            namespace: TEST_NAMESPACE.to_string(),
        };
        execute_as(deps, owner, msg)?;
        Ok(())
    }

    mod remove_module {
        use super::*;

        #[coverage_helper::test]
        fn test_only_admin() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            claim_test_namespace_as_owner(&mut deps, &abstr.owner)?;

            // add a module as the owner
            let mut new_module = ModuleInfo::from_id(TEST_MODULE_ID, TEST_VERSION.into())?;
            new_module.namespace = Namespace::new(TEST_NAMESPACE)?;
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            // Load the module from the library to check its presence
            assert!(REGISTERED_MODULES.has(&deps.storage, &new_module));

            // now, remove the module as the admin
            let msg = ExecuteMsg::RemoveModule { module: new_module };
            let other = deps.api.addr_make(TEST_OTHER);
            let res = execute_as(&mut deps, &other, msg);

            assert_eq!(
                res,
                Err(RegistryError::Ownership(OwnershipError::NotOwner {}))
            );
            Ok(())
        }

        #[coverage_helper::test]
        fn remove_from_library() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            claim_test_namespace_as_owner(&mut deps, &abstr.owner)?;

            // add a module as the owner
            let new_module = ModuleInfo::from_id(TEST_MODULE_ID, TEST_VERSION.into())?;
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            // Load the module from the library to check its presence
            assert!(REGISTERED_MODULES.has(&deps.storage, &new_module));

            // now, remove the module as the admin
            let msg = ExecuteMsg::RemoveModule {
                module: new_module.clone(),
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            assert!(!REGISTERED_MODULES.has(&deps.storage, &new_module));
            Ok(())
        }

        #[coverage_helper::test]
        fn leaves_pending() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            claim_test_namespace_as_owner(&mut deps, &abstr.owner)?;

            // add a module as the owner
            let new_module = ModuleInfo::from_id(TEST_MODULE_ID, TEST_VERSION.into())?;
            PENDING_MODULES.save(deps.as_mut().storage, &new_module, &ModuleReference::App(0))?;

            // yank the module as the owner
            let msg = ExecuteMsg::RemoveModule {
                module: new_module.clone(),
            };
            let res = execute_as(&mut deps, &abstr.owner, msg);

            assert_eq!(res, Err(RegistryError::ModuleNotFound(new_module)));
            Ok(())
        }

        #[coverage_helper::test]
        fn remove_from_yanked() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;
            claim_test_namespace_as_owner(&mut deps, &abstr.owner)?;

            // add a module as the owner
            let new_module = ModuleInfo::from_id(TEST_MODULE_ID, TEST_VERSION.into())?;
            YANKED_MODULES.save(deps.as_mut().storage, &new_module, &ModuleReference::App(0))?;

            // should be removed from library and added to yanked
            assert!(!REGISTERED_MODULES.has(&deps.storage, &new_module));
            assert!(YANKED_MODULES.has(&deps.storage, &new_module));

            // now, remove the module as the admin
            let msg = ExecuteMsg::RemoveModule {
                module: new_module.clone(),
            };
            execute_as(&mut deps, &abstr.owner, msg)?;

            assert!(!REGISTERED_MODULES.has(&deps.storage, &new_module));
            assert!(!YANKED_MODULES.has(&deps.storage, &new_module));
            Ok(())
        }
    }

    mod register_account {
        use super::*;

        #[coverage_helper::test]
        fn add_account() -> RegistryTestResult {
            let mut deps = registry_mock_deps();

            let other = deps.api.addr_make(TEST_OTHER);
            deps.querier = registry_mock_querier_builder(deps.api)
                .with_contract_version(&other, "some:contract", "0.0.0")
                .build();

            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let msg = ExecuteMsg::AddAccount {
                namespace: None,
                creator: abstr.owner.to_string(),
            };

            let first_acc_addr = deps.api.addr_make(FIRST_ACCOUNT);
            let first_acc = Account::new(first_acc_addr.clone());

            // as non-account
            let res = execute_as(&mut deps, &other, msg.clone());
            assert_eq!(
                res,
                Err(RegistryError::NotAccountInfo {
                    caller_info: ModuleInfo::from_id(
                        "some:contract",
                        ModuleVersion::Version(String::from("0.0.0")),
                    )
                    .unwrap(),
                })
            );

            // as account
            execute_as(&mut deps, &first_acc_addr, msg)?;

            let account = ACCOUNT_ADDRESSES.load(&deps.storage, &FIRST_TEST_ACCOUNT_ID)?;
            assert_eq!(account, first_acc);
            Ok(())
        }
    }

    mod configure {
        use super::*;

        #[coverage_helper::test]
        fn update_admin() -> RegistryTestResult {
            let mut deps = registry_mock_deps();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let other = deps.api.addr_make(TEST_OTHER);
            let transfer_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: other.to_string(),
                expiry: None,
            });

            // as other
            let transfer_res = execute_as(&mut deps, &other, transfer_msg.clone());
            assert_eq!(
                transfer_res,
                Err(RegistryError::Ownership(OwnershipError::NotOwner {}))
            );

            execute_as(&mut deps, &abstr.owner, transfer_msg)?;

            // Then update and accept as the new owner
            let accept_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership);
            let accept_res = execute_as(&mut deps, &other, accept_msg).unwrap();
            assert_eq!(0, accept_res.messages.len());

            assert_eq!(
                cw_ownable::get_ownership(&deps.storage).unwrap().owner,
                Some(other)
            );
            Ok(())
        }
    }

    mod query_account_owner {
        use abstract_sdk::namespaces::OWNERSHIP_STORAGE_KEY;

        use super::*;

        #[coverage_helper::test]
        fn returns_account_owner() -> RegistryTestResult {
            let mut deps = registry_mock_deps();
            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps, false)?;

            let account_owner = query_account_owner(
                &deps.as_ref().querier,
                abstr.account.addr().clone(),
                &ABSTRACT_ACCOUNT_ID,
            )?;

            assert_eq!(account_owner, abstr.owner);
            Ok(())
        }

        #[coverage_helper::test]
        fn no_owner_returns_err() -> RegistryTestResult {
            let mut deps = registry_mock_deps();
            let abstr = AbstractMockAddrs::new(deps.api);
            deps.querier = registry_mock_querier_builder(deps.api)
                .with_contract_item(
                    &abstr.account.addr().clone(),
                    cw_storage_plus::Item::<ownership::Ownership<Addr>>::new(OWNERSHIP_STORAGE_KEY),
                    &ownership::Ownership {
                        owner: ownership::GovernanceDetails::Renounced {},
                        pending_owner: None,
                        pending_expiry: None,
                    },
                )
                .build();

            mock_init_with_account(&mut deps, false)?;

            let account_id = ABSTRACT_ACCOUNT_ID;
            let res = query_account_owner(
                &deps.as_ref().querier,
                abstr.account.addr().clone(),
                &account_id,
            );
            assert_eq!(res, Err(RegistryError::NoAccountOwner { account_id }));
            Ok(())
        }
    }
}
