#![deny(missing_docs)]

//! A basic message buffer with limited history for a sending bytes around

pub mod buffer;
pub mod entrypoint;
pub mod error;

// Export current solana-sdk types for downstream users who may also be building with a different
// solana-sdk version
pub use solana_sdk;

solana_sdk::declare_id!("24CJEtjU3sjzNhzS3NarwcQwbZv68Zbgh1Kh86vQFX2f");
