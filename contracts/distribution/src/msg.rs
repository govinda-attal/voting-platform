use common::msg::WithdrawableResp;
use std::collections::HashMap;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Coin};

#[cw_serde]
pub struct InstantiateMsg {
    pub new_member_vote_tokens: Coin,
    pub vote_token_price: Coin,
    pub total_vote_tokens_in_circulation: Coin,
    pub data: Binary,
}

#[cw_serde]
pub enum ExecMsg {
    DistributeJoiningFee { voter_tokens: HashMap<String, Coin> },
    BuyVoteTokens {},
    Withdraw {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(WithdrawableResp)]
    Withdrawable { proxy: String },
}
