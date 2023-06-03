use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub distribution_contract: Addr,
    pub membership_contract: Addr,
    pub joining_fee: Coin,
}

pub const OWNER: Item<Addr> = Item::new("owner");
pub const CONFIG: Item<Config> = Item::new("config");
pub const IS_PASSED: Item<bool> = Item::new("is_passed");
