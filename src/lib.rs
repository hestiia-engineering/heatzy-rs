//! Heatzy API client library
//!
//! This crate provides a Rust client for the Heatzy REST API.
//!
//! # Example
//!
//! ```no_run
//! use heatzy::{Client, DeviceMode};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = Client::new()?;
//!     client.connect("user@example.com", "password").await?;
//!     
//!     let devices = client.list_devices().await?;
//!     for device in devices {
//!         println!("{}: {}", device.dev_alias, device.did);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod models;

pub use client::Client;
pub use error::HeatzyError;
pub use models::{Device, DeviceMode, LoginCredentials, AuthResponse};