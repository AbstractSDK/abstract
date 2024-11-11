use abstract_std::{
    account::{self, ExecuteMsgFns as _},
    ans_host::{self, ExecuteMsgFns as _},
    ibc_client::{self, ExecuteMsgFns as _},
    ibc_host::{self, ExecuteMsgFns as _},
    module_factory::{self, ExecuteMsgFns as _},
    objects::{
        gov_type::{GovAction, GovernanceDetails},
        module::ModuleInfo,
        module_reference::ModuleReference,
        ABSTRACT_ACCOUNT_ID,
    },
    registry::{self, ExecuteMsgFns as _, QueryMsgFns},
    ACCOUNT,
};
use cosmwasm_std::{to_json_binary, CosmosMsg, WasmMsg};
use cw_orch::{contract::Contract, prelude::*};
use cw_plus_orch::{
    cw3_flex_multisig::{self, Cw3FlexMultisig, ExecuteMsgInterfaceFns},
    cw4_group::{self, Cw4Group},
};
use prost::{Message, Name};

use crate::{Abstract, AbstractInterfaceError, AccountI, AnsHost, ModuleFactory, Registry};

pub const CW3_ABSTRACT: &str = "cw3:abstract";
pub const CW4_ABSTRACT: &str = "cw4:abstract";

#[derive(Clone)]
pub struct AbstractMultisig<Chain: CwEnv> {
    pub cw3: Cw3FlexMultisig<Chain>,
    pub cw4: Cw4Group<Chain>,
}

impl<Chain: CwEnv> AbstractMultisig<Chain> {
    pub fn new(chain: &Chain) -> Self {
        let cw3 = Cw3FlexMultisig::new(CW3_ABSTRACT, chain.clone());
        let cw4 = Cw4Group::new(CW4_ABSTRACT, chain.clone());
        Self { cw3, cw4 }
    }

    pub fn upload_if_needed(&self) -> Result<(), crate::AbstractInterfaceError> {
        self.cw3.upload_if_needed()?;
        self.cw4.upload_if_needed()?;
        Ok(())
    }

    // List of members
    pub fn instantiate(&self, admin: String, members: Vec<cw4::Member>) -> Result<(), CwOrchError> {
        let contract_admin = Addr::unchecked(admin.clone());
        let resp = self.cw4.instantiate(
            &cw4_group::InstantiateMsg {
                admin: Some(admin),
                members,
            },
            Some(&contract_admin),
            &[],
        )?;
        let cw4_address = resp.instantiated_contract_address()?;

        self.cw3.instantiate(
            &cw3_flex_multisig::InstantiateMsg {
                group_addr: cw4_address.to_string(),
                threshold: cw_utils::Threshold::AbsolutePercentage {
                    percentage: cosmwasm_std::Decimal::from_ratio(51_u128, 100_u128),
                },
                max_voting_period: cw_utils::WEEK,
                executor: None,
                proposal_deposit: None,
            },
            Some(&contract_admin),
            &[],
        )?;

        Ok(())
    }

    pub fn propose_on_registry_msgs(
        &self,
        registry: &Registry<Chain>,
        msgs: Vec<registry::ExecuteMsg>,
    ) -> Result<Vec<CosmosMsg>, AbstractInterfaceError> {
        let registry = registry.addr_str()?;
        msgs.into_iter()
            .map(|msg| {
                Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: registry.clone(),
                    msg: to_json_binary(&msg)?,
                    funds: vec![],
                }))
            })
            .collect()
    }

    pub fn propose_on_ans_msgs(
        &self,
        ans: &AnsHost<Chain>,
        msgs: Vec<ans_host::ExecuteMsg>,
    ) -> Result<Vec<CosmosMsg>, AbstractInterfaceError> {
        let ans_host = ans.addr_str()?;
        msgs.into_iter()
            .map(|msg| {
                Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ans_host.clone(),
                    msg: to_json_binary(&msg)?,
                    funds: vec![],
                }))
            })
            .collect()
    }

    pub fn propose_on_ibc_client_msgs(
        &self,
        ibc_client: &ModuleFactory<Chain>,
        msgs: Vec<ibc_client::ExecuteMsg>,
    ) -> Result<Vec<CosmosMsg>, AbstractInterfaceError> {
        let ibc_client = ibc_client.addr_str()?;
        msgs.into_iter()
            .map(|msg| {
                Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ibc_client.clone(),
                    msg: to_json_binary(&msg)?,
                    funds: vec![],
                }))
            })
            .collect()
    }

    pub fn propose_on_ibc_host_msgs(
        &self,
        ibc_host: &ModuleFactory<Chain>,
        msgs: Vec<ibc_host::ExecuteMsg>,
    ) -> Result<Vec<CosmosMsg>, AbstractInterfaceError> {
        let ibc_host = ibc_host.addr_str()?;
        msgs.into_iter()
            .map(|msg| {
                Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ibc_host.clone(),
                    msg: to_json_binary(&msg)?,
                    funds: vec![],
                }))
            })
            .collect()
    }

    /// Create messages for new modules
    pub fn propose_modules_msgs(
        &self,
        registry: &Registry<Chain>,
        modules: Vec<(ModuleInfo, ModuleReference)>,
    ) -> Result<Vec<CosmosMsg>, AbstractInterfaceError> {
        let mut msgs = vec![registry::ExecuteMsg::ProposeModules {
            modules: modules.clone(),
        }];
        let registry_config = registry.config()?;
        if !registry_config.security_disabled {
            msgs.push(registry::ExecuteMsg::ApproveOrRejectModules {
                approves: modules.into_iter().map(|(info, _reference)| info).collect(),
                rejects: vec![],
            });
        }
        self.propose_on_registry_msgs(registry, msgs)
    }
}

#[cfg(feature = "daemon")]
impl<T: CwEnv + Stargate> Abstract<T> {
    pub fn update_admin_to_multisig(
        &mut self,
        admin: String,
        members: Vec<cw4::Member>,
        proposal_creator: &T::Sender,
        extra_contracts: impl IntoIterator<Item = Contract<T>>,
    ) -> Result<(), AbstractInterfaceError> {
        self.multisig.cw3.set_sender(proposal_creator);
        self.multisig.cw4.set_sender(proposal_creator);

        let chain = self.registry.environment().clone();

        let proposal_creator_addr = self.multisig.cw3.environment().sender_addr();

        assert!(
            members
                .iter()
                .any(|member| member.addr == proposal_creator_addr.as_str()),
            "proposal_creator must be in members list"
        );

        self.multisig.upload_if_needed()?;
        self.multisig.instantiate(admin, members)?;

        let cw3_flex_address = self.multisig.cw3.address()?;

        let contract_admin_upgrades = self
            .contracts()
            .into_iter()
            .map(|(contract, _version)| contract.clone())
            .chain(extra_contracts)
            .map(|contract| prost_types::Any {
                value: cosmrs::proto::cosmwasm::wasm::v1::MsgUpdateAdmin {
                    sender: chain.sender_addr().to_string(),
                    new_admin: cw3_flex_address.to_string(),
                    contract: contract.address().unwrap().to_string(),
                }
                .encode_to_vec(),
                type_url: cosmrs::proto::cosmwasm::wasm::v1::MsgUpdateAdmin::type_url(),
            })
            .collect::<Vec<_>>();
        chain
            .commit_any(contract_admin_upgrades, None)
            .map_err(Into::into)?;
        log::info!("Updated migrate admin of abstract contracts");

        let mut msgs = vec![];
        // Transfer ownership
        let cw_ownable_transfer_msg = cw_ownable::Action::TransferOwnership {
            new_owner: cw3_flex_address.to_string(),
            expiry: None,
        };

        // Registry
        self.registry
            .update_ownership(cw_ownable_transfer_msg.clone())?;
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.registry.addr_str()?,
            msg: to_json_binary(&registry::ExecuteMsg::UpdateOwnership(
                cw_ownable::Action::AcceptOwnership,
            ))?,
            funds: vec![],
        }));

        // Ans host
        self.ans_host
            .update_ownership(cw_ownable_transfer_msg.clone())?;
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.ans_host.addr_str()?,
            msg: to_json_binary(&ans_host::ExecuteMsg::UpdateOwnership(
                cw_ownable::Action::AcceptOwnership,
            ))?,
            funds: vec![],
        }));

        // Module factory
        self.module_factory
            .update_ownership(cw_ownable_transfer_msg.clone())?;
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.module_factory.addr_str()?,
            msg: to_json_binary(&module_factory::ExecuteMsg::UpdateOwnership(
                cw_ownable::Action::AcceptOwnership,
            ))?,
            funds: vec![],
        }));

        // IBC Client
        self.ibc
            .client
            .update_ownership(cw_ownable_transfer_msg.clone())?;
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.ibc.client.addr_str()?,
            msg: to_json_binary(&ibc_client::ExecuteMsg::UpdateOwnership(
                cw_ownable::Action::AcceptOwnership,
            ))?,
            funds: vec![],
        }));

        // IBC Host
        self.ibc
            .host
            .update_ownership(cw_ownable_transfer_msg.clone())?;
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.ibc.host.addr_str()?,
            msg: to_json_binary(&ibc_host::ExecuteMsg::UpdateOwnership(
                cw_ownable::Action::AcceptOwnership,
            ))?,
            funds: vec![],
        }));

        // Accept new contracts owners
        let title = "Accept ownership of abstract contracts as multisig".to_owned();
        let description = "We should accept ownership of abstract contracts as multisig so our root account is not managed by a single wallet".to_owned();
        self.multisig
            .cw3
            .propose(description, msgs, title, None, &[])?;
        log::info!("Created proposal to update ownerships of abstract contracts");

        // Move ownership of the account
        let root_account = AccountI::load_from(&self, ABSTRACT_ACCOUNT_ID)?;
        root_account.update_ownership(GovAction::TransferOwnership {
            new_owner: GovernanceDetails::External {
                governance_address: cw3_flex_address.to_string(),
                governance_type: "cw3-flex".to_owned(),
            },
            expiry: None,
        })?;

        // Accept new account owner
        let title = "Accept ownership of abstract account".to_owned();
        let description = "We should accept ownership of abstract account as multisig so our root account is not managed by a single wallet".to_owned();
        self.multisig.cw3.propose(
            description,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: root_account.addr_str()?,
                msg: to_json_binary(&<account::ExecuteMsg>::UpdateOwnership(
                    GovAction::AcceptOwnership,
                ))?,
                funds: vec![],
            })],
            title,
            None,
            &[],
        )?;
        log::info!("Created proposal to update abstract root account governance");

        Ok(())
    }

    /// Create proposal for migration of the deployment based on version changes. If the registered contracts have the right version, we don't propose migration
    pub fn propose_migrate_if_version_changed(
        &self,
    ) -> Result<bool, crate::AbstractInterfaceError> {
        let mut has_uploaded = false;
        let mut msgs: Vec<CosmosMsg> = vec![];
        let mut natives_to_register = vec![];

        if ::module_factory::contract::CONTRACT_VERSION
            != crate::migrate::contract_version(&self.module_factory)?.version
        {
            self.module_factory.upload_if_needed()?;
            let new_code_id = self.module_factory.code_id()?;
            has_uploaded = true;
            natives_to_register.push((
                self.module_factory.as_instance(),
                ::module_factory::contract::CONTRACT_VERSION.to_string(),
            ));
            msgs.push(CosmosMsg::Wasm(WasmMsg::Migrate {
                contract_addr: self.module_factory.addr_str()?,
                new_code_id,
                msg: to_json_binary(&module_factory::MigrateMsg::Migrate {})?,
            }));
        }

        if ::registry::contract::CONTRACT_VERSION
            != crate::migrate::contract_version(&self.registry)?.version
        {
            self.registry.upload_if_needed()?;
            let new_code_id = self.registry.code_id()?;
            has_uploaded = true;
            natives_to_register.push((
                self.registry.as_instance(),
                ::registry::contract::CONTRACT_VERSION.to_string(),
            ));
            msgs.push(CosmosMsg::Wasm(WasmMsg::Migrate {
                contract_addr: self.registry.addr_str()?,
                new_code_id,
                msg: to_json_binary(&registry::MigrateMsg::Migrate {})?,
            }));
        }

        if ::ans_host::contract::CONTRACT_VERSION
            != crate::migrate::contract_version(&self.ans_host)?.version
        {
            self.ans_host.upload_if_needed()?;
            let new_code_id = self.ans_host.code_id()?;
            has_uploaded = true;
            natives_to_register.push((
                self.ans_host.as_instance(),
                ::ans_host::contract::CONTRACT_VERSION.to_string(),
            ));
            msgs.push(CosmosMsg::Wasm(WasmMsg::Migrate {
                contract_addr: self.ans_host.addr_str()?,
                new_code_id,
                msg: to_json_binary(&ans_host::MigrateMsg::Migrate {})?,
            }));
        }

        // TODO: reimplement desired logic here after #531 merged
        if ::ibc_client::contract::CONTRACT_VERSION
            != crate::migrate::contract_version(&self.ibc.client)?.version
        {
            self.ibc.client.upload_if_needed()?;
            let new_code_id = self.ibc.client.code_id()?;
            has_uploaded = true;
            natives_to_register.push((
                self.ibc.client.as_instance(),
                ::ibc_client::contract::CONTRACT_VERSION.to_string(),
            ));
            msgs.push(CosmosMsg::Wasm(WasmMsg::Migrate {
                contract_addr: self.ibc.client.addr_str()?,
                new_code_id,
                msg: to_json_binary(&ibc_client::MigrateMsg {})?,
            }));
        }
        if ::ibc_host::contract::CONTRACT_VERSION
            != crate::migrate::contract_version(&self.ibc.host)?.version
        {
            self.ibc.host.upload_if_needed()?;
            let new_code_id = self.ibc.host.code_id()?;
            has_uploaded = true;
            natives_to_register.push((
                self.ibc.host.as_instance(),
                ::ibc_host::contract::CONTRACT_VERSION.to_string(),
            ));
            msgs.push(CosmosMsg::Wasm(WasmMsg::Migrate {
                contract_addr: self.ibc.host.addr_str()?,
                new_code_id,
                msg: to_json_binary(&ibc_host::MigrateMsg {})?,
            }));
        }

        let mut modules_to_register = self
            .registry
            .contracts_into_module_entries(natives_to_register, |c| {
                ModuleReference::Native(c.address().unwrap())
            })?;

        // We need to check the version in registry for the account contract
        let account = self.registry.module(ModuleInfo::from_id_latest(ACCOUNT)?)?;

        if ::account::contract::CONTRACT_VERSION != account.info.version.to_string()
            && self.account.upload_if_needed()?.is_some()
        {
            modules_to_register.push((
                ModuleInfo::from_id(ACCOUNT, ::account::contract::CONTRACT_VERSION.parse()?)?,
                ModuleReference::Account(self.account.code_id()?),
            ));

            has_uploaded = true
        }

        let title = "Migrate native contracts of the abstract".to_owned();
        let description = "We should upgrade abstract contracts to a new versions".to_owned();
        self.multisig
            .cw3
            .propose(description, msgs, title, None, &[])?;

        let propose_modules_msgs = self
            .multisig
            .propose_modules_msgs(&self.registry, modules_to_register)?;

        let title = "Register abstract native modules in the abstract".to_owned();
        let description =
            "We should register upgraded modules in the abstract contracts under a new version"
                .to_owned();
        self.multisig
            .cw3
            .propose(description, propose_modules_msgs, title, None, &[])?;
        Ok(has_uploaded)
    }
}
