#![deny(missing_docs)]

//! A basic message buffer with limited history for a sending bytes around

pub mod buffer;
pub mod entrypoint;
pub mod error;

// Export current solana-sdk types for downstream users who may also be building with a different
// solana-sdk version
pub use solana_sdk;

solana_sdk::declare_id!("FZrii8Bse2T7u6bAXe9UaUSRg5juoFEpJj6XuJmrLQ7J");
