use crate::{MockQuerierBuilder, TEST_VERSION_CONTROL};
use abstract_os::objects::common_namespace::ADMIN_NAMESPACE;
use abstract_os::objects::core::OS_ID;
use abstract_os::version_control::state::OS_ADDRESSES;
use abstract_os::version_control::Core;
use cosmwasm_std::testing::MockQuerier;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;


pub struct AbstractMockQuerierBuilder {
    builder: MockQuerierBuilder,
    version_control: &'static str,
}

impl Default for AbstractMockQuerierBuilder {
    fn default() -> Self {
        Self {
            builder: MockQuerierBuilder::default(),
            version_control: TEST_VERSION_CONTROL,
        }
    }
}

impl AbstractMockQuerierBuilder {
    /// Mock the existence of an OS by setting the OS id for the proxy and manager along with registering the os to version control.
    pub fn os(mut self, manager: &str, proxy: &str, os_id: u32) -> Self {
        self.builder = self
            .builder
            .with_contract_item(proxy, OS_ID, &os_id)
            .with_contract_item(manager, OS_ID, &os_id)
            .with_contract_item(
                proxy,
                Item::new(ADMIN_NAMESPACE),
                &Some(Addr::unchecked(manager)),
            )
            .with_contract_map_entry(
                self.version_control,
                OS_ADDRESSES,
                (
                    os_id,
                    &Core {
                        manager: Addr::unchecked(manager),
                        proxy: Addr::unchecked(proxy),
                    },
                ),
            );

        self
    }

    /// Change the version control address. Any os added after this will be registered to this address.
    pub fn set_version_control(mut self, version_control: &'static str) -> Self {
        self.version_control = version_control;
        self
    }

    pub fn builder(self) -> MockQuerierBuilder {
        self.builder
    }

    pub fn build(self) -> MockQuerier {
        self.builder.build()
    }
}
