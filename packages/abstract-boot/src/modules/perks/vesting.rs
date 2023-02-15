use abstract_os::vesting::*;
use boot_core::{prelude::boot_contract, BootEnvironment, Contract};
use cosmwasm_std::Empty;

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct Vesting<Chain>;

impl<Chain: BootEnvironment> Vesting<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("cw20_vesting"), // .with_mock(Box::new(
                                                                       //     ContractWrapper::new_with_empty(
                                                                       //         ::contract::execute,
                                                                       //         ::contract::instantiate,
                                                                       //         ::contract::query,
                                                                       //     ),
                                                                       // ))
        )
    }

    // pub  fn set_vault_assets<C: Signing + Context>(
    //     &self,
    //     sender: &Sender<C>,
    //     path: &str,
    // ) -> Result<(), crate::AbstractBootError> {
    //     let file = File::open(path).expect(&format!("file should be present at {}", path));
    //     let json: serde_json::Value = from_reader(file)?;
    //     let maybe_assets = json.get(self.instance().deployment.network.chain_id.clone());

    // }
}
