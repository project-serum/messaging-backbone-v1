//! Program entrypoint

#![cfg(feature = "program")]
#![cfg(not(feature = "no-entrypoint"))]

use crate::buffer::{Message, MessageBuffer};
use crate::error::MessageError;
use solana_sdk::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

entrypoint!(process_instruction);
fn process_instruction(_: &Pubkey, accounts: &[AccountInfo], instruction: &[u8]) -> ProgramResult {
    if accounts.len() == 0 {
        Err(MessageError::NoAccountsPassed)?
    }
    if accounts.len() == 1 {
        Err(MessageError::QueueNotPassed)?
    }
    if accounts.len() > 2 {
        Err(MessageError::ExtraAccountsPassed)?
    }
    let signer = &accounts[0];
    let queue = &accounts[1];
    if !signer.is_signer {
        Err(MessageError::SenderDidNotSign)?;
    }
    let message = Message::unpack(instruction)?;
    let mut queue_ref = queue.try_borrow_mut_data()?;
    let message_buffer = MessageBuffer::unpack(&mut *queue_ref)?;

    message_buffer.append(&message);
    Ok(())
}
