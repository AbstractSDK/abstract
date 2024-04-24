//! # AuthZ
//! This module provides functionality to interact with the authz module of CosmosSDK Chains.
//! It allows for granting authorizations to perform actions on behalf of an account to other accounts.

use cosmos_sdk_proto::{
    cosmos::{
        authz,
        bank::v1beta1::MsgSend,
        distribution::v1beta1::{MsgSetWithdrawAddress, MsgWithdrawDelegatorReward},
        gov::v1beta1::{MsgVote, MsgVoteWeighted, WeightedVoteOption},
        staking::v1beta1::{MsgBeginRedelegate, MsgDelegate, MsgUndelegate},
    },
    cosmwasm::wasm::v1::{
        MsgClearAdmin, MsgExecuteContract, MsgInstantiateContract, MsgInstantiateContract2,
        MsgMigrateContract, MsgUpdateAdmin,
    },
    ibc::{applications::transfer::v1::MsgTransfer, core::client::v1::Height},
    traits::{Message, Name},
};
use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg, Timestamp, WasmMsg};
use prost_types::Any;

use super::stargate::{
    authz::{
        AuthZAuthorization, AuthorizationType, GenericAuthorization, Policy, SendAuthorization,
        StakeAuthorization,
    },
    convert_coin, convert_coins,
    gov::vote_to_option,
};

use crate::{features::AccountIdentification, AbstractSdkResult};
/// An interface to the CosmosSDK AuthZ module which allows for granting authorizations to perform actions on behalf of one account to other accounts.
pub trait AuthZInterface: AccountIdentification {
    /// API for accessing the Cosmos SDK AuthZ module.
    /// The **granter** is the address of the user **granting** an authorization to perform an action on their behalf.
    /// By default, it is the proxy address of the Account.

    /// ```
    /// use abstract_sdk::prelude::*;
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use abstract_sdk::mock_module::MockModule;
    /// # let module = MockModule::new();
    /// # let deps = mock_dependencies();

    /// let authz: AuthZ = module.auth_z(deps.as_ref(), None)?;
    /// ```
    fn auth_z<'a>(
        &'a self,
        deps: cosmwasm_std::Deps<'a>,
        granter: Option<Addr>,
    ) -> AbstractSdkResult<AuthZ> {
        let granter = granter.unwrap_or(self.proxy_address(deps)?);
        Ok(AuthZ { granter })
    }
}

impl<T> AuthZInterface for T where T: AccountIdentification {}

/// This struct provides methods to grant message authorizations and interact with the authz module.
///
/// # Example
/// ```
/// use abstract_sdk::prelude::*;
/// # use cosmwasm_std::testing::mock_dependencies;
/// # use abstract_sdk::mock_module::MockModule;
/// # let module = MockModule::new();
///
/// let authz: Authz  = module.auth_z(deps.as_ref(), None)?;
/// ```
/// */
#[derive(Clone)]
pub struct AuthZ {
    granter: Addr,
}

impl AuthZ {
    /// Retrieve the granter's address.
    /// By default, this is the proxy address of the Account.
    fn granter(&self) -> Addr {
        self.granter.clone()
    }

    /// Removes msg type authorization from the granter to the **grantee**.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `type_url` - The msg type url to revoke authorization.
    pub fn revoke(&self, grantee: &Addr, type_url: String) -> CosmosMsg {
        let msg = authz::v1beta1::MsgRevoke {
            granter: self.granter().to_string(),
            grantee: grantee.to_string(),
            msg_type_url: type_url,
        }
        .encode_to_vec();

        CosmosMsg::Stargate {
            // TODO: `Name` implementation is missing for authz
            // type_url: authz::v1beta1::MsgRevoke::type_url(),
            type_url: "/cosmos.authz.v1beta1.MsgRevoke".to_string(),
            value: Binary(msg),
        }
    }

    /// Generate cosmwasm message for the AuthZAuthorization type
    pub fn grant_authorization<A: AuthZAuthorization>(
        &self,
        grantee: &Addr,
        expiration: Option<Timestamp>,
        authorization: A,
    ) -> CosmosMsg {
        let msg = authz::v1beta1::MsgGrant {
            granter: self.granter().to_string(),
            grantee: grantee.to_string(),
            grant: Some(authorization.grant(expiration)),
        }
        .encode_to_vec();

        CosmosMsg::Stargate {
            type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
            value: Binary(msg),
        }
    }

    /// Grants generic authorization to a **grantee**.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `msg` - Allowed message type url. These are protobuf URLs defined in the Cosmos SDK.
    /// * `expiration` - The expiration timestamp of the grant.
    pub fn grant_generic(
        &self,
        grantee: &Addr,
        msg_type_url: String,
        expiration: Option<Timestamp>,
    ) -> CosmosMsg {
        let generic = GenericAuthorization::new(msg_type_url);

        self.grant_authorization(grantee, expiration, generic)
    }

    /// Grants send authorization to a **grantee**.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `spend_limits` - The maximum amount the grantee can spend.
    /// * `expiration` - The expiration timestamp of the grant.
    pub fn grant_send(
        &self,
        grantee: &Addr,
        spend_limit: Vec<Coin>,
        expiration: Option<Timestamp>,
    ) -> CosmosMsg {
        let send = SendAuthorization::new(spend_limit);

        self.grant_authorization(grantee, expiration, send)
    }

    /// Grants stake authorization to a **grantee**.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `max_tokens` - The maximum amount the grantee can stake. Empty means any amount of coins can be delegated.
    /// * `authorization_type` - The allowed delegate type.
    /// * `validators` - The list of validators to allow or deny.
    /// * `expiration` - The expiration timestamp of the grant.
    pub fn grant_stake(
        &self,
        grantee: &Addr,
        max_tokens: Option<Coin>,
        authorization_type: AuthorizationType,
        validators: Option<Policy>,
        expiration: Option<Timestamp>,
    ) -> CosmosMsg {
        let stake = StakeAuthorization::new(max_tokens, authorization_type, validators);

        self.grant_authorization(grantee, expiration, stake)
    }

    /// Executes a Cosmos message using authz
    ///
    /// # Arguments
    ///
    /// * `grantee` -   The address of the grantee.
    /// * `msg` -       Message that you want to send using authz
    /// When a sender is necessary in the resulting message, the granter is used
    pub fn execute(&self, grantee: &Addr, msg: impl Into<CosmosMsg>) -> CosmosMsg {
        let msg = msg.into();
        let (type_url, value) = match msg {
            CosmosMsg::Wasm(wasm_msg) => match wasm_msg {
                WasmMsg::Execute {
                    contract_addr,
                    msg,
                    funds,
                } => (
                    MsgExecuteContract::type_url(),
                    MsgExecuteContract {
                        sender: self.granter.to_string(),
                        contract: contract_addr,
                        msg: msg.into(),
                        funds: convert_coins(funds),
                    }
                    .encode_to_vec(),
                ),
                WasmMsg::Instantiate {
                    admin,
                    code_id,
                    msg,
                    funds,
                    label,
                } => (
                    MsgInstantiateContract::type_url(),
                    MsgInstantiateContract {
                        sender: self.granter.to_string(),
                        msg: msg.into(),
                        funds: convert_coins(funds),
                        admin: admin.unwrap_or("".to_string()),
                        code_id,
                        label,
                    }
                    .encode_to_vec(),
                ),
                WasmMsg::Instantiate2 {
                    admin,
                    code_id,
                    label,
                    msg,
                    funds,
                    salt,
                } => (
                    "/cosmwasm.wasm.v1.MsgInstantiateContract2".to_string(),
                    MsgInstantiateContract2 {
                        sender: self.granter.to_string(),
                        msg: msg.into(),
                        funds: convert_coins(funds),
                        admin: admin.unwrap_or("".to_string()),
                        code_id,
                        label,
                        salt: salt.to_vec(),
                        fix_msg: false,
                    }
                    .encode_to_vec(),
                ),
                WasmMsg::Migrate {
                    contract_addr,
                    new_code_id,
                    msg,
                } => (
                    MsgMigrateContract::type_url(),
                    MsgMigrateContract {
                        sender: self.granter.to_string(),
                        contract: contract_addr,
                        msg: msg.into(),
                        code_id: new_code_id,
                    }
                    .encode_to_vec(),
                ),
                WasmMsg::UpdateAdmin {
                    contract_addr,
                    admin,
                } => (
                    MsgUpdateAdmin::type_url(),
                    MsgUpdateAdmin {
                        sender: self.granter.to_string(),
                        contract: contract_addr,
                        new_admin: admin,
                    }
                    .encode_to_vec(),
                ),
                WasmMsg::ClearAdmin { contract_addr } => (
                    MsgClearAdmin::type_url(),
                    MsgClearAdmin {
                        sender: self.granter.to_string(),
                        contract: contract_addr,
                    }
                    .encode_to_vec(),
                ),
                _ => todo!(),
            },
            CosmosMsg::Stargate { type_url, value } => (type_url.clone(), value.into()),
            CosmosMsg::Bank(bank_msg) => match bank_msg {
                cosmwasm_std::BankMsg::Send { to_address, amount } => (
                    MsgSend::type_url(),
                    MsgSend {
                        from_address: self.granter.to_string(),
                        to_address,
                        amount: convert_coins(amount),
                    }
                    .encode_to_vec(),
                ),
                // There is no SDK message associated with this msg
                cosmwasm_std::BankMsg::Burn { amount: _ } => {
                    unimplemented!("Can't use authz with the authz api")
                }
                _ => todo!(),
            },
            CosmosMsg::Custom(_) => unimplemented!(
                "The authz api doesn't support custom messages. Use Stargate messages instead"
            ),
            CosmosMsg::Staking(staking_msg) => match staking_msg {
                cosmwasm_std::StakingMsg::Delegate { validator, amount } => (
                    MsgDelegate::type_url(),
                    MsgDelegate {
                        delegator_address: self.granter.to_string(),
                        validator_address: validator,
                        amount: Some(convert_coin(amount)),
                    }
                    .encode_to_vec(),
                ),
                cosmwasm_std::StakingMsg::Undelegate { validator, amount } => (
                    MsgUndelegate::type_url(),
                    MsgUndelegate {
                        delegator_address: self.granter.to_string(),
                        validator_address: validator,
                        amount: Some(convert_coin(amount)),
                    }
                    .encode_to_vec(),
                ),
                cosmwasm_std::StakingMsg::Redelegate {
                    src_validator,
                    dst_validator,
                    amount,
                } => (
                    MsgBeginRedelegate::type_url(),
                    MsgBeginRedelegate {
                        delegator_address: self.granter.to_string(),
                        amount: Some(convert_coin(amount)),
                        validator_src_address: src_validator.to_string(),
                        validator_dst_address: dst_validator.to_string(),
                    }
                    .encode_to_vec(),
                ),
                _ => todo!(),
            },
            CosmosMsg::Distribution(distribution_msg) => match distribution_msg {
                cosmwasm_std::DistributionMsg::SetWithdrawAddress { address } => (
                    MsgSetWithdrawAddress::type_url(),
                    MsgSetWithdrawAddress {
                        delegator_address: self.granter.to_string(),
                        withdraw_address: address.to_string(),
                    }
                    .encode_to_vec(),
                ),
                cosmwasm_std::DistributionMsg::WithdrawDelegatorReward { validator } => (
                    MsgWithdrawDelegatorReward::type_url(),
                    MsgWithdrawDelegatorReward {
                        delegator_address: self.granter.to_string(),
                        validator_address: validator,
                    }
                    .encode_to_vec(),
                ),
                // cosmwasm_std::DistributionMsg::FundCommunityPool { amount } => (
                //     MsgFundCommunityPool::type_url(),
                //     MsgFundCommunityPool {
                //         depositor: self.granter.to_string(),
                //         amount: convert_coins(amount),
                //     }
                //     .encode_to_vec(),
                // ),
                _ => todo!(),
            },
            CosmosMsg::Ibc(ibc_msg) => {
                match ibc_msg {
                    cosmwasm_std::IbcMsg::Transfer {
                        channel_id,
                        to_address,
                        amount,
                        timeout,
                    } => (
                        MsgTransfer::type_url(),
                        MsgTransfer{
                            source_port: "transfer".to_string(),
                            source_channel: channel_id,
                            token: Some(convert_coin(amount)),
                            sender: self.granter.to_string(),
                            receiver: to_address,
                            timeout_height: timeout.block().map(|b| Height{
                                revision_number: b.revision,
                                revision_height: b.height,
                            }),
                            timeout_timestamp: timeout.timestamp().map(|t| t.nanos()).unwrap_or_default()
                        }.encode_to_vec()
                    ),
                    // This is there because there is a priori no port associated with the sender
                    _=> unimplemented!("Abstract doesn't support IBC messages via authz. Abstract handles IBC requests natively")
                }
            }
            CosmosMsg::Gov(gov_msg) => match gov_msg {
                cosmwasm_std::GovMsg::Vote { proposal_id, vote } => (
                    "/cosmos.gov.v1beta1.MsgVote".to_string(),
                    MsgVote {
                        proposal_id,
                        voter: self.granter.to_string(),
                        option: vote_to_option(vote),
                    }
                    .encode_to_vec(),
                ),
                cosmwasm_std::GovMsg::VoteWeighted {
                    proposal_id,
                    options,
                } => (
                    "/cosmos.gov.v1beta1.MsgVoteWeighted".to_string(),
                    MsgVoteWeighted {
                        proposal_id,
                        voter: self.granter.to_string(),
                        options: options
                            .into_iter()
                            .map(|o| WeightedVoteOption {
                                option: vote_to_option(o.option.clone()),
                                weight: o.weight.to_string(),
                            })
                            .collect(),
                    }
                    .encode_to_vec(),
                ),
            },
            _ => todo!(),
        };
        self.execute_raw(grantee, type_url, value.into())
    }

    /// Executes a message using authz
    ///
    /// # Arguments
    ///
    /// * `msg_type_url` - Type url of the message that has to be sent using authz
    /// * `msg_value` - Proto encoded message value that has to be sent using authz
    /// * `grantee` - The address of the authz grantee. (This is the address that is actually sending the message)
    pub fn execute_raw(
        &self,
        grantee: &Addr,

        msg_type_url: String,
        msg_value: Binary,
    ) -> CosmosMsg {
        let msg = authz::v1beta1::MsgExec {
            grantee: grantee.to_string(),
            msgs: vec![Any {
                type_url: msg_type_url,
                value: msg_value.to_vec(),
            }],
        }
        .encode_to_vec();

        CosmosMsg::Stargate {
            // TODO: `Name` implementation is missing for authz
            // type_url: authz::v1beta1::MsgExec::type_url(),
            type_url: "/cosmos.authz.v1beta1.MsgExec".to_string(),
            value: Binary(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{apis::stargate::convert_stamp, mock_module::*};
    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn generic_authorization() {
        let app = MockModule::new();
        let deps = mock_dependencies();

        let granter = Addr::unchecked("granter");
        let grantee = Addr::unchecked("grantee");

        let auth_z = app.auth_z(deps.as_ref(), Some(granter.clone())).unwrap();
        let expiration = Some(Timestamp::from_seconds(10));

        let generic_authorization_msg = auth_z.grant_generic(
            &grantee,
            "/cosmos.gov.v1beta1.MsgVote".to_string(),
            expiration,
        );

        let expected_msg = CosmosMsg::Stargate {
            type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
            value: Binary(
                authz::v1beta1::MsgGrant {
                    granter: granter.into_string(),
                    grantee: grantee.into_string(),
                    grant: Some(authz::v1beta1::Grant {
                        authorization: Some(Any {
                            type_url: "/cosmos.authz.v1beta1.GenericAuthorization".to_string(),
                            value: authz::v1beta1::GenericAuthorization {
                                msg: "/cosmos.gov.v1beta1.MsgVote".to_string(),
                            }
                            .encode_to_vec(),
                        }),
                        expiration: expiration.map(convert_stamp),
                    }),
                }
                .encode_to_vec(),
            ),
        };

        assert_eq!(generic_authorization_msg, expected_msg);
    }

    #[test]
    fn revoke_authorization() {
        let app = MockModule::new();
        let deps = mock_dependencies();

        let granter = Addr::unchecked("granter");
        let grantee = Addr::unchecked("grantee");

        let auth_z = app.auth_z(deps.as_ref(), Some(granter.clone())).unwrap();

        let generic_authorization_msg =
            auth_z.revoke(&grantee, "/cosmos.gov.v1beta1.MsgVote".to_string());

        let expected_msg = CosmosMsg::Stargate {
            type_url: "/cosmos.authz.v1beta1.MsgRevoke".to_string(),
            value: Binary(
                authz::v1beta1::MsgRevoke {
                    granter: granter.into_string(),
                    grantee: grantee.into_string(),
                    msg_type_url: "/cosmos.gov.v1beta1.MsgVote".to_string(),
                }
                .encode_to_vec(),
            ),
        };

        assert_eq!(generic_authorization_msg, expected_msg);
    }
}
