use crate::{InstantiateMsg, SudoParams};
use abstract_std::PROFILE_MARKETPLACE;
use bs721::CollectionInfo;
use bs721_base::MintMsg;
use bs721_base::{ContractError::Unauthorized, InstantiateMsg as Bs721InstantiateMsg};
use bs_profile::{Metadata, TextRecord, NFT};
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_json, to_json_binary, Addr, ContractInfoResponse, ContractResult, Empty, OwnedDeps,
    Querier, QuerierResult, QueryRequest, StdError, SystemError, SystemResult, WasmQuery,
};
use cw721::Cw721Query;
use std::marker::PhantomData;

use crate::{commands::*, contract::*, ContractError};

pub type Bs721NameContract<'a> = bs721_base::Bs721Contract<'a, Metadata, Empty, Empty, Empty>;
const CREATOR: &str = "creator";
const IMPOSTER: &str = "imposter";

pub fn mock_deps() -> OwnedDeps<MockStorage, MockApi, CustomMockQuerier, Empty> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: CustomMockQuerier::new(MockQuerier::new(&[])),
        custom_query_type: PhantomData,
    }
}

pub struct CustomMockQuerier {
    base: MockQuerier,
}

impl Querier for CustomMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_json(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };

        self.handle_query(&request)
    }
}

impl CustomMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::ContractInfo { contract_addr: _ }) => {
                let mut response = ContractInfoResponse::default();
                response.code_id = 1;
                response.creator = CREATOR.to_string();
                SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
            }
            _ => self.base.handle_query(request),
        }
    }

    pub fn new(base: MockQuerier<Empty>) -> Self {
        CustomMockQuerier { base }
    }
}

fn init_msg() -> InstantiateMsg {
    let collection_info = CollectionInfo {
        creator: "bobo".to_string(),
        description: "bobo name da best".to_string(),
        image: "ipfs://something".to_string(),
        external_link: None,
        explicit_content: None,
        start_trading_time: None,
        royalty_info: None,
    };
    let base_init_msg = Bs721InstantiateMsg {
        name: "SG Names".to_string(),
        symbol: "NAME".to_string(),
        minter: CREATOR.to_string(),
        collection_info,
        uri: None,
    };
    InstantiateMsg {
        verifier: None,
        base_init_msg,
        marketplace: Addr::unchecked(PROFILE_MARKETPLACE),
    }
}

#[test]
fn init() {
    // instantiate sg-names collection
    let mut deps = mock_deps();
    let info = mock_info(CREATOR, &[]);

    instantiate(deps.as_mut(), mock_env(), info, init_msg()).unwrap();
}

#[test]
fn mint_and_update() {
    let contract = Bs721NameContract::default();
    // instantiate sg-names collection
    let mut deps = mock_deps();
    let info = mock_info(CREATOR, &[]);

    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg()).unwrap();

    // retrieve max record count
    let params: SudoParams =
        from_json(&query(deps.as_ref(), mock_env(), QueryMsg::Params {}).unwrap()).unwrap();
    let max_record_count = params.max_record_count;

    // mint token
    let token_id = "Enterprise";

    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            bs721_base::ExecuteMsg::Mint(MintMsg {
                token_id: token_id.to_string(),
                owner: info.sender.to_string(),
                token_uri: None,
                seller_fee_bps: None,
                payment_addr: None,
                extension: Metadata::default(),
            }),
        )
        .unwrap();

    // check token contains correct metadata
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.token_uri, None);
    assert_eq!(res.extension, Metadata::default());

    // update image
    let new_nft = NFT {
        collection: Addr::unchecked("contract"),
        token_id: "token_id".to_string(),
    };
    let update_image_msg = ExecuteMsg::UpdateImageNft {
        name: token_id.to_string(),
        nft: Some(new_nft.clone()),
    };
    let res = execute(deps.as_mut(), mock_env(), info.clone(), update_image_msg).unwrap();
    let nft_value = res.events[0].attributes[2].value.clone().into_bytes();
    let nft: NFT = from_json(&nft_value).unwrap();
    assert_eq!(nft, new_nft);

    // add text record
    let new_record = TextRecord {
        name: "test".to_string(),
        value: "test".to_string(),
        verified: None,
    };
    let update_record_msg = ExecuteMsg::UpdateTextRecord {
        name: token_id.to_string(),
        record: new_record.clone(),
    };
    let res = execute(deps.as_mut(), mock_env(), info.clone(), update_record_msg).unwrap();
    let record_value = res.events[0].attributes[2].value.clone().into_bytes();
    let record: TextRecord = from_json(&record_value).unwrap();
    assert_eq!(record, new_record);

    let records = query_text_records(deps.as_ref(), token_id).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].name, "test");
    assert_eq!(records[0].value, "test");

    let is_twitter_verified = query_is_twitter_verified(deps.as_ref(), token_id).unwrap();
    assert!(!is_twitter_verified);

    // trigger too many records error
    for i in 1..=(max_record_count) {
        let new_record = TextRecord {
            name: format!("key{:?}", i),
            value: "value".to_string(),
            verified: None,
        };
        let update_record_msg = ExecuteMsg::UpdateTextRecord {
            name: token_id.to_string(),
            record: new_record.clone(),
        };
        if i == max_record_count {
            let res = execute(deps.as_mut(), mock_env(), info.clone(), update_record_msg);
            assert_eq!(
                res.unwrap_err().to_string(),
                ContractError::TooManyRecords {
                    max: max_record_count
                }
                .to_string()
            );
            break;
        } else {
            execute(deps.as_mut(), mock_env(), info.clone(), update_record_msg).unwrap();
        }
    }

    // rm text records
    let rm_record_msg = ExecuteMsg::RemoveTextRecord {
        name: token_id.to_string(),
        record_name: "test".to_string(),
    };
    execute(deps.as_mut(), mock_env(), info.clone(), rm_record_msg).unwrap();

    for i in 1..=(max_record_count) {
        let record_name = format!("key{:?}", i);
        let rm_record_msg = ExecuteMsg::RemoveTextRecord {
            name: token_id.to_string(),
            record_name: record_name.clone(),
        };
        execute(deps.as_mut(), mock_env(), info.clone(), rm_record_msg).unwrap();
    }
    // txt record count should be 0
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension.records.len(), 0);

    // add txt record
    let record = TextRecord {
        name: "test".to_string(),
        value: "test".to_string(),
        verified: None,
    };
    let add_record_msg = ExecuteMsg::AddTextRecord {
        name: token_id.to_string(),
        record,
    };
    // unauthorized
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(IMPOSTER, &[]),
        add_record_msg.clone(),
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        ContractError::Base(Unauthorized {}).to_string()
    );
    // passes
    execute(deps.as_mut(), mock_env(), info.clone(), add_record_msg).unwrap();
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension.records.len(), 1);

    // add another txt record
    let record = TextRecord {
        name: "twitter".to_string(),
        value: "jackdorsey".to_string(),
        verified: None,
    };
    let add_record_msg = ExecuteMsg::AddTextRecord {
        name: token_id.to_string(),
        record,
    };
    execute(deps.as_mut(), mock_env(), info.clone(), add_record_msg).unwrap();
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension.records.len(), 2);

    // add duplicate record RecordNameAlreadyExist
    let record = TextRecord {
        name: "test".to_string(),
        value: "testtesttest".to_string(),
        verified: None,
    };
    let add_record_msg = ExecuteMsg::AddTextRecord {
        name: token_id.to_string(),
        record: record.clone(),
    };
    let err = execute(deps.as_mut(), mock_env(), info.clone(), add_record_msg).unwrap_err();
    assert_eq!(
        err.to_string(),
        ContractError::RecordNameAlreadyExists {}.to_string()
    );

    // update txt record
    let update_record_msg = ExecuteMsg::UpdateTextRecord {
        name: token_id.to_string(),
        record: record.clone(),
    };
    execute(deps.as_mut(), mock_env(), info.clone(), update_record_msg).unwrap();
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension.records.len(), 2);
    assert_eq!(res.extension.records[1].value, record.value);

    // rm txt record
    let rm_record_msg = ExecuteMsg::RemoveTextRecord {
        name: token_id.to_string(),
        record_name: record.name,
    };
    execute(deps.as_mut(), mock_env(), info, rm_record_msg).unwrap();
    let res = contract
        .parent
        .nft_info(deps.as_ref(), token_id.into())
        .unwrap();
    assert_eq!(res.extension.records.len(), 1);
}

#[test]
fn query_names() {
    let deps = mock_deps();
    let address = "bitsong1y54exmx84cqtasvjnskf9f63djuuj68pj7jph3".to_string();
    let err = query_name(deps.as_ref(), address.clone()).unwrap_err();
    assert_eq!(
        err.to_string(),
        StdError::GenericErr {
            msg: format!("No name associated with address {}", address)
        }
        .to_string()
    );
}

#[test]
fn test_transcode() {
    let res = transcode("cosmos1y54exmx84cqtasvjnskf9f63djuuj68p7hqf47");
    assert_eq!(
        res.unwrap(),
        "bitsong1y54exmx84cqtasvjnskf9f63djuuj68pj7jph3"
    );
}
