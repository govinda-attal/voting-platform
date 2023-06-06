use cosmwasm_std::StdError;
use cw_utils::{ParseReplyError, PaymentError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    ParseError(#[from] ParseReplyError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Unauthorised")]
    Unauthorized,

    #[error("Not enough initial members")]
    NotEnoughInitialMembers,

    #[error("Unrecognized reply id")]
    UnrecognizedReplyId(u64),

    #[error("vote tokens missing on initialization")]
    InitialisationVoteTokensMissing,

    #[error("Less vote tokens on initialization")]
    InitialisationLessVoteTokens,

    #[error("Not enough new member vote tokens")]
    NotEnoughNewMemberVoteTokens,

    #[error("not a proposed member")]
    NotProposedMember,

    #[error("joining fee must {denom}(s)")]
    JoiningFeeDenomInvalid { denom: String },

    #[error("Missing expected data")]
    DataMissing,

    #[error("Cannot propose a member")]
    AlreadyAMember,

    #[error("Not a member")]
    NotAMember,

    #[error("Member proxy mistmatch")]
    MemberProxyMismatch,

    #[error("Existing proposal voting in progress")]
    ExistingProposalInProgress,

    #[error("Cannot vote without vote tokens")]
    VoteRejectedNoVoteTokens,
}
