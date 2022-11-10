use crate::AbstractOS;
use abstract_os::vesting::*;
use boot_core::{Contract, IndexResponse, TxHandler, TxResponse};
use cosmwasm_std::Empty;

pub type Vesting<Chain> = AbstractOS<Chain, ExecuteMsg, InstantiateMsg, QueryMsg, Empty>;

impl<Chain: TxHandler + Clone> Vesting<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: &Chain) -> Self {
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
    // ) -> Result<(), BootError> {
    //     let file = File::open(path).expect(&format!("file should be present at {}", path));
    //     let json: serde_json::Value = from_reader(file)?;
    //     let maybe_assets = json.get(self.instance().deployment.network.chain_id.clone());

    // }
}
