use std::num::NonZeroU128;

use cosmwasm_std::{Addr, CosmosMsg, Deps};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{apis::token_factory::osmosis::OsmosisTokenFactory, features::AccountIdentification};

/// Osmosis chain
pub const OSMOSIS_TOKEN_FACTORY: &'static str = "OSMOSIS";

/// Error type for the abstract token factory API.
#[derive(Error, Debug, PartialEq)]
pub enum TokenFactoryError {
    /// Failed parsing during resolving subdenom for the coin
    #[error(
        "Invalid character({0}) found. Only alphanumeric characters, '.' and '/' are allowed."
    )]
    SubDenomParseError(char),

    /// Not known token factory
    #[error("Token factory {0} is not a known token factory")]
    UnknownTokenFactory(String),

    /// Failed to load proxy address
    #[error("Failed to get proxy address as sender")]
    ProxyNotFound {},
}

// TODO: do we want to use those types for anything?

// #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
// pub struct CreateDenomMsg {}

// #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
// pub struct ChangeAdminMsg {
//     pub new_admin_address: String,
// }

// #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
// pub struct MintTokensMsg {
//     pub amount: Uint128,
//     pub mint_to_address: String,
// }

// #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
// pub struct BurnTokensMsg {
//     pub amount: Uint128,
//     pub burn_from_address: String,
// }

// #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
// pub struct ForceTransferMsg {
//     pub amount: Uint128,
//     pub from_address: String,
//     pub to_address: String,
// }

// #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
// pub struct SetMetadataMsg {
//     pub metadata: Metadata,
// }

/// Osmosis token representation, for descriptions:
/// @see [cosmos_sdk_proto::cosmos::bank::v1beta1::Metadata]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub struct Metadata {
    /// @see [cosmos_sdk_proto::cosmos::bank::v1beta1::Metadata::description]
    pub description: String,
    /// @see [cosmos_sdk_proto::cosmos::bank::v1beta1::Metadata::denom_units]
    pub denom_units: Vec<DenomUnit>,
    /// @see [cosmos_sdk_proto::cosmos::bank::v1beta1::Metadata::base]
    pub base: String,
    /// @see [cosmos_sdk_proto::cosmos::bank::v1beta1::Metadata::display]
    pub display: String,
    /// @see [cosmos_sdk_proto::cosmos::bank::v1beta1::Metadata::name]
    pub name: String,
    /// @see [cosmos_sdk_proto::cosmos::bank::v1beta1::Metadata::symbol]
    pub symbol: String,
}

/// @see [cosmos_sdk_proto::cosmos::bank::v1beta1::DenomUnit]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub struct DenomUnit {
    /// @see [cosmos_sdk_proto::cosmos::bank::v1beta1::DenomUnit::denom]
    pub denom: String,
    /// @see [cosmos_sdk_proto::cosmos::bank::v1beta1::DenomUnit::exponent]
    pub exponent: u32,
    /// @see [cosmos_sdk_proto::cosmos::bank::v1beta1::DenomUnit::aliases]
    pub aliases: Vec<String>,
}

/// An interface to the CosmosSDK FeeTokenFactory module which allows for granting fee expenditure rights.
pub trait TokenFactoryInterface: AccountIdentification {
    /**
    API for accessing Osmosis' TokenFactory module.
    To leverage this api, you must retrieve the TokenFactory by passing in the subdenom.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # let module = MockModule::new();
    # let deps = mock_dependencies();

    let token_factory: TokenFactory = module.token_factory(deps.as_ref(), "uusd", None)?;
    ```
     */
    fn token_factory<'a>(
        &'a self,
        deps: Deps<'a>,
        subdenom: impl Into<String>,
        sender: Option<Addr>,
        chain: &str,
    ) -> Result<Box<dyn TokenFactoryCommand>, TokenFactoryError> {
        let sender = sender.unwrap_or(
            self.proxy_address(deps)
                .map_err(|_| TokenFactoryError::ProxyNotFound{})?,
        );
        // Check that the subdenom is valid
        let subdenom: String = subdenom.into();
        if let Some(invalid_char) = subdenom
            .chars()
            .find(|c| !c.is_ascii_alphanumeric() && *c != '.' && *c != '/')
        {
            return Err(TokenFactoryError::SubDenomParseError(invalid_char));
        }
        resolve_token_factory(subdenom, sender, chain)
    }
}

impl<T> TokenFactoryInterface for T where T: AccountIdentification {}

/// Token factory commands
pub trait TokenFactoryCommand {
    /// Retrieves the sender's address as a string.
    fn sender(&self) -> Addr;

    /// Retrieves the actual denom of the asset
    fn denom(&self) -> String;
    ///  Create denom
    /// ```
    /// # use cosmwasm_std::{ReplyOn, Response};
    /// use abstract_sdk::prelude::*;
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use abstract_sdk::mock_module::MockModule;
    /// # let module = MockModule::new();
    /// # let deps = mock_dependencies();
    /// const CREATE_DENOM_REPLY_ID: u64 = 1;
    /// let token_factory = module.token_factory(deps.as_ref(), "denom", None, "osmosis")?;
    /// let denom_msg = token_factory.create_denom();
    /// let denom_msg = module.executor(deps.as_ref()).execute_with_reply(vec![denom_msg.into()], ReplyOn::Always, CREATE_DENOM_REPLY_ID)?;
    ///
    ///  let response = Response::new().add_submessage(denom_msg);
    /// ```
    fn create_denom(&self) -> CosmosMsg;

    /// Mint tokens
    /// MsgMint is the sdk.Msg type for minting new tokens into existence.
    fn mint(&self, amount: NonZeroU128, mint_to_address: &Addr) -> CosmosMsg;

    /// Burn tokens
    /// MsgBurn is the sdk.Msg type for allowing an admin account to burn a token.
    /// For now, we only support burning from the sender account.
    fn burn(&self, amount: NonZeroU128, burn_from_address: &Addr) -> CosmosMsg;

    /// Change admin
    /// MsgChangeAdmin is the sdk.Msg type for allowing an admin account to reassign
    /// adminship of a denom to a new account.
    fn change_admin(&self, new_admin: &Addr) -> CosmosMsg;

    /// Set denom metadata
    /// MsgSetDenomMetadata is the sdk.Msg type for allowing an admin account to set
    /// the denom's bank metadata.
    /// If the metadata is empty, it will be deleted.
    fn set_denom_metadata(&self, metadata: Option<Metadata>) -> CosmosMsg;

    /// Force transfer tokens
    /// MsgForceTransfer is the sdk.Msg type for allowing an admin account to forcibly transfer tokens from one account to another.
    fn force_transfer(
        &self,
        amount: NonZeroU128,
        from_address: &Addr,
        recipient: &Addr,
    ) -> CosmosMsg;

    /// Set the token factory before send hook.
    /// TODO: this is not yet possible on any chain
    fn set_before_send_hook(&self, cosmwasm_address: &Addr) -> CosmosMsg;
}

pub(crate) fn resolve_token_factory(
    subdenom: String,
    sender: Addr,
    value: &str,
) -> Result<Box<dyn TokenFactoryCommand>, TokenFactoryError> {
    match value {
        OSMOSIS_TOKEN_FACTORY => Ok(Box::new(OsmosisTokenFactory { subdenom, sender })),
        _ => Err(TokenFactoryError::UnknownTokenFactory(value.to_owned())),
    }
}
