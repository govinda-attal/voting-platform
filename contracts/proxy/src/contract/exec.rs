use common::keys::VOTE_DENOM;
use common::msg::membership::ExecMsg as MembershipExecMsg;
use cosmwasm_std::{
    coins, ensure, to_binary, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg, Uint128,
    WasmMsg,
};
use cw_utils::must_pay;

use crate::contract::PROPOSE_MEMBER_REPLY_ID;
use crate::error::ContractError;
use crate::state::{CONFIG, OWNER};

pub fn propose_member(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    let vote_tokens = must_pay(&info, VOTE_DENOM)?;

    let owner = OWNER.load(deps.storage)?;
    ensure!(owner == info.sender, ContractError::Unauthorized);

    let config = CONFIG.load(deps.storage)?;

    let propose_msg = MembershipExecMsg::ProposeMember { addr: addr.clone() };
    let propose_msg = WasmMsg::Execute {
        contract_addr: config.membership_contract.into_string(),
        msg: to_binary(&propose_msg)?,
        funds: coins(vote_tokens.u128(), VOTE_DENOM),
    };

    let propose_msg = SubMsg::reply_on_success(propose_msg, PROPOSE_MEMBER_REPLY_ID);

    let resp = Response::new()
        .add_submessage(propose_msg)
        .add_attribute("action", "propose member")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("member", addr);

    Ok(resp)
}
