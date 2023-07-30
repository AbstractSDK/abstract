/// ANCHOR: ans
use crate::{ AbstractSdkResult};
use cosmwasm_std::{Addr, Coin, CosmosMsg, Deps, Timestamp, wasm_execute};
use crate::features::AbstractNameService;

/// Accessor to the Nois client.
/// TODO: query the nois-proxy for prices
pub trait NoisInterface: AbstractNameService + Sized {
    /// Get the Nois proxy address.
    fn nois_proxy_address(&self, deps: Deps) -> AbstractSdkResult<Addr>;

    /// Construct the nois client.
    fn nois_client<'a>(&'a self, deps: Deps<'a>) -> NoisClient<Self> {
        NoisClient {
            _base: self,
            _deps: deps,
            proxy: self.nois_proxy_address(deps).unwrap(),
        }
    }
}

/// The Nois client.
#[derive(Clone)]
pub struct NoisClient<'a, T: NoisInterface> {
    _base: &'a T,
    /// Cw deps.
    _deps: Deps<'a>,
    /// The address of the nois proxy.
    pub proxy: Addr,
}

impl<'a, T: NoisInterface> NoisClient<'a, T> {
    /// Retrieve the address of the Nois proxy
    pub fn proxy(&self) -> &Addr {
        &self.proxy
    }

    /// Request the next randomness from the nois proxy.
    /// The *job_id* is needed to know what randomness we are referring to upon reception in the callback.
    pub fn next_randomness(&self, job_id: impl ToString, funds: Vec<Coin>) -> AbstractSdkResult<CosmosMsg> {
        let msg = wasm_execute(
            self.proxy(),
            // GetNextRandomness requests the randomness from the proxy
            // The job id is needed to know what randomness we are referring to upon reception in the callback
            // In this example, the job_id represents one round of dice rolling.
            &nois::ProxyExecuteMsg::GetNextRandomness { job_id: job_id.to_string() },
            //In this example the randomness is sent from the gambler, but you may also send the funds from the contract balance
           funds,
        )?.into();

        Ok(msg)
    }

    /// Request the next randomness after a given timestamp.
    /// The *job_id* is needed to know what randomness we are referring to upon reception in the callback.
    pub fn randomness_after(&self, job_id: impl ToString, after: Timestamp, funds: Vec<Coin>) -> AbstractSdkResult<CosmosMsg> {
        let msg = wasm_execute(
            self.proxy(),
            //GetNextRandomnessAfter requests the randomness from the proxy after a given timestamp
            //The job id is needed to know what randomness we are referring to upon reception in the callback
            //In this example, the job_id represents one round of dice rolling.
            &nois::ProxyExecuteMsg::GetRandomnessAfter { after, job_id: job_id.to_string() },
            //In this example the randomness is sent from the gambler, but you may also send the funds from the contract balance
            funds,
        )?.into();

        Ok(msg)
    }

    /// Parse the randmess from a callback into a 32 byte array.
    /// Check out the means to leverage the parsed randomness in the [nois] crate.
    pub fn parse_randomness(&self, randomness: cosmwasm_std::HexBinary) -> AbstractSdkResult<[u8; 32]> {
        Ok(randomness
            .to_array()?)
    }

}
