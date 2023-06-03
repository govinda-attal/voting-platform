use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub distribution_contract: String,
    pub membership_contract: String,
}

#[cw_serde]
pub enum ExecutionMsg {
    ProposeMember { addr: String },

    BuyVoteTokens {},
}
