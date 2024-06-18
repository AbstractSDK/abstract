//! # Governance structure object

use cosmwasm_std::{Addr, Deps, QuerierWrapper};
use cw721::OwnerOfResponse;
use cw_address_like::AddressLike;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::account::ACCOUNT_ID;
use crate::AbstractError;

const MIN_GOV_TYPE_LENGTH: usize = 4;
const MAX_GOV_TYPE_LENGTH: usize = 64;

/// Governance types
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub enum GovernanceDetails<T: AddressLike> {
    /// A single address is admin
    Monarchy {
        /// The monarch's address
        monarch: T,
    },
    /// Used when the account is a sub-account of another account.
    SubAccount {
        /// The manager of the account of which this account is the sub-account.
        manager: T,
        /// The proxy of the account of which this account is the sub-account.
        proxy: T,
    },
    /// An external governance source
    External {
        /// The external contract address
        governance_address: T,
        /// Governance type used for doing extra off-chain queries depending on the type.
        governance_type: String,
    },
    /// Renounced account
    /// This account no longer has an owner and cannot be used.
    Renounced {},
    NFT {
        collection_addr: T,
        token_id: String,
    },
}

impl GovernanceDetails<String> {
    /// Verify the governance details and convert to `Self<Addr>`
    pub fn verify(
        self,
        deps: Deps,
        version_control_addr: Addr,
    ) -> Result<GovernanceDetails<Addr>, AbstractError> {
        match self {
            GovernanceDetails::Monarchy { monarch } => {
                let addr = deps.api.addr_validate(&monarch)?;
                Ok(GovernanceDetails::Monarchy { monarch: addr })
            }
            GovernanceDetails::SubAccount { manager, proxy } => {
                let manager_addr = deps.api.addr_validate(&manager)?;
                let account_id = ACCOUNT_ID.query(&deps.querier, manager_addr)?;
                let base = crate::version_control::state::ACCOUNT_ADDRESSES.query(
                    &deps.querier,
                    version_control_addr,
                    &account_id,
                )?;
                let Some(b) = base else {
                    return Err(AbstractError::Std(cosmwasm_std::StdError::generic_err(
                        format!("Version control does not have account id of manager {manager}"),
                    )));
                };
                if b.manager == manager && b.proxy == proxy {
                    Ok(GovernanceDetails::SubAccount {
                        manager: b.manager,
                        proxy: b.proxy,
                    })
                } else {
                    Err(AbstractError::Std(cosmwasm_std::StdError::generic_err(
                        "Verification of sub-account failed, manager and proxy has different account ids",
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
            } => {
                // get owner of token_id from collection
                let owner: OwnerOfResponse = deps.querier.query_wasm_smart(
                    &collection_addr,
                    &cw721::Cw721QueryMsg::OwnerOf {
                        token_id: token_id.clone(),
                        include_expired: None,
                    },
                )?;
                // verify owner
                // if *sender == owner.owner {
                //     Ok(())
                // } else {
                //     Err(ownership_error)
                // }

                Ok(GovernanceDetails::NFT {
                    collection_addr: deps.api.addr_validate(&collection_addr.to_string())?,
                    token_id,
                })
            }
        }
    }
}

impl GovernanceDetails<Addr> {
    /// Get the owner address from the governance details
    pub fn owner_address(&self, api: Option<QuerierWrapper>) -> Option<Addr> {
        match self {
            GovernanceDetails::Monarchy { monarch } => Some(monarch.clone()),
            GovernanceDetails::SubAccount { proxy, .. } => Some(proxy.clone()),
            GovernanceDetails::External {
                governance_address, ..
            } => Some(governance_address.clone()),
            GovernanceDetails::Renounced {} => None,
            GovernanceDetails::NFT {
                collection_addr,
                token_id,
            } => {
                if let Some(api) = api {
                    let res: OwnerOfResponse = api
                        .query_wasm_smart(
                            collection_addr,
                            &cw721::Cw721QueryMsg::OwnerOf {
                                token_id: token_id.to_string(),
                                include_expired: None,
                            },
                        )
                        .unwrap();
                    return Some(Addr::unchecked(&res.owner));
                } else {
                    return None;
                }
            }
        }
    }
}

impl From<GovernanceDetails<Addr>> for GovernanceDetails<String> {
    fn from(value: GovernanceDetails<Addr>) -> Self {
        match value {
            GovernanceDetails::Monarchy { monarch } => GovernanceDetails::Monarchy {
                monarch: monarch.into_string(),
            },
            GovernanceDetails::SubAccount { manager, proxy } => GovernanceDetails::SubAccount {
                manager: manager.into_string(),
                proxy: proxy.into_string(),
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
        }
    }
}

impl<T: AddressLike> std::fmt::Display for GovernanceDetails<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            GovernanceDetails::Monarchy { .. } => "monarch".to_string(),
            GovernanceDetails::SubAccount { .. } => "sub-account".to_string(),
            GovernanceDetails::External {
                governance_type, ..
            } => governance_type.to_owned(),
            GovernanceDetails::Renounced {} => "renounced".to_string(),
            GovernanceDetails::NFT { .. } => "nft".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::mock_dependencies;
    use speculoos::prelude::*;

    use super::*;

    #[test]
    fn test_verify() {
        let deps = mock_dependencies();
        let gov = GovernanceDetails::Monarchy {
            monarch: "monarch".to_string(),
        };
        let mock_version_control = Addr::unchecked("mock_version_control");
        assert_that!(gov.verify(deps.as_ref(), mock_version_control.clone())).is_ok();

        let gov = GovernanceDetails::External {
            governance_address: "gov_addr".to_string(),
            governance_type: "external-multisig".to_string(),
        };
        assert_that!(gov.verify(deps.as_ref(), mock_version_control.clone())).is_ok();

        let gov = GovernanceDetails::Monarchy {
            monarch: "NOT_OK".to_string(),
        };
        assert_that!(gov.verify(deps.as_ref(), mock_version_control.clone())).is_err();

        let gov = GovernanceDetails::External {
            governance_address: "gov_address".to_string(),
            governance_type: "gov_type".to_string(),
        };
        // '_' not allowed
        assert_that!(gov.verify(deps.as_ref(), mock_version_control.clone())).is_err();

        // too short
        let gov = GovernanceDetails::External {
            governance_address: "gov_address".to_string(),
            governance_type: "gov".to_string(),
        };
        assert_that!(gov.verify(deps.as_ref(), mock_version_control.clone())).is_err();

        // too long
        let gov = GovernanceDetails::External {
            governance_address: "gov_address".to_string(),
            governance_type: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        };
        assert_that!(gov.verify(deps.as_ref(), mock_version_control.clone())).is_err();

        // invalid addr
        let gov = GovernanceDetails::External {
            governance_address: "NOT_OK".to_string(),
            governance_type: "gov_type".to_string(),
        };
        assert_that!(gov.verify(deps.as_ref(), mock_version_control)).is_err();
    }
}
