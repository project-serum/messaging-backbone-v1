//! Error types

use num_derive::FromPrimitive;
use solana_sdk::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum MessageError {
    /// Message instruction is too large
    #[error("Instruction size too large")]
    MessageTooLarge,
    /// Message instruction is too small
    #[error("Instruction size too small")]
    MessageTooSmall,
    /// Message size is greater than max
    #[error("Message length too large for instruction")]
    MessageLengthTooLarge,
    /// Message size is too small (nonzero data contained after length ends)
    #[error("Message length too small")]
    MessageLengthTooSmall,
    /// Message contains no data
    #[error("Message is empty")]
    MessageEmpty,
    /// Message queue buffer invalid size
    #[error("Message queue account is the wrong size")]
    MessageQueueAccountWrongSize,
    /// Message queue head invalid
    #[error("Message queue head is invalid")]
    MessageQueueBad,
    /// Signer and queue accounts not passed
    #[error("Invocation expects [signer, queue] accounts to get passed")]
    NoAccountsPassed,
    /// Queue account not passed
    #[error("Queue account not passed")]
    QueueNotPassed,
    /// Extra accounts passed
    #[error("Extra accounts passed")]
    ExtraAccountsPassed,
    /// Sender did not sign
    #[error("Sender must have signed the transaction")]
    SenderDidNotSign,
}

impl From<MessageError> for ProgramError {
    fn from(e: MessageError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for MessageError {
    fn type_of() -> &'static str {
        "MessageError"
    }
}
