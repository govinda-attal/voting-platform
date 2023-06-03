#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use error::ContractError;
use msg::InstantiateMsg;

mod contract;
pub mod error;
pub mod msg;
pub mod state;

#[cfg(any(feature = "mt", test))]
pub mod multitest;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    contract::instantiate(deps, env, info, msg)
}
