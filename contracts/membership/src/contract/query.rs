use cosmwasm_std::{Addr, Deps, StdResult};

use crate::state::members;
use common::msg::membership::IsMemberResp;

pub fn is_member(deps: Deps, addr: String) -> StdResult<IsMemberResp> {
    let is_member = members().has(deps.storage, &Addr::unchecked(addr));

    Ok(IsMemberResp { is_member })
}
