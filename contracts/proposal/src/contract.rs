use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response};

use crate::{
    error::ContractError,
    msg::ExecMsg,
    msg::InstantiateMsg,
    state::{Config, CONFIG, OWNER},
};

mod exec;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let owner = deps.api.addr_validate(&msg.proposed_owner)?;

    // addresses are trusted as they come from membership contract
    let distribution_contract = Addr::unchecked(msg.distribution_contract);
    let membership_contract = Addr::unchecked(&msg.membership_contract);

    OWNER.save(deps.storage, &owner)?;

    CONFIG.save(
        deps.storage,
        &Config {
            distribution_contract,
            membership_contract,
            joining_fee: msg.joining_fee,
        },
    )?;

    let resp = Response::new()
        .add_attribute("action", "new_proposal")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("owner", owner.as_str());
    Ok(resp)
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecMsg,
) -> Result<Response, ContractError> {
    use ExecMsg::*;

    match msg {
        Pass {} => exec::pass(deps, env, info),
        Vote {} => exec::vote(deps, env, info),
        Join {} => exec::join(deps, env, info),
    }
}
