use cosmwasm_std::Reply;

use crate::contract::{ProxyResponse, ProxyResult};

/// Add the message's data to the response
pub fn forward_response_data(result: Reply) -> ProxyResult {
    // get the result from the reply
    let resp = cw_utils::parse_reply_execute_data(result)?;

    // log and add data if needed
    let resp = if let Some(data) = resp.data {
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
