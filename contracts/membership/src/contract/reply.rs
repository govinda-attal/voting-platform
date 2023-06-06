use std::collections::HashMap;

use common::{
    keys::VOTE_DENOM,
    msg::{ProposalMemberData, ProxyMemberData},
};
use cosmwasm_std::{
    coin, from_binary, to_binary, Addr, DepsMut, Env, Order, Response, StdError, StdResult, SubMsg,
    SubMsgResponse, Uint128, WasmMsg,
};
use cw_utils::parse_instantiate_response_data;

use crate::{
    error::ContractError,
    msg::InstantiationData,
    state::{AWAITING_INITIAL_RESPS, CANDIDATES, CONFIG},
};
use proxy::msg::InstantiateMsg as ProxyInstantiateMsg;

use crate::state::members;

pub fn distribution_instantiated(
    deps: DepsMut,
    env: Env,
    reply: Result<SubMsgResponse, String>,
) -> Result<Response, ContractError> {
    let response = reply.map_err(StdError::generic_err)?;
    let data = response.data.ok_or(ContractError::DataMissing)?;
    let response = parse_instantiate_response_data(&data)?;
    let initial_members: Vec<String> =
        from_binary(&response.data.ok_or(ContractError::DataMissing)?)?;

    let mut config = CONFIG.load(deps.storage)?;
    config.distribution_contract = Addr::unchecked(response.contract_address);
    CONFIG.save(deps.storage, &config)?;

    let balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), VOTE_DENOM)?;

    let vote_tokens_per_member = coin(
        balance.amount.u128() / initial_members.len() as u128,
        VOTE_DENOM,
    );

    let membership_contract = env.contract.address.to_string();
    let msgs: Vec<_> = initial_members
        .into_iter()
        .map(|member| -> Result<_, ContractError> {
            let addr = deps.api.addr_validate(&member)?;
            let init_msg = ProxyInstantiateMsg {
                owner: addr.to_string(),
                distribution_contract: config.distribution_contract.to_string(),
                membership_contract: membership_contract.clone(),
            };
            let msg = WasmMsg::Instantiate {
                admin: Some(membership_contract.clone()),
                code_id: config.proxy_code_id,
                msg: to_binary(&init_msg)?,
                funds: vec![vote_tokens_per_member.clone()],
                label: format!("{} Proxy", addr),
            };
            let msg = SubMsg::reply_on_success(msg, super::INITIAL_PROXY_INSTANTIATION_REPLY_ID);

            Ok(msg)
        })
        .collect::<Result<_, _>>()?;

    AWAITING_INITIAL_RESPS.save(deps.storage, &(msgs.len() as _))?;
    let resp = Response::new().add_submessages(msgs);

    Ok(resp)
}

pub fn initial_proxy_instantiated(
    deps: DepsMut,
    reply: Result<SubMsgResponse, String>,
) -> Result<Response, ContractError> {
    let response = reply.map_err(StdError::generic_err)?;
    let data = response.data.ok_or(ContractError::DataMissing)?;
    let response = parse_instantiate_response_data(&data)?;
    let addr = Addr::unchecked(response.contract_address);

    let owner = proxy::state::OWNER.query(&deps.querier, addr.clone())?;
    members().save(deps.storage, &addr, &owner)?;

    let awaiting = AWAITING_INITIAL_RESPS.load(deps.storage)? - 1;
    if awaiting > 0 {
        AWAITING_INITIAL_RESPS.save(deps.storage, &awaiting)?;

        let resp = Response::new().add_attribute("proxy_addr", addr);
        return Ok(resp);
    }

    let members: Vec<_> = members()
        .range(deps.storage, None, None, Order::Ascending)
        .map(|member| -> StdResult<_> {
            let (member, owner) = member?;
            let data = ProxyMemberData {
                owner_addr: owner.into(),
                proxy_addr: member.into(),
            };
            Ok(data)
        })
        .collect::<StdResult<_>>()?;

    let inst_data = InstantiationData { members };
    let resp = Response::new()
        .add_attribute("proxy addr", addr.as_str())
        .set_data(to_binary(&inst_data)?);

    Ok(resp)
}

pub fn proxy_instantiated(
    deps: DepsMut,
    reply: Result<SubMsgResponse, String>,
) -> Result<Response, ContractError> {
    let response = reply.map_err(StdError::generic_err)?;
    let data = response.data.ok_or(ContractError::DataMissing)?;
    let response = parse_instantiate_response_data(&data)?;

    let proxy_addr = Addr::unchecked(response.contract_address);
    let proxy_owner = proxy::state::OWNER.query(&deps.querier, proxy_addr.clone())?;

    members().save(deps.storage, &proxy_addr, &proxy_owner)?;

    let member_data = ProxyMemberData {
        owner_addr: proxy_owner.to_string(),
        proxy_addr: proxy_addr.to_string(),
    };

    let resp = Response::new()
        .add_attribute("proxy addr", proxy_addr.as_str())
        .set_data(to_binary(&member_data)?);

    Ok(resp)
}

pub fn proposal_instantiated(
    deps: DepsMut,
    reply: Result<SubMsgResponse, String>,
) -> Result<Response, ContractError> {
    let response = reply.map_err(StdError::generic_err)?;
    let data = response.data.ok_or(ContractError::DataMissing)?;
    let response = parse_instantiate_response_data(&data)?;
    let addr = Addr::unchecked(response.contract_address);

    let owner: Addr = proposal::state::OWNER.query(&deps.querier, addr.clone())?;

    // new proposal new candidate
    CANDIDATES.save(deps.storage, &owner, &addr)?;

    let data = ProposalMemberData {
        owner_addr: owner.into(),
        proposal_addr: addr.to_string(),
    };

    let resp = Response::new()
        .add_attribute("proposal addr", addr.as_str())
        .set_data(to_binary(&data)?);

    Ok(resp)
}

pub fn proposal_passed(
    deps: DepsMut,
    env: Env,
    reply: Result<SubMsgResponse, String>,
) -> Result<Response, ContractError> {
    let response = reply.map_err(StdError::generic_err)?;
    let data = response.data.ok_or(ContractError::DataMissing)?;
    let response = parse_instantiate_response_data(&data)?;
    let proposal_addr = Addr::unchecked(response.contract_address);

    let owner: Addr = proposal::state::OWNER.query(&deps.querier, proposal_addr.clone())?;

    let resp = Response::new()
        .add_attribute("proposal_addr", proposal_addr.as_str())
        .add_attribute("owner", owner.as_str());

    Ok(resp)
}
