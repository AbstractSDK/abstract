

pub trait TransferInterface<'a>: AbstractNameService + Sized {
    fn applications(&self)-> Applications<'a> {
        Applications { base: self }
    }
}

impl<'a, T> TransferInterface<'a> for T
    where T: OsAddress + Sized
{}

pub struct Applications <'a> {
    base: &'a dyn OsAddress
}

impl<'a> Applications<'a> {
    /// Construct an API request message.
    pub fn api_request<T: Serialize>(
        api_address: impl Into<String>,
        message: impl Into<ExecuteMsg<T, Empty>>,
        funds: Vec<Coin>,
    ) -> StdResult<CosmosMsg> {
        let api_msg: ExecuteMsg<T, Empty> = message.into();
        Ok(wasm_execute(api_address, &api_msg, funds)?.into())
    }
    
    /// Construct an API configure message
    pub fn configure_api(
        api_address: impl Into<String>,
        message: BaseExecuteMsg,
    ) -> StdResult<CosmosMsg> {
        let api_msg: ExecuteMsg<Empty, Empty> = message.into();
        Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
    }
    
    pub fn api_init_msg(ans_host_address: &Addr, version_control_address: &Addr) -> StdResult<Binary> {
        to_binary(&BaseInstantiateMsg {
            ans_host_address: ans_host_address.to_string(),
            version_control_address: version_control_address.to_string(),
        })
    }    
