//! # TokenFactory API
//! Interacts with the osmosis tokenfactory module
//!

use cosmos_sdk_proto::traits::Message;
use cosmwasm_std::{Addr, Binary, CosmosMsg, Deps, StdError};
use osmosis_std::types::cosmos::bank::v1beta1::Metadata;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{
    MsgBurn, MsgChangeAdmin, MsgCreateDenom, MsgForceTransfer, MsgMint, MsgSetBeforeSendHook,
    MsgSetDenomMetadata,
};
use std::num::NonZeroU128;

use crate::features::AccountIdentification;
use crate::AbstractSdkResult;

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
    ) -> AbstractSdkResult<TokenFactory> {
        let sender = sender.unwrap_or(self.proxy_address(deps)?);
        // Check that the subdenom is valid
        let subdenom = subdenom.into();
        if !subdenom
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '/')
        {
            return Err(StdError::generic_err(
                "Invalid character found. Only alphanumeric characters, '.' and '/' are allowed.",
            )
            .into());
        }
        Ok(TokenFactory { subdenom, sender })
    }
}

impl<T> TokenFactoryInterface for T where T: AccountIdentification {}

/**
API for accessing the Osmosis TokenFactory module.

# Example
```rust
use abstract_sdk::prelude::*;
# use cosmwasm_std::testing::mock_dependencies;
# use abstract_sdk::mock_module::MockModule;
# let module = MockModule::new();
# let deps = mock_dependencies();

let token_factory: TokenFactory  = module.token_factory(deps.as_ref(), "uusd".to_string(), None)?;
```
 */
#[derive(Clone)]
pub struct TokenFactory {
    subdenom: String,
    sender: Addr,
}

impl TokenFactory {
    /// Retrieves the sender's address as a string.
    fn sender(&self) -> Addr {
        self.sender.clone()
    }

    /// Retrieves the actual denom of the asset
    pub fn denom(&self) -> String {
        ["factory", self.sender().as_str(), self.subdenom.as_str()].join("/")
    }
    ///  Create denom
    /// ```
    /// # use cosmwasm_std::{ReplyOn, Response};
    /// use abstract_sdk::prelude::*;
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use abstract_sdk::mock_module::MockModule;
    /// # let module = MockModule::new();
    /// # let deps = mock_dependencies();
    /// const CREATE_DENOM_REPLY_ID: u64 = 1;
    /// let token_factory: TokenFactory<MockModule> = module.token_factory(deps.as_ref(), "denom", None)?;
    /// let denom_msg = token_factory.create_denom();
    /// let denom_msg = module.executor(deps.as_ref()).execute_with_reply(vec![denom_msg.into()], ReplyOn::Always, CREATE_DENOM_REPLY_ID)?;
    ///
    ///  let response = Response::new().add_submessage(denom_msg);
    /// ```
    pub fn create_denom(&self) -> CosmosMsg {
        let msg = MsgCreateDenom {
            sender: self.sender().to_string(),
            subdenom: self.subdenom.to_string(),
        };

        CosmosMsg::Stargate {
            type_url: MsgCreateDenom::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        }
    }

    /// Mint tokens
    /// MsgMint is the sdk.Msg type for minting new tokens into existence.
    pub fn mint(&self, amount: NonZeroU128, mint_to_address: &Addr) -> CosmosMsg {
        let msg = MsgMint {
            sender: self.sender().to_string(),
            amount: Some(self.build_coin(amount)),
            mint_to_address: mint_to_address.to_string(),
        };

        CosmosMsg::Stargate {
            type_url: MsgMint::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        }
    }

    /// Burn tokens
    /// MsgBurn is the sdk.Msg type for allowing an admin account to burn a token.
    /// For now, we only support burning from the sender account.
    pub fn burn(&self, amount: NonZeroU128, burn_from_address: &Addr) -> CosmosMsg {
        let msg = MsgBurn {
            sender: self.sender().to_string(),
            amount: Some(self.build_coin(amount)),
            burn_from_address: burn_from_address.to_string(),
        };

        CosmosMsg::Stargate {
            type_url: MsgBurn::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        }
    }

    /// Change admin
    /// MsgChangeAdmin is the sdk.Msg type for allowing an admin account to reassign
    /// adminship of a denom to a new account.
    pub fn change_admin(&self, new_admin: &Addr) -> CosmosMsg {
        let msg = MsgChangeAdmin {
            sender: self.sender().to_string(),
            denom: self.denom().to_string(),
            new_admin: new_admin.to_string(),
        };

        CosmosMsg::Stargate {
            type_url: MsgChangeAdmin::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        }
    }

    /// Set denom metadata
    /// MsgSetDenomMetadata is the sdk.Msg type for allowing an admin account to set
    /// the denom's bank metadata.
    /// If the metadata is empty, it will be deleted.
    pub fn set_denom_metadata(&self, metadata: Option<Metadata>) -> CosmosMsg {
        let msg = MsgSetDenomMetadata {
            sender: self.sender().to_string(),
            metadata,
        };

        CosmosMsg::Stargate {
            type_url: MsgSetDenomMetadata::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        }
    }

    /// Force transfer tokens
    /// MsgForceTransfer is the sdk.Msg type for allowing an admin account to forcibly transfer tokens from one account to another.
    pub fn force_transfer(
        &self,
        amount: NonZeroU128,
        from_address: &Addr,
        recipient: &Addr,
    ) -> CosmosMsg {
        let msg = MsgForceTransfer {
            sender: self.sender().to_string(),
            amount: Some(self.build_coin(amount)),
            transfer_from_address: from_address.to_string(),
            transfer_to_address: recipient.to_string(),
        };

        CosmosMsg::Stargate {
            type_url: MsgForceTransfer::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        }
    }

    /// Set the token factory before send hook.
    /// TODO: this is not yet possible on the chain
    pub fn set_before_send_hook(&self, cosmwasm_address: Addr) -> CosmosMsg {
        let msg = MsgSetBeforeSendHook {
            sender: self.sender().to_string(),
            denom: self.denom().to_string(),
            cosmwasm_address: cosmwasm_address.to_string(),
        };

        CosmosMsg::Stargate {
            type_url: MsgSetBeforeSendHook::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        }
    }

    /// Build the osmosis coin
    fn build_coin(&self, amount: NonZeroU128) -> osmosis_std::types::cosmos::base::v1beta1::Coin {
        osmosis_std::types::cosmos::base::v1beta1::Coin {
            denom: self.denom().clone(),
            amount: amount.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;
    const MOCK_DENOM: &str = "factory/proxy_address/denom";

    /// Asserts that the provided CosmosMsg::Stargate has the expected type_url and value
    /// If the CosmosMsg is not a Stargate, this function will panic
    /// TODO: This should be moved to abstract-testing
    pub fn assert_stargate_message(
        msg: cosmwasm_std::CosmosMsg,
        expected_type_url: &str,
        expected_value: Binary,
    ) {
        match msg {
            cosmwasm_std::CosmosMsg::Stargate { type_url, value } => {
                speculoos::assert_that!(type_url).is_equal_to(expected_type_url.to_string());
                speculoos::assert_that!(value).is_equal_to(expected_value);
            }
            _ => panic!("Unexpected message type"),
        }
    }

    mod create_denom {
        use super::*;

        use abstract_testing::addresses::TEST_PROXY;
        use cosmos_sdk_proto::traits::Message;

        #[test]
        fn create_denom() {
            let module = MockModule::new();
            let deps = mock_dependencies();
            let token_factory: TokenFactory = module
                .token_factory(deps.as_ref(), "denom".to_string(), None)
                .unwrap();
            let create_denom_msg = token_factory.create_denom();
            let expected_msg_create_denom = MsgCreateDenom {
                sender: TEST_PROXY.to_string(),
                subdenom: "denom".to_string(),
            };

            assert_stargate_message(
                create_denom_msg,
                "/osmosis.tokenfactory.v1beta1.MsgCreateDenom",
                Binary(expected_msg_create_denom.encode_to_vec()),
            )
        }
    }
    mod mint {
        use super::*;
        use abstract_testing::prelude::TEST_PROXY;
        use cosmos_sdk_proto::traits::Message;

        #[test]
        fn happy_mint() {
            let module = MockModule::new();
            let deps = mock_dependencies();
            let token_factory: TokenFactory = module
                .token_factory(deps.as_ref(), "denom".to_string(), None)
                .unwrap();

            let mint_msg = token_factory.mint(
                NonZeroU128::new(100u128).unwrap(),
                &Addr::unchecked("mint_to_address"),
            );

            let expected_msg_mint = MsgMint {
                sender: TEST_PROXY.to_string(),
                amount: Some(osmosis_std::types::cosmos::base::v1beta1::Coin {
                    denom: MOCK_DENOM.to_string(),
                    amount: "100".to_string(),
                }),
                mint_to_address: "mint_to_address".to_string(),
            };

            assert_stargate_message(
                mint_msg,
                "/osmosis.tokenfactory.v1beta1.MsgMint",
                Binary(expected_msg_mint.encode_to_vec()),
            );
        }
    }
}
