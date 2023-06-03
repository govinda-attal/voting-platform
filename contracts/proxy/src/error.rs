use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    StdError(#[from] StdError),
    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized,
    #[error("{0}")]
    UnrecognizedReplyId(u64),
}
