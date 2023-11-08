use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_interface::Abstract;
use cw_orch::{deploy::Deploy, prelude::CwEnv};

use crate::{
    account::AccountBuilder,
    publisher::{Publisher, PublisherBuilder},
};

pub struct AbstractClient<Chain: CwEnv> {
    abstr: Abstract<Chain>,
}

// TODO: Handle errors later.
impl<Chain: CwEnv> AbstractClient<Chain> {
    pub fn new(chain: Chain) -> Self {
        let abstr = Abstract::load_from(chain).unwrap();
        Self { abstr }
    }

    // TODO: Switch to builder later.
    pub fn existing_publisher(&self, namespace: String) -> Publisher<Chain> {
        Publisher::new_existing_publisher(&self.abstr, namespace)
    }

    pub fn new_publisher(
        &self,
        governance_details: GovernanceDetails<String>,
    ) -> PublisherBuilder<Chain> {
        PublisherBuilder::new(AccountBuilder::new(&self.abstr, governance_details))
    }

    pub fn new_account(
        &self,
        governance_details: GovernanceDetails<String>,
    ) -> AccountBuilder<Chain> {
        AccountBuilder::new(&self.abstr, governance_details)
    }
}
