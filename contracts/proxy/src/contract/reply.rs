use common::keys::{ATOM, VOTE_DENOM};
use cosmwasm_std::{BankMsg, DepsMut, Env, Response, StdError, SubMsgResponse};

use crate::{
    error::ContractError,
    state::{CONFIG, OWNER},
};

pub fn propose_member(reply: Result<SubMsgResponse, String>) -> Result<Response, ContractError> {
    let response = reply.map_err(StdError::generic_err)?;
    if let Some(data) = response.data {
        let resp = Response::new().set_data(data);
        Ok(resp)
    } else {
        Ok(Response::new())
    }
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    _reply: Result<SubMsgResponse, String>,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    let total_amount = deps.querier.query_balance(env.contract.address, ATOM)?;

    let bank_msg = BankMsg::Send {
        to_address: owner.into_string(),
        amount: vec![total_amount.clone()],
    };

    let resp = Response::new()
        .add_message(bank_msg)
        .add_attribute("amount", total_amount.to_string());

    Ok(resp)
}

pub fn buy_vote_tokens(
    deps: DepsMut,
    env: Env,
    _reply: Result<SubMsgResponse, String>,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    let vote_tokens = deps
        .querier
        .query_balance(env.contract.address, VOTE_DENOM)?;

    let bank_msg = BankMsg::Send {
        to_address: owner.into_string(),
        amount: vec![vote_tokens.clone()],
    };

    let resp = Response::new()
        .add_message(bank_msg)
        .add_attribute("amount", vote_tokens.to_string());

    Ok(resp)
}
