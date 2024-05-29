use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;

pub mod common;
pub mod market;
pub mod minter;


pub type TokenId = String;

#[cosmwasm_schema::cw_serde]
pub struct NFT {
    pub collection: Addr,
    pub token_id: TokenId,
}

impl NFT {
    pub fn into_json_string(self: &NFT) -> String {
        String::from_utf8(cosmwasm_std::to_json_vec(&self).unwrap_or_default()).unwrap_or_default()
    }
}

#[cosmwasm_schema::cw_serde]
pub struct TextRecord {
    pub name: String,           // "twitter"
    pub value: String,          // "shan3v"
    pub verified: Option<bool>, // can only be set by oracle
}

impl TextRecord {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            verified: None,
        }
    }

    pub fn into_json_string(self: &TextRecord) -> String {
        String::from_utf8(cosmwasm_std::to_json_vec(&self).unwrap_or_default()).unwrap_or_default()
    }
}

/// Note that the address mapped to the name is stored in `token_uri`.
#[cosmwasm_schema::cw_serde]
#[derive(Default)]
pub struct Metadata {
    pub image_nft: Option<NFT>,
    pub records: Vec<TextRecord>,
}

impl Metadata {
    // Yes this is rather ugly. It is used to convert metadata into a JSON
    // string for use in emitting events. Events can only be strings, so
    // serializing it into a JSON string allows the indexer to parse it
    // and represent it as a type. Note that we have to use the CosmWasm fork
    // of serde_json to avoid floats.
    pub fn into_json_string(self: &Metadata) -> String {
        String::from_utf8(cosmwasm_std::to_json_vec(&self).unwrap_or_default()).unwrap_or_default()
    }
}


#[cosmwasm_schema::cw_serde]
pub enum BsProfileExecuteMsg {
    /// Set name marketplace contract address
    SetNameMarketplace { address: String },
    /// Set an address for name reverse lookup
    /// Can be an EOA or a contract address
    AssociateAddress {
        name: String,
        address: Option<String>,
    },
    /// Update image
    UpdateImageNft { name: String, nft: Option<NFT> },
    /// Update Metadata
    UpdateMetadata {
        name: String,
        metadata: Option<Metadata>,
    },
    /// Add text record ex: twitter handle, discord name, etc
    AddTextRecord { name: String, record: TextRecord },
    /// Remove text record ex: twitter handle, discord name, etc
    RemoveTextRecord { name: String, record_name: String },
    /// Update text record ex: twitter handle, discord name, etc
    UpdateTextRecord { name: String, record: TextRecord },
    /// Verify a text record (via oracle)
    VerifyTextRecord {
        name: String,
        record_name: String,
        result: bool,
    },
    /// Update the reset the verification oracle
    UpdateVerifier { verifier: Option<String> },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum BsProfileQueryMsg {
    /// `address` can be any Bech32 encoded address. It will be
    /// converted to a bitsong address for internal mapping.
    #[returns(String)]
    Name { address: String },
    #[returns(Addr)]
    NameMarketplace {},
    #[returns(String)]
    AssociatedAddress { name: String },
    #[returns(Option<NFT>)]
    ImageNFT { name: String },
    #[returns(Vec<TextRecord>)]
    TextRecords { name: String },
    #[returns(bool)]
    IsTwitterVerified { name: String },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_nft() {
        let nft = NFT {
            collection: Addr::unchecked("bitsong1abc123"),
            token_id: "1".to_string(),
        };
        let json = nft.into_json_string();
        assert_eq!(
            json,
            r#"{"collection":"bitsong1abc123","token_id":"1"}"#
        );
    }

    #[test]
    fn encode_text_record() {
        let mut record = TextRecord::new("twitter", "shan3v");
        let json = record.into_json_string();
        assert_eq!(
            json,
            r#"{"name":"twitter","value":"shan3v","verified":null}"#
        );

        record.verified = Some(true);

        let json = record.into_json_string();
        assert_eq!(
            json,
            r#"{"name":"twitter","value":"shan3v","verified":true}"#
        );

        record.verified = Some(false);

        let json = record.into_json_string();
        assert_eq!(
            json,
            r#"{"name":"twitter","value":"shan3v","verified":false}"#
        );
    }

    #[test]
    fn encode_metadata() {
        let image_nft = Some(NFT {
            collection: Addr::unchecked("bitsong1y54exmx84cqtasvjnskf9f63djuuj68pj7jph3"),
            token_id: "1".to_string(),
        });
        let record_1 = TextRecord::new("twitter", "shan3v");
        let mut record_2 = TextRecord::new("discord", "shan3v");
        record_2.verified = Some(true);
        let records = vec![record_1, record_2];
        let metadata = Metadata { image_nft, records };

        let json = metadata.into_json_string();
        assert_eq!(
            json,
            r#"{"image_nft":{"collection":"bitsong1y54exmx84cqtasvjnskf9f63djuuj68pj7jph3","token_id":"1"},"records":[{"name":"twitter","value":"shan3v","verified":null},{"name":"discord","value":"shan3v","verified":true}]}"#,
        );
    }
}