// https://github.com/cosmos/cosmos-sdk/blob/101a63941559705f8e0ddcc10811b8363acdb27a/proto/cosmos/gov/v1/gov.proto#L31

use cosmos_sdk_proto::cosmos::gov::v1beta1::VoteOption;

pub fn vote_to_option(vote: cosmwasm_std::VoteOption) -> i32 {
    match vote {
        cosmwasm_std::VoteOption::Yes => VoteOption::Yes.into(),
        cosmwasm_std::VoteOption::No => VoteOption::No.into(),
        cosmwasm_std::VoteOption::Abstain => VoteOption::Abstain.into(),
        cosmwasm_std::VoteOption::NoWithVeto => VoteOption::NoWithVeto.into(),
    }
}
