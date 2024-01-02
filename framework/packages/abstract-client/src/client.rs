use abstract_interface::ManagerQueryFns;
use abstract_interface::{Abstract, AnsHost, VersionControl};
use cosmwasm_std::{Addr, BlockInfo, Coin, Uint128};
use cw_orch::{deploy::Deploy, environment::MutCwEnv, prelude::CwEnv};

use crate::{
    account::{Account, AccountBuilder},
    error::AbstractClientError,
    infrastructure::Environment,
    publisher::{Publisher, PublisherBuilder},
};

pub struct AbstractClient<Chain: CwEnv> {
    pub(crate) abstr: Abstract<Chain>,
}

pub type AbstractClientResult<T> = Result<T, AbstractClientError>;

impl<Chain: CwEnv> AbstractClient<Chain> {
    pub fn new(chain: Chain) -> AbstractClientResult<Self> {
        let abstr = Abstract::load_from(chain)?;
        Ok(Self { abstr })
    }

    pub fn name_service(&self) -> &AnsHost<Chain> {
        &self.abstr.ans_host
    }

    pub fn version_control(&self) -> &VersionControl<Chain> {
        &self.abstr.version_control
    }

    pub fn block_info(&self) -> AbstractClientResult<BlockInfo> {
        self.environment()
            .block_info()
            .map_err(Into::<cw_orch::prelude::CwOrchError>::into)
            .map_err(Into::<AbstractClientError>::into)
    }

    pub fn get_publisher_from_namespace(
        &self,
        namespace: &str,
    ) -> AbstractClientResult<Publisher<Chain>> {
        Ok(Publisher::new(self.get_account_from_namespace(namespace)?))
    }

    pub fn publisher_builder(&self) -> PublisherBuilder<Chain> {
        PublisherBuilder::new(AccountBuilder::new(&self.abstr))
    }

    pub fn account_builder(&self) -> AccountBuilder<Chain> {
        AccountBuilder::new(&self.abstr)
    }

    pub fn get_account_from_namespace(
        &self,
        namespace: &str,
    ) -> AbstractClientResult<Account<Chain>> {
        Account::from_namespace(&self.abstr, namespace)
    }

    pub fn sender(&self) -> Addr {
        self.environment().sender()
    }

    pub fn query_balance(
        &self,
        address: &Addr,
        denom: impl Into<String>,
    ) -> AbstractClientResult<Uint128> {
        let coins = self
            .environment()
            .balance(address, Some(denom.into()))
            .map_err(Into::into)?;
        // There will always be a single element in this case.
        Ok(coins[0].amount)
    }

    pub fn query_balances(&self, address: &Addr) -> AbstractClientResult<Vec<Coin>> {
        self.environment()
            .balance(address, None)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    pub fn wait_blocks(&self, amount: u64) -> AbstractClientResult<()> {
        self.environment()
            .wait_blocks(amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    pub fn wait_seconds(&self, amount: u64) -> AbstractClientResult<()> {
        self.environment()
            .wait_seconds(amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    pub fn next_block(&self) -> AbstractClientResult<()> {
        self.environment()
            .next_block()
            .map_err(Into::into)
            .map_err(Into::into)
    }
}

impl<Chain: MutCwEnv> AbstractClient<Chain> {
    pub fn set_balance(&self, address: &Addr, amount: Vec<Coin>) -> AbstractClientResult<()> {
        self.environment()
            .set_balance(address, amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    pub fn add_balance(&self, address: &Addr, amount: Vec<Coin>) -> AbstractClientResult<()> {
        self.environment()
            .add_balance(address, amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }
}

pub mod daemon {
    use abstract_core::{objects::AccountId, MANAGER};
    use abstract_interface::AbstractAccount;
    use cw_orch::{
        daemon::Daemon,
        environment::TxHandler as _,
        state::{ChainState, StateInterface},
    };

    use super::*;

    impl AbstractClient<Daemon> {
        /// Retrieve the last account created by the client.
        /// Returns `None` if no account has been created yet.
        pub fn get_last_account(&self) -> AbstractClientResult<Option<Account<Daemon>>> {
            let addresses = self.environment().state().get_all_addresses()?;

            // Now search for all the keys that start with "abstract:manager-x" and return the one which has the highest x.
            let mut last_account: Option<(u32, Account<Daemon>)> = None;
            for (id, _) in addresses {
                if !id.starts_with(MANAGER) {
                    continue;
                }

                let account_id_str = id.rsplitn(2, '-').next().unwrap();
                let account_id = AccountId::try_from(account_id_str)?;

                // Only take local accounts into account.
                if account_id.is_remote() {
                    continue;
                }

                // only take accounts that the current sender owns
                let account = AbstractAccount::new(&self.abstr, account_id.clone());
                if account.manager.ownership()?.owner
                    != Some(self.environment().sender().to_string())
                {
                    continue;
                }

                if let Some((last_account_id, _)) = last_account {
                    if account_id.seq() > last_account_id {
                        last_account = Some((account_id.seq(), Account::new(account)));
                    }
                } else {
                    last_account = Some((account_id.seq(), Account::new(account)));
                }
            }
            Ok(last_account.map(|(_, account)| account))
        }
    }
}
