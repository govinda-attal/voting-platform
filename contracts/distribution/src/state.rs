use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
#[derive(Default)]
pub struct Correction {
    pub points_balance: Uint128,
}

#[cw_serde]
#[derive(Default)]
pub struct MemberData {
    pub reward_balance: Coin,
    pub points_balance: Uint128,
}

impl MemberData {
    pub fn with_reward_balance(mut self, bal: Coin) -> Self {
        self.reward_balance = bal;
        self
    }
}

#[cw_serde]
pub struct Config {
    pub membership_contract: Addr,
    pub vote_token_price: Coin,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const TOTAL_VOTE_TOKENS_IN_CIRCULATION: Item<Coin> =
    Item::new("total_vote_tokens_in_circulation");

pub const CORRECTION: Item<Correction> = Item::new("correction");
pub const MEMBER_DATA: Map<&Addr, MemberData> = Map::new("member_data");
