use common::msg::ProxyMemberData;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal};

#[cw_serde]
pub struct InstantiateMsg {
    pub new_member_vote_tokens: Coin,
    pub vote_token_price: Coin,
    pub joining_fee: Coin,
    pub proxy_code_id: u64,
    pub proposal_code_id: u64,
    pub distribution_code_id: u64,
    pub initial_members: Vec<String>,
}

#[cw_serde]
pub struct InstantiationData {
    pub members: Vec<ProxyMemberData>,
}
