use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized for operation")]
    Unauthorized,

    #[error("{0}")]
    PaymentError(#[from] PaymentError),
}
