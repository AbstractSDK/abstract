use cosmwasm_schema::cw_serde;
use cosmwasm_std::Timestamp;

#[cw_serde]
pub enum BsProfileMarketplaceExecuteMsg {
    /// List name NFT on the marketplace by creating a new ask
    /// Only the name minter can call this.
    SetAsk { token_id: String, seller: String },
    /// Remove name on the marketplace.
    /// Only the name collection can call this (i.e: when burned).
    RemoveAsk { token_id: String },
    /// Update ask when an NFT is transferred
    /// Only the name collection can call this
    UpdateAsk { token_id: String, seller: String },
    /// Place a bid on an existing ask
    SetBid { token_id: String },
    /// Remove an existing bid from an ask
    RemoveBid { token_id: String },
    /// Accept a bid on an existing ask
    AcceptBid { token_id: String, bidder: String },
    /// Fund renewal of a name
    FundRenewal { token_id: String },
    /// Refund a renewal of a name
    RefundRenewal { token_id: String },
    /// Check if expired names have been paid for, and collect fees.
    /// If not paid, transfer ownership to the highest bidder.
    ProcessRenewals { time: Timestamp },
    /// Setup contract with minter and collection addresses
    /// Can only be run once
    Setup { minter: String, collection: String },
}
