use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_gov_ownable::{cw_ownable_execute, cw_ownable_query, GovAction};

#[cw_ownable_execute]
#[cw_serde]
enum ExecuteMsg {
    Foo,
    Bar(u64),
    Fuzz { buzz: String },
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
enum QueryMsg {
    #[returns(String)]
    Foo,

    #[returns(String)]
    Bar(u64),

    #[returns(String)]
    Fuzz { buzz: String },
}

#[test]
fn derive_execute_variants() {
    let msg = ExecuteMsg::Foo;

    // If this compiles we have won.
    match msg {
        ExecuteMsg::UpdateOwnership(GovAction::TransferOwnership {
            new_owner: _,
            expiry: _,
        })
        | ExecuteMsg::UpdateOwnership(GovAction::AcceptOwnership)
        | ExecuteMsg::UpdateOwnership(GovAction::RenounceOwnership)
        | ExecuteMsg::Foo
        | ExecuteMsg::Bar(_)
        | ExecuteMsg::Fuzz { .. } => "yay",
    };
}

#[test]
fn derive_query_variants() {
    let msg = QueryMsg::Foo;

    // If this compiles we have won.
    match msg {
        QueryMsg::Ownership {} | QueryMsg::Foo | QueryMsg::Bar(_) | QueryMsg::Fuzz { .. } => "yay",
    };
}
