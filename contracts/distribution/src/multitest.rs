use cosmwasm_std::Addr;
use cw_multi_test::{App, ContractWrapper};

use crate::{execute, instantiate, query};

pub struct CodeId(u64);

impl CodeId {
    pub fn store_code(app: &mut App) -> Self {
        let contract = ContractWrapper::new(execute, instantiate, query);
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
}
