use crate::cw_helpers::wasm_smart_query;
use crate::features::AbstractNameService;
use crate::AbstractSdkResult;
use cosmwasm_std::{wasm_execute, Addr, Coin, CosmosMsg, Deps, Timestamp};

/// Accessor to the Nois client.
/// TODO: query the nois-proxy for prices
pub trait NoisInterface: AbstractNameService + Sized {
    /// Get the Nois proxy address.
    fn nois_proxy_address(&self, deps: Deps) -> AbstractSdkResult<Addr>;

    /// Construct the nois client.
    fn nois<'a>(&'a self, deps: Deps<'a>) -> AbstractSdkResult<NoisClient<Self>> {
        Ok(NoisClient {
            _base: self,
            deps: deps,
            proxy: self.nois_proxy_address(deps)?,
        })
    }
}

/// The Nois client.
#[derive(Clone)]
pub struct NoisClient<'a, T: NoisInterface> {
    _base: &'a T,
    /// Cw deps.
    deps: Deps<'a>,
    /// The address of the nois proxy.
    pub proxy: Addr,
}

// enum PaymentOption {
//     FromCaller {
//         funds: Vec<Coin>,
//     },
//     FromModule,
//     FromProxy {
//         env: cosmwasm_std::Env,
//     }
// }

impl<'a, T: NoisInterface> NoisClient<'a, T> {
    /// Retrieve the address of the Nois proxy
    pub fn proxy(&self) -> &Addr {
        &self.proxy
    }

    /// Retrieve the prices from the nois proxy.
    pub fn prices(&self) -> AbstractSdkResult<Vec<Coin>> {
        let query = wasm_smart_query(self.proxy(), &nois_proxy::msg::QueryMsg::Prices {})?;
        let resp: nois_proxy::msg::PricesResponse = self.deps.querier.query(&query)?;
        Ok(resp.prices)
    }

    /// Request the next randomness from the nois proxy.
    /// The *job_id* is needed to know what randomness we are referring to upon reception in the callback.
    pub fn next_randomness(
        &self,
        job_id: impl ToString,
        funds: Vec<Coin>,
    ) -> AbstractSdkResult<Vec<CosmosMsg>> {
        let prices = self.prices()?;
        // check that the funds that they sent match one of the assets in prices and is at least as much
        // as the price
        for Coin { denom, amount } in prices.iter() {
            for Coin {
                denom: fund_denom,
                amount: fund_amount,
            } in funds.iter()
            {
                if denom == fund_denom && fund_amount < amount {
                    return Err(cosmwasm_std::StdError::generic_err(format!(
                        "Insufficient funds. {} is less than {}",
                        fund_amount, amount
                    ))
                    .into());
                }
            }
        }

        let msg = wasm_execute(
            self.proxy(),
            // GetNextRandomness requests the randomness from the proxy
            // The job id is needed to know what randomness we are referring to upon reception in the callback
            &nois::ProxyExecuteMsg::GetNextRandomness {
                job_id: job_id.to_string(),
            },
            //In this example the randomness is sent from the gambler, but you may also send the funds from the contract balance
            funds,
        )?
        .into();

        Ok(vec![msg])
    }

    /// Request the next randomness after a given timestamp.
    /// The *job_id* is needed to know what randomness we are referring to upon reception in the callback.
    pub fn randomness_after(
        &self,
        job_id: impl ToString,
        after: Timestamp,
        funds: Vec<Coin>,
    ) -> AbstractSdkResult<Vec<CosmosMsg>> {
        let msg = wasm_execute(
            self.proxy(),
            // GetNextRandomnessAfter requests the randomness from the proxy after a given timestamp
            &nois::ProxyExecuteMsg::GetRandomnessAfter {
                after,
                job_id: job_id.to_string(),
            },
            //In this example the randomness is sent from the gambler, but you may also send the funds from the contract balance
            funds,
        )?
        .into();

        Ok(vec![msg])
    }

    /// Parse the randmess from a callback into a 32 byte array.
    /// Check out the means to leverage the parsed randomness in the [nois] crate.
    pub fn parse_randomness(
        &self,
        randomness: cosmwasm_std::HexBinary,
    ) -> AbstractSdkResult<[u8; 32]> {
        Ok(randomness.to_array()?)
    }
}
