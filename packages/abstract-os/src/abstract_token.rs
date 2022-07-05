//! # Abstract Token
//!
//! `abstract_os::abstract_token` implements shared functionality that's useful for creating new Abstract add-ons.
//!
//! ## Description
//! An add-on is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract while also being migratable.
//! The source code is accessible on [todo](todo).
//!
use std::convert::TryInto;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Binary, StdError, StdResult, Uint128};
use cw20::{Cw20Coin, Cw20ExecuteMsg, Expiration, Logo, MinterResponse};
use cw20_base::msg::QueryMsg as Cw20QueryMsg;

/// ## Description
/// This structure describes the basic settings for creating a token contract.
/// TokenContract InstantiateMsg
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    /// the name
    pub name: String,
    /// the symbol
    pub symbol: String,
    /// the precision after the decimal point
    pub decimals: u8,
    /// the initial balance of token
    pub initial_balances: Vec<Cw20Coin>,
    /// the controls configs of type [`MinterResponse`]
    pub mint: Option<MinterResponse>,
    /// address of version control contract.
    pub version_control_address: String,
}

/// ## Description
/// This structure describes a migration message.
/// We currently take no arguments for migrations.
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MigrateMsg {}

impl InstantiateMsg {
    pub fn get_cap(&self) -> Option<Uint128> {
        self.mint.as_ref().and_then(|v| v.cap)
    }

    pub fn validate(&self) -> StdResult<()> {
        // Check name, symbol, decimals
        if !is_valid_name(&self.name) {
            return Err(StdError::generic_err(
                "Name is not in the expected format (3-50 UTF-8 bytes)",
            ));
        }
        if !is_valid_symbol(&self.symbol) {
            return Err(StdError::generic_err(
                "Ticker symbol is not in expected format [a-zA-Z\\-]{3,12}",
            ));
        }
        if self.decimals > 18 {
            return Err(StdError::generic_err("Decimals must not exceed 18"));
        }
        Ok(())
    }
}

/// ## Description
/// Checks the validity of the token name
/// ## Params
/// * **name** is the object of type [`str`]. the name to check
fn is_valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.len() < 3 || bytes.len() > 50 {
        return false;
    }
    true
}

/// ## Description
/// Checks the validity of the token symbol
/// ## Params
/// * **symbol** is the object of type [`str`]. the symbol to check
fn is_valid_symbol(symbol: &str) -> bool {
    let bytes = symbol.as_bytes();
    if bytes.len() < 3 || bytes.len() > 12 {
        return false;
    }
    for byte in bytes.iter() {
        if (*byte != 45) && (*byte < 65 || *byte > 90) && (*byte < 97 || *byte > 122) {
            return false;
        }
    }
    true
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateWhitelist {
        to_add: Vec<String>,
        to_remove: Vec<String>,
        restrict_transfers: Option<bool>,
    },
    UpdateAdmin {
        new_admin: String,
    },
    /// Transfer is a base message to move tokens to another account without triggering actions
    Transfer {
        recipient: String,
        amount: Uint128,
    },
    /// Burn is a base message to destroy tokens forever
    Burn {
        amount: Uint128,
    },
    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Only with "approval" extension. Allows spender to access an additional amount tokens
    /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
    /// expiration with this one.
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Only with "approval" extension. Lowers the spender's access of tokens
    /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
    /// allowance expiration with this one.
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Only with "approval" extension. Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    /// Only with "approval" extension. Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Only with "approval" extension. Destroys tokens forever
    BurnFrom {
        owner: String,
        amount: Uint128,
    },
    /// Only with the "mintable" extension. If authorized, creates amount new tokens
    /// and adds to the recipient balance.
    Mint {
        recipient: String,
        amount: Uint128,
    },
    /// Only with the "marketing" extension. If authorized, updates marketing metadata.
    /// Setting None/null for any of these will leave it unchanged.
    /// Setting Some("") will clear this field on the contract storage
    UpdateMarketing {
        /// A URL pointing to the project behind this token.
        project: Option<String>,
        /// A longer description of the token and it's utility. Designed for tooltips or such
        description: Option<String>,
        /// The address (if any) who can update this data structure
        marketing: Option<String>,
    },
    /// If set as the "marketing" role on the contract, upload a new URL, SVG, or PNG for the token
    UploadLogo(Logo),
}

impl TryInto<Cw20ExecuteMsg> for ExecuteMsg {
    type Error = StdError;

    fn try_into(self) -> Result<Cw20ExecuteMsg, Self::Error> {
        match self {
            ExecuteMsg::UpdateWhitelist {
                to_add: _,
                to_remove: _,
                restrict_transfers: _,
            } => Err(StdError::generic_err("can't parse into cw20 msg")),
            ExecuteMsg::UpdateAdmin { new_admin: _ } => {
                Err(StdError::generic_err("can't parse into cw20 msg"))
            }
            ExecuteMsg::Transfer { recipient, amount } => {
                Ok(Cw20ExecuteMsg::Transfer { recipient, amount })
            }
            ExecuteMsg::Burn { amount } => Ok(Cw20ExecuteMsg::Burn { amount }),
            ExecuteMsg::Send {
                contract,
                amount,
                msg,
            } => Ok(Cw20ExecuteMsg::Send {
                contract,
                amount,
                msg,
            }),
            ExecuteMsg::IncreaseAllowance {
                spender,
                amount,
                expires,
            } => Ok(Cw20ExecuteMsg::IncreaseAllowance {
                spender,
                amount,
                expires,
            }),
            ExecuteMsg::DecreaseAllowance {
                spender,
                amount,
                expires,
            } => Ok(Cw20ExecuteMsg::DecreaseAllowance {
                spender,
                amount,
                expires,
            }),
            ExecuteMsg::TransferFrom {
                owner,
                recipient,
                amount,
            } => Ok(Cw20ExecuteMsg::TransferFrom {
                owner,
                recipient,
                amount,
            }),
            ExecuteMsg::SendFrom {
                owner,
                contract,
                amount,
                msg,
            } => Ok(Cw20ExecuteMsg::SendFrom {
                owner,
                contract,
                amount,
                msg,
            }),
            ExecuteMsg::BurnFrom { owner, amount } => {
                Ok(Cw20ExecuteMsg::BurnFrom { owner, amount })
            }
            ExecuteMsg::Mint { recipient, amount } => {
                Ok(Cw20ExecuteMsg::Mint { recipient, amount })
            }
            ExecuteMsg::UpdateMarketing {
                project,
                description,
                marketing,
            } => Ok(Cw20ExecuteMsg::UpdateMarketing {
                project,
                description,
                marketing,
            }),
            ExecuteMsg::UploadLogo(l) => Ok(Cw20ExecuteMsg::UploadLogo(l)),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    /// Returns the current balance of the given address, 0 if unset.
    /// Return type: BalanceResponse.
    Balance {
        address: String,
    },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    /// Return type: TokenInfoResponse.
    TokenInfo {},
    /// Only with "mintable" extension.
    /// Returns who can mint and the hard cap on maximum tokens after minting.
    /// Return type: MinterResponse.
    Minter {},
    /// Only with "allowance" extension.
    /// Returns how much spender can use from owner account, 0 if unset.
    /// Return type: AllowanceResponse.
    Allowance {
        owner: String,
        spender: String,
    },
    /// Only with "enumerable" extension (and "allowances")
    /// Returns all allowances this owner has approved. Supports pagination.
    /// Return type: AllAllowancesResponse.
    AllAllowances {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Only with "enumerable" extension
    /// Returns all accounts that have balances. Supports pagination.
    /// Return type: AllAccountsResponse.
    AllAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Only with "marketing" extension
    /// Returns more metadata on the contract to display in the client:
    /// - description, logo, project url, etc.
    /// Return type: MarketingInfoResponse
    MarketingInfo {},
    /// Only with "marketing" extension
    /// Downloads the embedded logo data (if stored on chain). Errors if no logo data is stored for this
    /// contract.
    /// Return type: DownloadLogoResponse.
    DownloadLogo {},
}

impl TryInto<Cw20QueryMsg> for QueryMsg {
    type Error = StdError;

    fn try_into(self) -> Result<Cw20QueryMsg, Self::Error> {
        match self {
            Self::Balance { address } => Ok(Cw20QueryMsg::Balance { address }),
            Self::TokenInfo {} => Ok(Cw20QueryMsg::TokenInfo {}),
            Self::Minter {} => Ok(Cw20QueryMsg::Minter {}),
            Self::Allowance { owner, spender } => Ok(Cw20QueryMsg::Allowance { owner, spender }),
            Self::AllAllowances {
                owner,
                start_after,
                limit,
            } => Ok(Cw20QueryMsg::AllAllowances {
                owner,
                start_after,
                limit,
            }),
            Self::AllAccounts { start_after, limit } => {
                Ok(Cw20QueryMsg::AllAccounts { start_after, limit })
            }
            Self::MarketingInfo {} => Ok(Cw20QueryMsg::MarketingInfo {}),
            Self::DownloadLogo {} => Ok(Cw20QueryMsg::DownloadLogo {}),
            QueryMsg::Config {} => Err(StdError::generic_err("could not convert into cw20 query")),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub transfers_restricted: bool,
    pub version_control_address: String,
    pub whitelisted_addr: Vec<String>,
    pub admin: String,
}
