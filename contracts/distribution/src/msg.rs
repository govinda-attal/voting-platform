use cosmwasm_schema::cw_serde;
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
    BuyVoteTokens {},
    DistributeJoiningFee {},
}
