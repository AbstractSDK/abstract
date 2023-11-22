//! # TokenFactory API
//! Interacts with the osmosis tokenfactory module
//!

use cosmos_sdk_proto::traits::Message;
use cosmwasm_std::{Addr, Binary, CosmosMsg};
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{
    MsgBurn, MsgChangeAdmin, MsgCreateDenom, MsgForceTransfer, MsgMint, MsgSetBeforeSendHook,
    MsgSetDenomMetadata,
};
use std::num::NonZeroU128;

use crate::{
    apis::stargate::token_factory::{Metadata, TokenFactoryCommand},
    TokenFactoryResult,
};

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
pub struct OsmosisTokenFactory {
    pub(crate) subdenom: String,
    pub(crate) sender: Addr,
}

impl OsmosisTokenFactory {
    /// Build the osmosis coin
    fn build_coin(&self, amount: NonZeroU128) -> osmosis_std::types::cosmos::base::v1beta1::Coin {
        osmosis_std::types::cosmos::base::v1beta1::Coin {
            denom: self.denom().clone(),
            amount: amount.to_string(),
        }
    }
}

impl TokenFactoryCommand for OsmosisTokenFactory {
    /// Retrieves the sender's address as a string.
    fn sender(&self) -> Addr {
        self.sender.clone()
    }

    /// Retrieves the actual denom of the asset
    fn denom(&self) -> String {
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
    fn create_denom(&self) -> TokenFactoryResult<CosmosMsg> {
        let msg = MsgCreateDenom {
            sender: self.sender().to_string(),
            subdenom: self.subdenom.to_string(),
        };

        Ok(CosmosMsg::Stargate {
            type_url: MsgCreateDenom::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        })
    }

    /// Mint tokens
    /// MsgMint is the sdk.Msg type for minting new tokens into existence.
    fn mint(&self, amount: NonZeroU128, mint_to_address: &Addr) -> TokenFactoryResult<CosmosMsg> {
        let msg = MsgMint {
            sender: self.sender().to_string(),
            amount: Some(self.build_coin(amount)),
            mint_to_address: mint_to_address.to_string(),
        };

        Ok(CosmosMsg::Stargate {
            type_url: MsgMint::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        })
    }

    /// Burn tokens
    /// MsgBurn is the sdk.Msg type for allowing an admin account to burn a token.
    /// For now, we only support burning from the sender account.
    fn burn(&self, amount: NonZeroU128, burn_from_address: &Addr) -> TokenFactoryResult<CosmosMsg> {
        let msg = MsgBurn {
            sender: self.sender().to_string(),
            amount: Some(self.build_coin(amount)),
            burn_from_address: burn_from_address.to_string(),
        };

        Ok(CosmosMsg::Stargate {
            type_url: MsgBurn::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        })
    }

    /// Change admin
    /// MsgChangeAdmin is the sdk.Msg type for allowing an admin account to reassign
    /// adminship of a denom to a new account.
    fn change_admin(&self, new_admin: &Addr) -> TokenFactoryResult<CosmosMsg> {
        let msg = MsgChangeAdmin {
            sender: self.sender().to_string(),
            denom: self.denom().to_string(),
            new_admin: new_admin.to_string(),
        };

        Ok(CosmosMsg::Stargate {
            type_url: MsgChangeAdmin::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        })
    }

    /// Set denom metadata
    /// MsgSetDenomMetadata is the sdk.Msg type for allowing an admin account to set
    /// the denom's bank metadata.
    /// If the metadata is empty, it will be deleted.
    fn set_denom_metadata(&self, metadata: Option<Metadata>) -> TokenFactoryResult<CosmosMsg> {
        let msg = MsgSetDenomMetadata {
            sender: self.sender().to_string(),
            metadata: metadata.map(Into::into),
        };

        Ok(CosmosMsg::Stargate {
            type_url: MsgSetDenomMetadata::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        })
    }

    /// Force transfer tokens
    /// MsgForceTransfer is the sdk.Msg type for allowing an admin account to forcibly transfer tokens from one account to another.
    fn force_transfer(
        &self,
        amount: NonZeroU128,
        from_address: &Addr,
        recipient: &Addr,
    ) -> TokenFactoryResult<CosmosMsg> {
        let msg = MsgForceTransfer {
            sender: self.sender().to_string(),
            amount: Some(self.build_coin(amount)),
            transfer_from_address: from_address.to_string(),
            transfer_to_address: recipient.to_string(),
        };

        Ok(CosmosMsg::Stargate {
            type_url: MsgForceTransfer::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        })
    }

    /// Set the token factory before send hook.
    /// TODO: this is not yet possible on osmosis
    fn set_before_send_hook(&self, cosmwasm_address: &Addr) -> TokenFactoryResult<CosmosMsg> {
        let msg = MsgSetBeforeSendHook {
            sender: self.sender().to_string(),
            denom: self.denom().to_string(),
            cosmwasm_address: cosmwasm_address.to_string(),
        };

        Ok(CosmosMsg::Stargate {
            type_url: MsgSetBeforeSendHook::TYPE_URL.to_owned(),
            value: Binary(msg.encode_to_vec()),
        })
    }
}

impl From<Metadata> for osmosis_std::types::cosmos::bank::v1beta1::Metadata {
    fn from(value: Metadata) -> Self {
        Self {
            description: value.description,
            denom_units: value
                .denom_units
                .into_iter()
                .map(
                    |unit| osmosis_std::types::cosmos::bank::v1beta1::DenomUnit {
                        denom: unit.denom,
                        exponent: unit.exponent,
                        aliases: unit.aliases,
                    },
                )
                .collect(),
            base: value.base,
            display: value.display,
            name: value.name,
            symbol: value.symbol,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;
    use cosmwasm_std::testing::*;
    const MOCK_DENOM: &str = "factory/proxy_address/denom";

    mod create_denom {
        use crate::apis::stargate::token_factory::{TokenFactoryInterface, OSMOSIS_TOKEN_FACTORY};

        use super::*;

        use abstract_testing::addresses::TEST_PROXY;
        use cosmos_sdk_proto::traits::Message;

        #[test]
        fn create_denom() {
            let module = MockModule::new();
            let deps = mock_dependencies();
            let token_factory = module
                .token_factory(
                    deps.as_ref(),
                    "denom".to_string(),
                    None,
                    OSMOSIS_TOKEN_FACTORY,
                )
                .unwrap();
            let create_denom_msg = token_factory.create_denom().unwrap();
            let expected_msg_create_denom = MsgCreateDenom {
                sender: TEST_PROXY.to_string(),
                subdenom: "denom".to_string(),
            };

            assert_eq!(
                create_denom_msg,
                CosmosMsg::Stargate {
                    type_url: "/osmosis.tokenfactory.v1beta1.MsgCreateDenom".to_owned(),
                    value: Binary(expected_msg_create_denom.encode_to_vec())
                }
            );
        }
    }
    mod mint {
        use crate::apis::stargate::token_factory::{TokenFactoryInterface, OSMOSIS_TOKEN_FACTORY};

        use super::*;
        use abstract_testing::prelude::TEST_PROXY;
        use cosmos_sdk_proto::traits::Message;

        #[test]
        fn happy_mint() {
            let module = MockModule::new();
            let deps = mock_dependencies();
            let token_factory = module
                .token_factory(
                    deps.as_ref(),
                    "denom".to_string(),
                    None,
                    OSMOSIS_TOKEN_FACTORY,
                )
                .unwrap();

            let mint_msg = token_factory
                .mint(
                    NonZeroU128::new(100u128).unwrap(),
                    &Addr::unchecked("mint_to_address"),
                )
                .unwrap();

            let expected_msg_mint = MsgMint {
                sender: TEST_PROXY.to_string(),
                amount: Some(osmosis_std::types::cosmos::base::v1beta1::Coin {
                    denom: MOCK_DENOM.to_string(),
                    amount: "100".to_string(),
                }),
                mint_to_address: "mint_to_address".to_string(),
            };

            assert_eq!(
                mint_msg,
                CosmosMsg::Stargate {
                    type_url: "/osmosis.tokenfactory.v1beta1.MsgMint".to_owned(),
                    value: Binary(expected_msg_mint.encode_to_vec())
                }
            );
        }
    }
}
