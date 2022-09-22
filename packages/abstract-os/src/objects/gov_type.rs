//! # Governance structure object

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Governance types
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema)]

pub enum GovernanceDetails {
    /// A single address is admin
    Monarchy {
        /// The monarch's address
        monarch: String,
    },
    /// An external governance source
    External {
        /// The external contract address
        governance_address: String,
        /// Governance type used for doing extra off-chain queries depending on the type.
        governance_type: String,
    },
}

impl ToString for GovernanceDetails {
    fn to_string(&self) -> String {
        match self {
            GovernanceDetails::Monarchy { monarch: _ } => "monarchy".to_string(),
            GovernanceDetails::External {
                governance_address: _,
                governance_type,
            } => governance_type.clone(),
        }
    }
}
