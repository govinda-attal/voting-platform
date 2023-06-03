use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;

pub const MEMBERSHIP: Item<Addr> = Item::new("membership");
pub const NEW_MEMBER_VOTE_TOKENS: Item<Coin> = Item::new("new_member_vote_tokens");
pub const VOTE_TOKEN_PRICE: Item<Coin> = Item::new("vote_token_price");
pub const TOTAL_VOTE_TOKENS_IN_CIRCULATION: Item<Coin> =
    Item::new("total_vote_tokens_in_circulation");
