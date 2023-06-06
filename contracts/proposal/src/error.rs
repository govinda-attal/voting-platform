use cosmwasm_std::{Coin, StdError};
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

    #[error("Vote rejected as proposal was passed earlier")]
    VoteRejectedProposalWasPassedEarlier,

    #[error("Pay joining fee {fee}")]
    JoinRejected { fee: Coin },
}
