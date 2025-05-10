#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![cfg_attr(docsrs, deny(rustdoc::broken_intra_doc_links))]
#![no_std]
//! This crate provides procedural macros and types for implementing
//! AWS AppSync Direct Lambda resolvers.
//!
//! It helps convert GraphQL schemas into type-safe Rust code with full AWS Lambda runtime support.
//! The main functionality is provided through the [appsync_lambda_main] and [appsync_operation] macros.
//!
//! # Complete Example
//!
//! ```no_run
//! use lambda_appsync::{appsync_lambda_main, appsync_operation, AppsyncError};
//!
//! // 1. First define your GraphQL schema (e.g. `schema.graphql`):
//! //
//! // type Query {
//! //   players: [Player!]!
//! //   gameStatus: GameStatus!
//! // }
//! //
//! // type Player {
//! //   id: ID!
//! //   name: String!
//! //   team: Team!
//! // }
//! //
//! // enum Team {
//! //   RUST
//! //   PYTHON
//! //   JS
//! // }
//! //
//! // enum GameStatus {
//! //   STARTED
//! //   STOPPED
//! // }
//!
//! // 2. Initialize the Lambda runtime with AWS SDK clients in main.rs:
//!
//! // Optional hook for custom request validation/auth
//! async fn verify_request(
//!     event: &lambda_appsync::AppsyncEvent<Operation>
//! ) -> Option<lambda_appsync::AppsyncResponse> {
//!     // Return Some(response) to short-circuit normal execution
//!     None
//! }
//! // Generate types and runtime setup from schema
//! appsync_lambda_main!(
//!     "schema.graphql",
//!     // Initialize DynamoDB client if needed
//!     dynamodb() -> aws_sdk_dynamodb::Client,
//!     // Enable validation hook
//!     hook = verify_request,
//!     // Enable batch processing
//!     batch = true
//! );
//!
//! // 3. Implement resolver functions for GraphQL operations:
//!
//! #[appsync_operation(query(players))]
//! async fn get_players() -> Result<Vec<Player>, AppsyncError> {
//!     let client = dynamodb();
//!     todo!()
//! }
//!
//! #[appsync_operation(query(gameStatus))]
//! async fn get_game_status() -> Result<GameStatus, AppsyncError> {
//!     let client = dynamodb();
//!     todo!()
//! }
//! // The macro ensures the function signature matches the GraphQL schema
//! // and wires everything up to handle AWS AppSync requests automatically
//! # mod child {fn main() {}}
//! ```

#[cfg(all(feature = "std", feature = "wasm"))]
compile_error!("feature \"wasm\" and feature \"std\" cannot be enabled at the same time");

#[cfg(feature = "std")]
include!("./with_std.rs");

#[cfg(not(feature = "std"))]
include!("./without_std.rs");

mod appsync;
mod aws_scalars;
mod id;

pub use aws_scalars::{
    datetime::{AWSDate, AWSDateTime, AWSTime},
    email::AWSEmail,
    phone::AWSPhone,
    timestamp::AWSTimestamp,
    url::AWSUrl,
};

pub use id::ID;

pub use appsync::*;

#[doc(inline)]
pub use lambda_appsync_proc::appsync_lambda_main;

#[doc(inline)]
pub use lambda_appsync_proc::appsync_operation;

// Re-export crates that are mandatory for the proc_macro to succeed
pub use log;
pub use serde;
pub use serde_json;

#[cfg(feature = "std")]
pub use aws_config;
#[cfg(feature = "std")]
pub use env_logger;
#[cfg(feature = "std")]
pub use lambda_runtime;
#[cfg(feature = "std")]
pub use tokio;
