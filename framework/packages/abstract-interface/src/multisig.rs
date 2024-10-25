use cosmrs::tx::Msg;
use cosmwasm_std::from_json;
use cw_orch::{contract::Contract, prelude::*};
use cw_plus_orch::cw3_flex_multisig::{self, Cw3FlexMultisig};
use prost::{Message, Name};

use crate::{Abstract, AbstractInterfaceError};

impl<T: CwEnv + Stargate> Abstract<T> {
    pub fn update_admin_to_cw3_flex(
        &self,
        cw3_flex_contract: Cw3FlexMultisig<T>,
        extra_contracts: impl IntoIterator<Item = Contract<T>>,
    ) -> Result<(), AbstractInterfaceError> {
        // Make sure we have cw3-flex
        let chain = self.registry.environment().clone();
        let cw3_flex_address = cw3_flex_contract.address()?;
        let cw2_of_cw3: cw2::ContractVersion = cosmwasm_std::from_json(
            chain
                .wasm_querier()
                .raw_query(&cw3_flex_address, cw2::CONTRACT.as_slice().to_vec())
                .map_err(Into::into)?,
        )?;
        if !cw2_of_cw3.contract.contains("cw3-flex-multisig") {
            return Err(AbstractInterfaceError::Multisig(
                "cw3-flex-multisig contract info missmatch".to_string(),
            ));
        }

        for contract in self
            .contracts()
            .into_iter()
            .map(|(contract, _version)| contract.clone())
            .chain(extra_contracts)
        {
            chain
                .commit_any::<cosmrs::proto::cosmwasm::wasm::v1::MsgUpdateAdminResponse>(
                    vec![prost_types::Any {
                        value: cosmrs::proto::cosmwasm::wasm::v1::MsgUpdateAdmin {
                            sender: chain.sender_addr().to_string(),
                            new_admin: cw3_flex_address.to_string(),
                            contract: contract.address()?.to_string(),
                        }
                        .encode_to_vec(),
                        type_url: cosmrs::proto::cosmwasm::wasm::v1::MsgUpdateAdmin::type_url(),
                    }],
                    None,
                )
                .map_err(Into::into)?;
        }
        Ok(())
    }
}
