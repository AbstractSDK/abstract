//! Structured IBC protocol logic to simplify the invocations of IBC protocols.
use cosmwasm_schema::cw_serde;


pub struct ICS_999;

#[cw_serde]
pub enum IbcState {
    INIT,
    HANDSHAKE,
    OPEN,
    CLOSE,
    ERROR,
}


pub trait IBC_Protocol {
    
}