//! Super basic messaging buffer with limited history
use crate::error::MessageError;

use solana_sdk::{program_error::ProgramError, pubkey::Pubkey};

/// Number of bytes allowed per message
pub const MESSAGE_LENGTH: usize = 256;
/// Number of messages kept in the history
pub const MESSAGE_HISTORY: usize = 128;

/**
 * Basic on-chain message struct - defines layout of the message
 */
#[repr(packed)]
#[derive(Copy, Clone)]
pub struct Message {
    /// Key of the writer of the transaction
    pub writer: Pubkey,
    /// Number of bytes contained in the message
    pub length: u8,

    /// Actual instruction data
    pub bytes: [u8; MESSAGE_LENGTH],
}

/**
 * Represent the on-chain state of a message buffer
 */
pub struct MessageBuffer {
    /// Index of the first message in the buffer
    pub queue_head: u32,

    /// Array containing messages
    pub messages: [Message; MESSAGE_HISTORY],
}

impl MessageBuffer {
    /// Appends a message to the queue, possibly overwriting an old one
    #[inline]
    pub fn append(&mut self, message: &Message) {
        let next_head = self.queue_head + 1;
        let next_head = next_head % MESSAGE_HISTORY as u32;

        self.messages[next_head as usize] = *message;
        self.queue_head = next_head;
    }
}

impl MessageBuffer {
    /// Unpacks a buffer of
    pub fn unpack(input: &mut [u8]) -> Result<&mut MessageBuffer, ProgramError> {
        if input.len() != std::mem::size_of::<MessageBuffer>() {
            Err(MessageError::MessageQueueAccountWrongSize)?;
        }

        assert_eq!(
            std::mem::align_of::<MessageBuffer>(),
            std::mem::align_of::<u8>()
        );

        let buffer = unsafe { &mut *(input.as_mut_ptr() as *mut MessageBuffer) };

        if buffer.queue_head >= MESSAGE_HISTORY as u32 {
            Err(MessageError::MessageQueueBad)?;
        }

        Ok(buffer)
    }
}

impl Message {
    /// Unpacks a
    pub fn unpack(input: &[u8]) -> Result<Message, ProgramError> {
        if input.len() > std::mem::size_of::<Message>() {
            Err(MessageError::MessageTooLarge)?;
        }
        if input.len() < std::mem::size_of::<Message>() {
            Err(MessageError::MessageTooSmall)?;
        }

        assert_eq!(input.len(), std::mem::size_of::<Message>());

        let (key, rest) = input.split_at(std::mem::size_of::<Pubkey>());
        let (length, rest) = rest.split_first().unwrap();

        let writer = Pubkey::new(key);
        let length = *length;
        let mut bytes = [0; MESSAGE_LENGTH];
        bytes.copy_from_slice(rest);

        if length as usize > MESSAGE_LENGTH {
            Err(MessageError::MessageLengthTooLarge)?;
        }

        if bytes[length as usize..].iter().any(|b| *b != 0) {
            Err(MessageError::MessageLengthTooSmall)?;
        }

        if length == 0 {
            Err(MessageError::MessageEmpty)?;
        }

        Ok(Message {
            writer,
            length,
            bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn construct_basic_message(marker: Option<u8>) -> Message {
        let mut rval = Message {
            writer: Pubkey::new_from_array([0; 32]),
            length: if marker.is_some() { 1 } else { 0 },
            bytes: [0; MESSAGE_LENGTH],
        };
        if let Some(marker) = marker {
            rval.bytes[0] = marker;
        }
        rval
    }

    fn construct_buffer() -> MessageBuffer {
        let base = construct_basic_message(None);
        MessageBuffer {
            queue_head: 0,
            messages: [base; MESSAGE_HISTORY],
        }
    }

    #[test]
    fn test_push() {
        let mut buffer = construct_buffer();
        let initial = 1 + buffer.queue_head as usize;
        for i in 0..MESSAGE_HISTORY {
            assert_eq!(buffer.queue_head, i as u32);
            buffer.append(&construct_basic_message(Some(i as u8)));
            assert_eq!(buffer.queue_head, (i + 1) as u32 % MESSAGE_HISTORY as u32);
            for j in 0..=i {
                let index = initial + j;
                let index = index % MESSAGE_HISTORY;
                assert_eq!(buffer.messages[index].length, 1);
                assert_eq!(buffer.messages[index].bytes[0], j as u8);
            }

            for j in (i + 1)..MESSAGE_HISTORY {
                let index = initial + j;
                let index = index % MESSAGE_HISTORY;
                assert_eq!(buffer.messages[index].length, 0);
                assert_eq!(buffer.messages[index].bytes[0], 0);
            }
        }
        assert_eq!(buffer.queue_head, 0);
        assert_eq!(buffer.messages[1].length, 1);
        assert_eq!(buffer.messages[1].bytes[0], 0);
        buffer.append(&construct_basic_message(Some(MESSAGE_HISTORY as u8)));
        assert_eq!(buffer.queue_head, 1);
        assert_eq!(buffer.messages[1].length, 1);
        assert_eq!(buffer.messages[1].bytes[0], MESSAGE_HISTORY as u8);
    }
}
