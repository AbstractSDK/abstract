//! # TokenFactory API
//! Interacts with the osmosis tokenfactory module
//!

use cosmos_sdk_proto::traits::Message;
use cosmwasm_std::{Addr, Deps, Uint128};
use osmosis_std::types::cosmos::bank::v1beta1::Metadata;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{
    MsgBurn, MsgChangeAdmin, MsgCreateDenom, MsgForceTransfer, MsgMint, MsgSetDenomMetadata, MsgSetBeforeSendHook,
};
use std::cell::RefCell;

use abstract_core::AbstractError;

use crate::cw_helpers::stargate_msg;
use crate::features::AccountIdentification;
use crate::AbstractSdkResult;
use crate::AccountAction;

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

    let token_factory: TokenFactory<MockModule> = module.token_factory(deps.as_ref(), "uusd".to_string());
    ```
     */
    fn token_factory<'a>(
        &'a self,
        deps: Deps<'a>,
        subdenom: impl ToString,
        sender: Option<Addr>,
    ) -> TokenFactory<Self> {
        TokenFactory {
            base: self,
            deps,
            subdenom: subdenom.to_string(),
            sender: sender.into(),
        }
    }
}

impl<T> TokenFactoryInterface for T where T: AccountIdentification {}

/**
API for accessing the Osmosis TokenFactory module.

# Example
```
use abstract_sdk::prelude::*;
# use cosmwasm_std::testing::mock_dependencies;
# use abstract_sdk::mock_module::MockModule;
# let module = MockModule::new();

let token_factory: TokenFactory  = module.grant();
```
 */
#[derive(Clone)]
pub struct TokenFactory<'a, T: TokenFactoryInterface> {
    base: &'a T,
    deps: Deps<'a>,
    subdenom: String,
    // We use a RefCell here to allow for the sender to be set after the TokenFactory is created
    sender: RefCell<Option<Addr>>,
}

impl<'a, T: TokenFactoryInterface> TokenFactory<'a, T> {
    /// Retrieves the sender's address as a string.
    fn sender(&self) -> AbstractSdkResult<String> {
        // If the sender was already set or overridden, return it
        if let Some(sender) = &*self.sender.borrow() {
            return Ok(sender.to_string());
        }

        let sender = self.base.proxy_address(self.deps)?;
        *self.sender.borrow_mut() = Some(sender.clone());
        Ok(sender.to_string())
    }

    ///  Retrieves the actual denom of the asset
    pub fn denom(&self) -> AbstractSdkResult<String> {
        Ok(vec!["factory", self.sender()?.as_str(), self.subdenom.as_str()].join("/"))
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
    /// let token_factory: TokenFactory<MockModule> = module.token_factory(deps.as_ref(), "denom".to_string());
    /// let denom_msg = token_factory.create_denom()?;
    /// let denom_msg = module.executor(deps.as_ref()).execute_with_reply(vec![denom_msg], ReplyOn::Always, CREATE_DENOM_REPLY_ID)?;
    ///
    ///  let response = Response::new().add_submessage(denom_msg);
    /// ```
    pub fn create_denom(&self) -> AbstractSdkResult<AccountAction> {
        let msg = MsgCreateDenom {
            sender: self.sender()?,
            subdenom: self.subdenom.to_string(),
        }
        .encode_to_vec();

        let msg = stargate_msg("/osmosis.tokenfactory.v1beta1.MsgCreateDenom", &msg)?;

        Ok(msg.into())
    }

    /// Mint tokens
    /// MsgMint is the sdk.Msg type for minting new tokens into existence.
    pub fn mint(
        &self,
        amount: Uint128,
        mint_to_address: &Addr,
    ) -> AbstractSdkResult<AccountAction> {
        // don't allow minting of 0 coins
        if amount.is_zero() {
            return Err(AbstractError::Assert("Cannot mint 0 coins".to_string()).into());
        }

        let msg = MsgMint {
            sender: self.sender()?,
            amount: Some(self.build_coin(amount)?),
            mint_to_address: mint_to_address.to_string(),
        }
        .encode_to_vec();

        let msg = stargate_msg("/osmosis.tokenfactory.v1beta1.MsgMint", &msg)?;

        Ok(msg.into())
    }

    /// Burn tokens
    /// MsgBurn is the sdk.Msg type for allowing an admin account to burn a token.
    /// For now, we only support burning from the sender account.
    pub fn burn(
        &self,
        amount: Uint128,
        burn_from_address: &Addr,
    ) -> AbstractSdkResult<AccountAction> {
        // don't allow burning of 0 coins
        if amount.is_zero() {
            return Err(AbstractError::Assert("Cannot burn 0 coins".to_string()).into());
        }

        let msg = MsgBurn {
            sender: self.sender()?,
            amount: Some(self.build_coin(amount)?),
            burn_from_address: burn_from_address.to_string(),
        }
        .encode_to_vec();

        let msg = stargate_msg("/osmosis.tokenfactory.v1beta1.MsgBurn", &msg)?;

        Ok(msg.into())
    }

    /// Change admin
    /// MsgChangeAdmin is the sdk.Msg type for allowing an admin account to reassign
    /// adminship of a denom to a new account.
    pub fn change_admin(&self, new_admin: &Addr) -> AbstractSdkResult<AccountAction> {
        let msg = MsgChangeAdmin {
            sender: self.sender()?,
            denom: self.denom()?.to_string(),
            new_admin: new_admin.to_string(),
        }
        .encode_to_vec();

        let msg = stargate_msg("/osmosis.tokenfactory.v1beta1.MsgChangeAdmin", &msg)?;

        Ok(msg.into())
    }

    /// Set denom metadata
    /// MsgSetDenomMetadata is the sdk.Msg type for allowing an admin account to set
    /// the denom's bank metadata.
    /// If the metadata is empty, it will be deleted.
    pub fn set_denom_metadata(
        &self,
        metadata: Option<Metadata>,
    ) -> AbstractSdkResult<AccountAction> {
        let msg = MsgSetDenomMetadata {
            sender: self.sender()?,
            metadata,
        }
        .encode_to_vec();

        let msg = stargate_msg("/osmosis.tokenfactory.v1beta1.MsgSetDenomMetadata", &msg)?;

        Ok(msg.into())
    }

    /// Force transfer tokens
    /// MsgForceTransfer is the sdk.Msg type for allowing an admin account to forcibly transfer tokens from one account to another.
    pub fn force_transfer(
        &self,
        amount: Uint128,
        recipient: &Addr,
    ) -> AbstractSdkResult<AccountAction> {
        let msg = MsgForceTransfer {
            sender: self.sender()?,
            amount: Some(self.build_coin(amount)?),
            // We send from the proxy address
            transfer_from_address: self.sender()?.to_string(),
            transfer_to_address: recipient.to_string(),
        }
        .encode_to_vec();

        let msg = stargate_msg("/osmosis.tokenfactory.v1beta1.MsgForceTransfer", &msg)?;

        Ok(msg.into())
    }

    pub fn set_before_send_hook(&self, cosmwasm_address: Addr) -> AbstractSdkResult<AccountAction> {
        let msg = MsgSetBeforeSendHook {
            sender: self.sender()?,
            denom: self.denom()?,
            cosmwasm_address: cosmwasm_address.to_string(),
        }
        .encode_to_vec();

        let msg = stargate_msg("/osmosis.tokenfactory.v1beta1.MsgSetBeforeSendHook", &msg)?;

        Ok(msg.into())
    }

    /// Build the osmosis coin
    fn build_coin(
        &self,
        amount: Uint128,
    ) -> AbstractSdkResult<osmosis_std::types::cosmos::base::v1beta1::Coin> {
        Ok(osmosis_std::types::cosmos::base::v1beta1::Coin {
            denom: self.denom()?.clone(),
            amount: amount.to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;
    const MOCK_SUBDENOM: &str = "denom";
    const MOCK_DENOM: &str = "factory/proxy_address/denom";

    /// Asserts that the provided CosmosMsg::Stargate has the expected type_url and value
    /// If the CosmosMsg is not a Stargate, this function will panic
    /// TODO: This should be moved to abstract-testing
    pub fn assert_stargate_message<T: cosmos_sdk_proto::traits::Message + Default>(
        msg: cosmwasm_std::CosmosMsg,
        expected_type_url: &str,
        expected_value: &T,
    ) {
        match msg {
            cosmwasm_std::CosmosMsg::Stargate { type_url, value } => {
                speculoos::assert_that!(type_url).is_equal_to(expected_type_url.to_string());
                speculoos::assert_that!(value)
                    .is_equal_to(cosmwasm_std::to_binary(&expected_value.encode_to_vec()).unwrap());
            }
            _ => panic!("Unexpected message type"),
        }
    }

    mod mint {
        use super::*;
        use abstract_testing::prelude::{TEST_ADMIN, TEST_PROXY};
        use cosmwasm_std::{to_binary, CosmosMsg};

        #[test]
        fn happy_mint() {
            let module = MockModule::new();
            let deps = mock_dependencies();
            let token_factory: TokenFactory<MockModule> =
                module.token_factory(deps.as_ref(), "denom".to_string(), None);

            let mint_msg = token_factory
                .mint(100u128.into(), &Addr::unchecked("mint_to_address"))
                .unwrap();

            assert_that!(mint_msg.messages()).has_length(1);
            let msg = mint_msg.messages().swap_remove(0);

            let expected_msg_mint = MsgMint {
                sender: TEST_PROXY.to_string(),
                amount: Some(osmosis_std::types::cosmos::base::v1beta1::Coin {
                    denom: MOCK_DENOM.to_string(),
                    amount: "100".to_string(),
                }),
                mint_to_address: "mint_to_address".to_string(),
            };

            assert_stargate_message(
                msg,
                "/osmosis.tokenfactory.v1beta1.MsgMint",
                &expected_msg_mint,
            );
        }

        #[test]
        fn mint_zero() {
            let module = MockModule::new();
            let deps = mock_dependencies();
            let token_factory: TokenFactory<MockModule> =
                module.token_factory(deps.as_ref(), "denom".to_string(), None);

            let mint_msg = token_factory.mint(0u128.into(), &Addr::unchecked("mint_to_address"));

            assert!(mint_msg.is_err());
        }
    }
}
