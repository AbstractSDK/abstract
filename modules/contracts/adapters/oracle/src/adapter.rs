use abstract_core::objects::pool_id::PoolAddressBase;
use abstract_sdk::{
    cw_helpers::Chargeable,
    features::{AbstractNameService, AbstractRegistryAccess},
    Execution,
};
use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps};
use cw_asset::{AssetBase, AssetInfoBase};

use crate::msg::OracleAction;

impl<T> OracleAdapter for T where T: AbstractNameService + Execution + AbstractRegistryAccess {}

pub(crate) type ReplyId = u64;

pub trait OracleAdapter: AbstractNameService + AbstractRegistryAccess + Execution {}
