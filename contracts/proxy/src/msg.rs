use common::msg::WithdrawableResp;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub distribution_contract: String,
    pub membership_contract: String,
}

#[cw_serde]
pub enum ExecMsg {
    ProposeMember { addr: String },

    BuyVoteTokens {},

    Withdraw {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(WithdrawableResp)]
    Withdrawable {},
}
