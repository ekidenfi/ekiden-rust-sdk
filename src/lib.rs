pub mod auth;
pub mod client;
pub mod config;
pub mod error;
pub mod types;
pub mod utils;
pub mod ws;

// Re-export main types for convenience
pub use auth::Auth;
pub use client::{EkidenClient, EkidenClientBuilder};
pub use config::EkidenConfig;
pub use error::{EkidenError, Result};
pub use types::*;
pub use utils::{Crypto, KeyPair};

pub mod aptos;

pub use aptos::*;
