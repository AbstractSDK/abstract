//! # Governance structure object

use cosmwasm_std::{Addr, Api};
use cw_address_like::AddressLike;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    /// An external governance source
    External {
        /// The external contract address
        governance_address: T,
        /// Governance type used for doing extra off-chain queries depending on the type.
        governance_type: String,
    },
}

impl GovernanceDetails<String> {
    /// Verify the governance details and convert to `Self<Addr>`
    pub fn verify(self, api: &dyn Api) -> Result<GovernanceDetails<Addr>, AbstractError> {
        match self {
            GovernanceDetails::Monarchy { monarch } => {
                let addr = api.addr_validate(&monarch)?;
                Ok(GovernanceDetails::Monarchy { monarch: addr })
            }
            GovernanceDetails::External {
                governance_address,
                governance_type,
            } => {
                let addr = api.addr_validate(&governance_address)?;

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
        }
    }
}

impl GovernanceDetails<Addr> {
    /// Get the owner address from the governance details
    pub fn owner_address(&self) -> Addr {
        match self {
            GovernanceDetails::Monarchy { monarch } => monarch.clone(),
            GovernanceDetails::External {
                governance_address, ..
            } => governance_address.clone(),
        }
    }
}

impl From<GovernanceDetails<Addr>> for GovernanceDetails<String> {
    fn from(value: GovernanceDetails<Addr>) -> Self {
        match value {
            GovernanceDetails::Monarchy { monarch } => GovernanceDetails::Monarchy {
                monarch: monarch.to_string(),
            },
            GovernanceDetails::External {
                governance_address,
                governance_type,
            } => GovernanceDetails::External {
                governance_address: governance_address.to_string(),
                governance_type,
            },
        }
    }
}

impl<T: AddressLike> ToString for GovernanceDetails<T> {
    fn to_string(&self) -> String {
        match self {
            GovernanceDetails::Monarchy { .. } => "monarch".to_string(),
            GovernanceDetails::External {
                governance_type, ..
            } => governance_type.to_owned(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use speculoos::prelude::*;

    #[test]
    fn test_verify() {
        let deps = mock_dependencies();
        let gov = GovernanceDetails::Monarchy {
            monarch: "monarch".to_string(),
        };
        assert_that!(gov.verify(&deps.api)).is_ok();

        let gov = GovernanceDetails::External {
            governance_address: "gov_addr".to_string(),
            governance_type: "external-multisig".to_string(),
        };
        assert_that!(gov.verify(&deps.api)).is_ok();

        let gov = GovernanceDetails::Monarchy {
            monarch: "NOT_OK".to_string(),
        };
        assert_that!(gov.verify(&deps.api)).is_err();

        let gov = GovernanceDetails::External {
            governance_address: "gov_address".to_string(),
            governance_type: "gov_type".to_string(),
        };
        // '_' not allowed
        assert_that!(gov.verify(&deps.api)).is_err();

        // too short
        let gov = GovernanceDetails::External {
            governance_address: "gov_address".to_string(),
            governance_type: "gov".to_string(),
        };
        assert_that!(gov.verify(&deps.api)).is_err();

        // too long
        let gov = GovernanceDetails::External {
            governance_address: "gov_address".to_string(),
            governance_type: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        };
        assert_that!(gov.verify(&deps.api)).is_err();

        // invalid addr
        let gov = GovernanceDetails::External {
            governance_address: "NOT_OK".to_string(),
            governance_type: "gov_type".to_string(),
        };
        assert_that!(gov.verify(&deps.api)).is_err();
    }
}
