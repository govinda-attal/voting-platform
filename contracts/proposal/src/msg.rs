use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

#[cw_serde]
pub struct InstantiateMsg {
    pub proposed_owner: String,
    pub distribution_contract: String,
    pub membership_contract: String,
    pub joining_fee: Coin,
}

#[cw_serde]
pub enum ExecMsg {
    Pass {},
    Vote {},
    Join {},
}
