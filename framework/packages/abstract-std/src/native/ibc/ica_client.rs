pub mod state {
    use cosmwasm_std::Addr;
    use cw_storage_plus::Map;

    use crate::objects::{storage_namespaces, TruncatedChainId};

    /// Information about the deployed infrastructure we're connected to.
    #[cosmwasm_schema::cw_serde]
    pub struct IcaInfrastructure {
        /// Address of the polytone note deployed on the local chain. This contract will forward the messages for us.
        pub polytone_note: Addr,
    }

    pub const ICA_INFRA: Map<&TruncatedChainId, IcaInfrastructure> =
        Map::new(storage_namespaces::ica_client::ICA_INFRA);
}
