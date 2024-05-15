pub use crate::error::ContractError;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

pub mod contract;
mod error;
mod helpers;
pub mod sudo;
pub mod commands;

pub use helpers::NameCollectionContract;

#[cfg(test)]
pub mod unit_tests;

use bs721_base::InstantiateMsg as Bs721InstantiateMsg;
use bs721_base::msg::CollectionInfoResponse;
use bs721_base::{ExecuteMsg as Bs721ExecuteMsg, MintMsg, QueryMsg as Bs721QueryMsg};
use bs_profile::{Metadata, TextRecord, NFT};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Binary, Empty};
use cw721::{
    AllNftInfoResponse, ApprovalResponse, ApprovalsResponse, ContractInfoResponse, Expiration,
    NftInfoResponse, NumTokensResponse, OperatorsResponse, OwnerOfResponse, TokensResponse,
};
use cw721_base::MinterResponse;

#[allow(unused_imports)]
use crate::{};

pub mod state {
    use cosmwasm_std::Addr;
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};

    use super::SudoParams;
    pub type TokenUri = Addr;
    pub type TokenId = String;

    /// Address of the text record verification oracle
    pub const REVERSE_MAP: Map<&TokenUri, TokenId> = Map::new("rm");
    pub const VERIFIER: Admin = Admin::new("verifier");
    pub const SUDO_PARAMS: Item<SudoParams> = Item::new("params");
    pub const NAME_MARKETPLACE: Item<Addr> = Item::new("name-marketplace");
}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub verifier: Option<String>,
    pub base_init_msg: Bs721InstantiateMsg,
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg<T> {
    /// Set name marketplace contract address
    SetNameMarketplace { address: String },
    /// Set an address for name reverse lookup and updates token_uri
    /// Can be an EOA or a contract address.
    AssociateAddress {
        name: String,
        address: Option<String>,
    },
    /// Update image NFT
    UpdateImageNft { name: String, nft: Option<NFT> },
    /// Add text record ex: twitter handle, discord name, etc
    AddTextRecord { name: String, record: TextRecord },
    /// Remove text record ex: twitter handle, discord name, etc
    RemoveTextRecord { name: String, record_name: String },
    /// Update text record ex: twitter handle, discord name, etc
    UpdateTextRecord { name: String, record: TextRecord },
    /// Verify a text record as true or false (via oracle)
    VerifyTextRecord {
        name: String,
        record_name: String,
        result: bool,
    },
    /// Update the reset the verification oracle
    UpdateVerifier { verifier: Option<String> },
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: String },
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke { spender: String, token_id: String },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: String },
    /// Mint a new NFT, can only be called by the contract minter
    Mint(MintMsg<T>),
    /// Burn an NFT the sender has access to
    Burn { token_id: String },
    /// Update specific collection info fields
    // UpdateCollectionInfo {
    //     collection_info: UpdateCollectionInfoMsg<RoyaltyInfoResponse>,
    // },
    /// Called by the minter to update trading start time
    // UpdateStartTradingTime(Option<Timestamp>),
    /// Freeze collection info from further updates
    FreezeCollectionInfo {},
}

impl<T> From<ExecuteMsg<T>> for Bs721ExecuteMsg<T, Empty> {
    fn from(msg: ExecuteMsg<T>) -> Bs721ExecuteMsg<T, Empty> {
        match msg {
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => Bs721ExecuteMsg::TransferNft {
                recipient,
                token_id,
            },
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => Bs721ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            },
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => Bs721ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            },
            ExecuteMsg::ApproveAll { operator, expires } => {
                Bs721ExecuteMsg::ApproveAll { operator, expires }
            }
            ExecuteMsg::Revoke { spender, token_id } => {
                Bs721ExecuteMsg::Revoke { spender, token_id }
            }
            ExecuteMsg::RevokeAll { operator } => Bs721ExecuteMsg::RevokeAll { operator },
            ExecuteMsg::Burn { token_id } => Bs721ExecuteMsg::Burn { token_id },
            // ExecuteMsg::UpdateCollectionInfo { collection_info } => {
            //     Bs721ExecuteMsg::UpdateCollectionInfo { collection_info }
            // }
            // ExecuteMsg::UpdateStartTradingTime(start_time) => {
            //     Bs721ExecuteMsg::UpdateStartTradingTime(start_time)
            // }
            // ExecuteMsg::FreezeCollectionInfo {} => Bs721ExecuteMsg::FreezeCollectionInfo {},
            ExecuteMsg::Mint(msg) => Bs721ExecuteMsg::Mint(MintMsg::from(msg)),
            _ => unreachable!("Invalid ExecuteMsg"),
        }
    }
}
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    /// Contains the enabled modules
    /// Returns sudo params
    #[returns(SudoParams)]
    Params {},
    /// Reverse lookup of name for address
    #[returns(String)]
    Name { address: String },
    /// Returns the marketplace contract address
    #[returns(Addr)]
    NameMarketplace {},
    /// Returns the associated address for a name
    #[returns(Addr)]
    AssociatedAddress { name: String },
    /// Returns the image NFT for a name
    #[returns(Option<NFT>)]
    ImageNFT { name: String },
    /// Returns the text records for a name
    #[returns(Vec<TextRecord>)]
    TextRecords { name: String },
    /// Returns if Twitter is verified for a name
    #[returns(bool)]
    IsTwitterVerified { name: String },
    /// Returns the verification oracle address
    #[returns(Option<String>)]
    Verifier {},
    /// Everything below is inherited from bs721
    #[returns(OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(ApprovalResponse)]
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    #[returns(ApprovalsResponse)]
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(OperatorsResponse)]
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(NumTokensResponse)]
    NumTokens {},
    #[returns(ContractInfoResponse)]
    ContractInfo {},
    #[returns(NftInfoResponse<Metadata>)]
    NftInfo { token_id: String },
    #[returns(AllNftInfoResponse<Metadata>)]
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(MinterResponse)]
    Minter {},
    #[returns(CollectionInfoResponse)]
    CollectionInfo {},
}

impl From<QueryMsg> for Bs721QueryMsg<Empty> {
    fn from(msg: QueryMsg) -> Bs721QueryMsg<Empty> {
        match msg {
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => Bs721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
            QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            } => Bs721QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            },
            QueryMsg::Approvals {
                token_id,
                include_expired,
            } => Bs721QueryMsg::Approvals {
                token_id,
                include_expired,
            },
            QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            } => Bs721QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            },
            QueryMsg::NumTokens {} => Bs721QueryMsg::NumTokens {},
            QueryMsg::ContractInfo {} => Bs721QueryMsg::ContractInfo {},
            QueryMsg::NftInfo { token_id } => Bs721QueryMsg::NftInfo { token_id },
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => Bs721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            },
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => Bs721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllTokens { start_after, limit } => {
                Bs721QueryMsg::AllTokens { start_after, limit }
            }
            QueryMsg::Minter {} => Bs721QueryMsg::Minter {},
            QueryMsg::CollectionInfo {} => Bs721QueryMsg::CollectionInfo {},
            _ => unreachable!("cannot convert {:?} to Cw721QueryMsg", msg),
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct SudoParams {
    pub max_record_count: u32,
}

#[cosmwasm_schema::cw_serde]
pub enum SudoMsg {
    UpdateParams { max_record_count: u32 },
}