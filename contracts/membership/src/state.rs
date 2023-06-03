use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Empty};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, UniqueIndex};

#[cw_serde]
pub struct Config {
    pub proxy_code_id: u64,
    pub proposal_code_id: u64,
    pub distribution_contract: Addr,
    pub joining_fee: Coin,
}

pub struct MembersIndexes<'a> {
    pub owner: UniqueIndex<'a, Addr, Addr, Addr>,
}

impl<'a> IndexList<Addr> for MembersIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Addr>> + '_> {
        let v: [&dyn Index<Addr>; 1] = [&self.owner];
        Box::new(v.into_iter())
    }
}

// proxy => owner
//
// secondary indexes:
// * owner
pub fn members() -> IndexedMap<'static, &'static Addr, Addr, MembersIndexes<'static>> {
    let indexes = MembersIndexes {
        owner: UniqueIndex::new(|owner| owner.clone(), "members__owner"),
    };
    IndexedMap::new("members", indexes)
}

pub const CONFIG: Item<Config> = Item::new("config");
// (candidate-addr, proposal-addr)
pub const CANDIDATES: Map<&Addr, Addr> = Map::new("candidates");

pub const AWAITING_INITIAL_RESPS: Item<u64> = Item::new("awaiting_initial_resps");
