use cosmwasm_std::{Reply, StdError};

use crate::contract::{ProxyResponse, ProxyResult};

/// Add the message's data to the response
pub fn forward_response_data(result: Reply) -> ProxyResult {
    // get the result from the reply
    let res = result.result.into_result().map_err(StdError::generic_err)?;

    // log and add data if needed
    let resp = if let Some(data) = res.data {
        ProxyResponse::new(
            "forward_response_data_reply",
            vec![("response_data", "true")],
        )
        .set_data(data)
    } else {
        ProxyResponse::new(
            "forward_response_data_reply",
            vec![("response_data", "false")],
        )
    };

    Ok(resp)
}
