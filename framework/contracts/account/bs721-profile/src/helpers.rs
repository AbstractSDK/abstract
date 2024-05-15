use crate::QueryMsg;
use bs_profile::{TextRecord, NFT};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmQuery};

/// NameCollectionContract is a wrapper around Addr that provides a lot of helpers
#[cw_serde]
pub struct NameCollectionContract(pub Addr);

impl NameCollectionContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn name(&self, querier: &QuerierWrapper, address: &str) -> StdResult<String> {
        let res: String = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&QueryMsg::Name {
                address: address.to_string(),
            })?,
        }))?;

        Ok(res)
    }

    pub fn image_nft(&self, querier: &QuerierWrapper, name: &str) -> StdResult<Option<NFT>> {
        let res: Option<NFT> = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&QueryMsg::ImageNFT {
                name: name.to_string(),
            })?,
        }))?;

        Ok(res)
    }

    pub fn text_records(&self, querier: &QuerierWrapper, name: &str) -> StdResult<Vec<TextRecord>> {
        let res: Vec<TextRecord> = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&QueryMsg::TextRecords {
                name: name.to_string(),
            })?,
        }))?;

        Ok(res)
    }

    pub fn is_twitter_verified(&self, querier: &QuerierWrapper, name: &str) -> StdResult<bool> {
        let res: bool = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&QueryMsg::IsTwitterVerified {
                name: name.to_string(),
            })?,
        }))?;

        Ok(res)
    }
}
