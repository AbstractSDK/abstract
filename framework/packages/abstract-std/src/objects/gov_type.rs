//! # Governance structure object

use crate::{account::state::ACCOUNT_ID, native_addrs, registry};
use cosmwasm_std::{Addr, Deps, QuerierWrapper};
use cw_address_like::AddressLike;
use cw_utils::Expiration;

use crate::AbstractError;

use super::ownership::cw721;

const MIN_GOV_TYPE_LENGTH: usize = 4;
const MAX_GOV_TYPE_LENGTH: usize = 64;

/// Governance types
#[cosmwasm_schema::cw_serde]
#[derive(Eq)]
#[non_exhaustive]
pub enum GovernanceDetails<T: AddressLike> {
    /// A single address is admin
    Monarchy {
        /// The monarch's address
        monarch: T,
    },
    /// Used when the account is a sub-account of another account.
    SubAccount {
        // Account address
        account: T,
    },
    /// An external governance source. This could be a cw3 contract for instance
    /// The External Governance source still leverages one address that is admin of the contract
    External {
        /// The external contract address
        governance_address: T,
        /// Governance type used for doing extra off-chain queries depending on the type.
        governance_type: String,
    },
    /// This account is linked to an NFT collection.
    /// The owner of the specified token_id is the owner of the account
    NFT {
        collection_addr: T,
        token_id: String,
    },
    /// Abstract account.
    /// Admin actions have to be sent through signature bit flag
    ///
    /// More details: https://github.com/burnt-labs/abstract-account/blob/2c933a7b2a8dacc0ae5bf4344159a7d4ab080135/README.md
    AbstractAccount {
        /// Address of this abstract account
        address: Addr,
    },
    /// Renounced account
    /// This account no longer has an owner and cannot be used.
    Renounced {},
}

/// Actions that can be taken to alter the contract's governance ownership
#[cosmwasm_schema::cw_serde]
pub enum GovAction {
    /// Propose to transfer the contract's ownership to another account,
    /// optionally with an expiry time.
    ///
    /// Can only be called by the contract's current owner.
    ///
    /// Any existing pending ownership transfer is overwritten.
    TransferOwnership {
        new_owner: GovernanceDetails<String>,
        expiry: Option<Expiration>,
    },

    /// Accept the pending ownership transfer.
    ///
    /// Can only be called by the pending owner.
    AcceptOwnership,

    /// Give up the contract's ownership and the possibility of appointing
    /// a new owner.
    ///
    /// Can only be invoked by the contract's current owner.
    ///
    /// Any existing pending ownership transfer is canceled.
    RenounceOwnership,
}

impl GovernanceDetails<String> {
    /// Verify the governance details and convert to `Self<Addr>`
    pub fn verify(self, deps: Deps) -> Result<GovernanceDetails<Addr>, AbstractError> {
        match self {
            GovernanceDetails::Monarchy { monarch } => {
                let addr = deps.api.addr_validate(&monarch)?;
                Ok(GovernanceDetails::Monarchy { monarch: addr })
            }
            GovernanceDetails::SubAccount { account } => {
                let account_addr = deps.api.addr_validate(&account)?;

                let abstract_code_id = native_addrs::abstract_code_id(&deps.querier, account)?;
                let registry_address = native_addrs::registry_address(deps, abstract_code_id)?;
                let registry_address = deps.api.addr_humanize(&registry_address)?;

                let account_id = ACCOUNT_ID.query(&deps.querier, account_addr.clone())?;
                let base = registry::state::ACCOUNT_ADDRESSES.query(
                    &deps.querier,
                    registry_address,
                    &account_id,
                )?;
                let Some(b) = base else {
                    return Err(AbstractError::Std(cosmwasm_std::StdError::generic_err(
                        format!(
                            "Version control does not have account id of account {account_addr}"
                        ),
                    )));
                };
                if b.addr() == account_addr {
                    Ok(GovernanceDetails::SubAccount {
                        account: account_addr,
                    })
                } else {
                    Err(AbstractError::Std(cosmwasm_std::StdError::generic_err(
                        "Verification of sub-account failed, account has different account ids",
                    )))
                }
            }
            GovernanceDetails::External {
                governance_address,
                governance_type,
            } => {
                let addr = deps.api.addr_validate(&governance_address)?;

                if governance_type.len() < MIN_GOV_TYPE_LENGTH {
                    return Err(AbstractError::FormattingError {
                        object: "governance type".into(),
                        expected: "at least 3 characters".into(),
                        actual: governance_type.len().to_string(),
                    });
                }
                if governance_type.len() > MAX_GOV_TYPE_LENGTH {
                    return Err(AbstractError::FormattingError {
                        object: "governance type".into(),
                        expected: "at most 64 characters".into(),
                        actual: governance_type.len().to_string(),
                    });
                }
                if governance_type.contains(|c: char| !c.is_ascii_alphanumeric() && c != '-') {
                    return Err(AbstractError::FormattingError {
                        object: "governance type".into(),
                        expected: "alphanumeric characters and hyphens".into(),
                        actual: governance_type,
                    });
                }

                if governance_type != governance_type.to_lowercase() {
                    return Err(AbstractError::FormattingError {
                        object: "governance type".into(),
                        expected: governance_type.to_ascii_lowercase(),
                        actual: governance_type,
                    });
                }

                Ok(GovernanceDetails::External {
                    governance_address: addr,
                    governance_type,
                })
            }
            GovernanceDetails::Renounced {} => Ok(GovernanceDetails::Renounced {}),
            GovernanceDetails::NFT {
                collection_addr,
                token_id,
            } => Ok(GovernanceDetails::NFT {
                collection_addr: deps.api.addr_validate(&collection_addr.to_string())?,
                token_id,
            }),
            GovernanceDetails::AbstractAccount { address } => {
                Ok(GovernanceDetails::AbstractAccount { address })
            }
        }
    }
}

impl GovernanceDetails<Addr> {
    /// Get the owner address from the governance details
    pub fn owner_address(&self, querier: &QuerierWrapper) -> Option<Addr> {
        match self {
            GovernanceDetails::Monarchy { monarch } => Some(monarch.clone()),
            GovernanceDetails::SubAccount { account } => Some(account.clone()),
            GovernanceDetails::External {
                governance_address, ..
            } => Some(governance_address.clone()),
            GovernanceDetails::Renounced {} => None,
            GovernanceDetails::NFT {
                collection_addr,
                token_id,
            } => {
                let res: Option<cw721::OwnerOfResponse> = querier
                    .query_wasm_smart(
                        collection_addr,
                        &cw721::Cw721QueryMsg::OwnerOf {
                            token_id: token_id.to_string(),
                            include_expired: None,
                        },
                    )
                    .ok();
                res.map(|owner_response| Addr::unchecked(owner_response.owner))
            }
            GovernanceDetails::AbstractAccount { address } => Some(address.to_owned()),
        }
    }
}

impl From<GovernanceDetails<Addr>> for GovernanceDetails<String> {
    fn from(value: GovernanceDetails<Addr>) -> Self {
        match value {
            GovernanceDetails::Monarchy { monarch } => GovernanceDetails::Monarchy {
                monarch: monarch.into_string(),
            },
            GovernanceDetails::SubAccount { account } => GovernanceDetails::SubAccount {
                account: account.into_string(),
            },
            GovernanceDetails::External {
                governance_address,
                governance_type,
            } => GovernanceDetails::External {
                governance_address: governance_address.into_string(),
                governance_type,
            },
            GovernanceDetails::Renounced {} => GovernanceDetails::Renounced {},
            GovernanceDetails::NFT {
                collection_addr,
                token_id,
            } => GovernanceDetails::NFT {
                collection_addr: collection_addr.to_string(),
                token_id,
            },
            GovernanceDetails::AbstractAccount { address } => {
                GovernanceDetails::AbstractAccount { address }
            }
        }
    }
}

impl<T: AddressLike> std::fmt::Display for GovernanceDetails<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            GovernanceDetails::Monarchy { .. } => "monarch",
            GovernanceDetails::SubAccount { .. } => "sub-account",
            GovernanceDetails::External {
                governance_type, ..
            } => governance_type.as_str(),
            GovernanceDetails::Renounced {} => "renounced",
            GovernanceDetails::NFT { .. } => "nft",
            GovernanceDetails::AbstractAccount { .. } => "abstract-account",
        };
        write!(f, "{str}")
    }
}

#[cosmwasm_schema::cw_serde]
pub struct TopLevelOwnerResponse {
    pub address: Addr,
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;

    use cosmwasm_std::testing::mock_dependencies;

    #[coverage_helper::test]
    fn test_verify() {
        let deps = mock_dependencies();
        let owner = deps.api.addr_make("monarch");
        let gov = GovernanceDetails::Monarchy {
            monarch: owner.to_string(),
        };
        assert!(gov.verify(deps.as_ref()).is_ok());

        let gov_addr = deps.api.addr_make("gov_addr");
        let gov = GovernanceDetails::External {
            governance_address: gov_addr.to_string(),
            governance_type: "external-multisig".to_string(),
        };
        assert!(gov.verify(deps.as_ref()).is_ok());

        let gov = GovernanceDetails::Monarchy {
            monarch: "NOT_OK".to_string(),
        };
        assert!(gov.verify(deps.as_ref()).is_err());
        let gov = GovernanceDetails::External {
            governance_address: "gov_address".to_string(),
            governance_type: "gov_type".to_string(),
        };
        // '_' not allowed
        assert!(gov.verify(deps.as_ref()).is_err());

        // too short
        let gov_address = deps.api.addr_make("gov_address");
        let gov = GovernanceDetails::External {
            governance_address: gov_address.to_string(),
            governance_type: "gov".to_string(),
        };
        assert!(gov.verify(deps.as_ref()).is_err());

        // too long
        let gov = GovernanceDetails::External {
            governance_address: gov_address.to_string(),
            governance_type: "a".repeat(190),
        };
        assert!(gov.verify(deps.as_ref()).is_err());

        // invalid addr
        let gov = GovernanceDetails::External {
            governance_address: "NOT_OK".to_string(),
            governance_type: "gov_type".to_string(),
        };
        assert!(gov.verify(deps.as_ref()).is_err());

        // good nft
        let collection_addr = deps.api.addr_make("collection_addr");
        let gov = GovernanceDetails::NFT {
            collection_addr: collection_addr.to_string(),
            token_id: "1".to_string(),
        };
        assert!(gov.verify(deps.as_ref()).is_ok());
    }
}
