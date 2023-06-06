use crate::{
    error::ContractError,
    state::{candidates, members},
};
use common::msg::membership::{IsMemberResp, IsProposedMemberResp, OwnerProxyResp};
use cosmwasm_std::{Addr, Deps, Order, StdError, StdResult};
use cw_storage_plus::Prefixer;
use std::str;

pub fn is_member(deps: Deps, addr: String) -> StdResult<IsMemberResp> {
    let addr = deps.api.addr_validate(&addr)?;
    let ok = members().has(deps.storage, &addr);

    Ok(IsMemberResp { ok })
}

pub fn is_proposed_member(deps: Deps, addr: String) -> StdResult<IsProposedMemberResp> {
    let addr = deps.api.addr_validate(&addr)?;
    let ok = candidates().has(deps.storage, &addr);

    Ok(IsProposedMemberResp { ok })
}

pub fn owner_proxy(deps: Deps, owner: String) -> StdResult<OwnerProxyResp> {
    let owner = deps.api.addr_validate(&owner)?;
    let (pk, sk) = members()
        .idx
        .owner
        .item(deps.storage, owner.clone())?
        .ok_or(StdError::generic_err("not an owner"))?;

    let pk = str::from_utf8(&pk).unwrap();

    Ok(OwnerProxyResp {
        owner: owner.into(),
        proxy: pk.to_string(),
    })
}
