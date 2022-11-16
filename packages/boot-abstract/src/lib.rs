use boot_core::{Contract, IndexResponse, TxHandler};

use serde::Serialize;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

// Update `MyProjectName` to your project name and export contract implementations here.
// No need to touch anything else

pub struct AbstractOS<
    Chain: TxHandler,
    ExecuteMsg: Serialize + Debug,
    InitMsg: Serialize + Debug,
    QueryMsg: Serialize + Debug,
    M: Serialize + Debug,
>(Contract<Chain, ExecuteMsg, InitMsg, QueryMsg, M>)
where
    <Chain as TxHandler>::Response: IndexResponse;
impl<
        Chain: TxHandler,
        E: Serialize + Debug,
        I: Serialize + Debug,
        Q: Serialize + Debug,
        M: Serialize + Debug,
    > Deref for AbstractOS<Chain, E, I, Q, M>
where
    <Chain as TxHandler>::Response: IndexResponse,
{
    type Target = Contract<Chain, E, I, Q, M>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<
        Chain: TxHandler,
        E: Serialize + Debug,
        I: Serialize + Debug,
        Q: Serialize + Debug,
        M: Serialize + Debug,
    > DerefMut for AbstractOS<Chain, E, I, Q, M>
where
    <Chain as TxHandler>::Response: IndexResponse,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub mod ans_host;
pub mod manager;
pub mod module_factory;
pub mod os_factory;
pub mod proxy;
pub mod subscription;
// mod terraswap_dapp;
// pub mod balancer;
pub mod dex_api;
pub mod etf;
pub mod ibc_client;
pub mod idea_token;
pub mod osmosis_host;
pub mod tendermint_staking_api;
pub mod version_control;
pub mod vesting;
// pub mod callback_capturer;
