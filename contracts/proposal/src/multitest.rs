use anyhow::{Ok, Result as AnyResult};
use common::msg::{ProposalMemberData, ProxyMemberData};
use cosmwasm_std::{from_binary, Addr, Coin, Decimal};
use cw_multi_test::{App, ContractWrapper, Executor};
use cw_utils::parse_execute_response_data;

use crate::msg::{ExecMsg, InstantiateMsg};
use crate::{execute, instantiate, query, reply};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct CodeId(u64);

impl CodeId {
    pub fn store_code(app: &mut App) -> Self {
        let contract = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
        CodeId(app.store_code(Box::new(contract)))
    }
}

impl From<CodeId> for u64 {
    fn from(value: CodeId) -> Self {
        value.0
    }
}

#[derive(Debug)]
pub struct Contract(Addr);

impl Contract {
    pub fn from_addr(addr: Addr) -> Self {
        Self(addr)
    }

    pub fn addr(&self) -> &Addr {
        &self.0
    }

    #[track_caller]
    pub fn join(
        &self,
        app: &mut App,
        sender: &Addr,
        funds: &[Coin],
    ) -> AnyResult<Option<ProxyMemberData>> {
        let msg = ExecMsg::Join {};
        let resp = app.execute_contract(sender.clone(), self.addr().clone(), &msg, &funds)?;
        resp.data
            .map(|data| parse_execute_response_data(&data))
            .transpose()?
            .and_then(|data| data.data)
            .map(|data| from_binary(&data))
            .transpose()
            .map_err(Into::into)
    }

    #[track_caller]
    pub fn vote(&self, app: &mut App, sender: &Addr, funds: &[Coin]) -> AnyResult<()> {
        let msg = ExecMsg::Vote {};
        app.execute_contract(sender.clone(), self.addr().clone(), &msg, funds)?;
        Ok(())
    }
}
