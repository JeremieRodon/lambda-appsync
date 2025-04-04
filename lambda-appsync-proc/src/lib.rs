#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
//! This crate provides procedural macros for implementing AWS AppSync Direct Lambda resolvers.
//!
//! It helps convert GraphQL schemas into type-safe Rust code with full AWS Lambda runtime support.
//! The main functionality is provided through the `appsync_lambda_main` and `appsync_operation` macros.
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
//!
//! // Generate types and runtime setup from schema
//! appsync_lambda_main!(
//!     "schema.graphql",
//!     // Initialize DynamoDB client if needed
//!     dynamodb() -> aws_sdk_dynamodb::Client,
//!     // Enable validation hook
//!     hook = verify_request,
//!     // Enable batch processing
//!     batch = true,
//! #   exclude_lambda_handler = true,
//! );
//! # fn dynamodb() -> aws_sdk_dynamodb::Client {todo!()}
//! # fn main() {}
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
//!
//! // The macro ensures the function signature matches the GraphQL schema
//! // and wires everything up to handle AWS AppSync requests automatically
//! ```

mod appsync_lambda_main;
mod appsync_operation;
mod common;

use proc_macro::TokenStream;

/// Generates the code required to handle AWS AppSync Direct Lambda resolver events based on a GraphQL schema.
///
/// This macro takes a path to a GraphQL schema file and generates the complete foundation
/// for implementing an AWS AppSync Direct Lambda resolver:
///
/// - Rust types for all GraphQL types (enums, inputs, objects)
/// - Query/Mutation/Subscription operation enums
/// - AWS Lambda runtime setup with logging to handle the AWS AppSync event
/// - Optional AWS SDK client initialization
///
/// # Options
///
/// - `batch = bool`: Enable/disable batch request handling (default: true)
/// - `hook = fn_name`: Add a custom hook function for request validation/auth
/// - `exclude_lambda_handler = bool`: Skip generation of Lambda handler code
/// - `only_lambda_handler = bool`: Only generate Lambda handler code
/// - `exclude_appsync_types = bool`: Skip generation of GraphQL type definitions
/// - `only_appsync_types = bool`: Only generate GraphQL type definitions
/// - `exclude_appsync_operations = bool`: Skip generation of operation enums
/// - `only_appsync_operations = bool`: Only generate operation enums
/// - `field_type_override = Type.field: CustomType`: Override type of a specific field
///
/// # Examples
///
/// Basic usage with authentication hook:
/// ```no_run
/// use lambda_appsync::{appsync_lambda_main, AppsyncEvent, AppsyncResponse, AppsyncIdentity};
///
///  fn is_authorized(identity: Option<&AppsyncIdentity>) -> bool {
///     todo!()
/// }
///
/// // If the function returns Some(AppsyncResponse), the Lambda function will immediatly return it
/// // Else, the normal flow of the AppSync operation processing will continue
/// // This is primarily intended for advanced authentication checks that AppSync cannot perform,
/// // such as verifying that a user is requesting their own ID for example.
/// async fn auth_hook(
///     event: &lambda_appsync::AppsyncEvent<Operation>
/// ) -> Option<lambda_appsync::AppsyncResponse> {
///     // Verify JWT token, check permissions etc
///     if !is_authorized(event.identity.as_ref()) {
///         return Some(AppsyncResponse::unauthorized());
///     }
///     None
/// }
///
/// appsync_lambda_main!(
///     "schema.graphql",
///     hook = auth_hook,
///     dynamodb() -> aws_sdk_dynamodb::Client
/// );
/// ```
///
/// Generate only types for lib code generation:
/// ```no_run
/// use lambda_appsync::appsync_lambda_main;
/// appsync_lambda_main!(
///     "schema.graphql",
///     only_appsync_types = true
/// );
/// ```
///
/// Override field types and use multiple AWS clients:
/// ```no_run
/// use lambda_appsync::appsync_lambda_main;
/// appsync_lambda_main!(
///     "schema.graphql",
///     dynamodb() -> aws_sdk_dynamodb::Client,
///     s3() -> aws_sdk_s3::Client,
///     // Use String instead of the default lambda_appsync::ID
///     field_type_override = Player.id: String,
/// );
/// ```
///
/// Disable batch processing:
/// ```no_run
/// lambda_appsync::appsync_lambda_main!(
///     "schema.graphql",
///     batch = false
/// );
/// ```
#[proc_macro]
pub fn appsync_lambda_main(input: TokenStream) -> TokenStream {
    appsync_lambda_main::appsync_lambda_main_impl(input)
}

/// Marks an async function as an AWS AppSync resolver operation, binding it to a specific Query,
/// Mutation or Subscription operation defined in the GraphQL schema.
///
/// The marked function must match the signature of the GraphQL operation, with parameters and return
/// type matching what is defined in the schema. The function will be wired up to handle requests
/// for that operation through the AWS AppSync Direct Lambda resolver.
///
/// # Important
/// This macro can only be used in a crate where the [appsync_lambda_main!] macro has been used at the
/// root level (typically in `main.rs`). The code generated by this macro depends on types and
/// implementations that are created by [appsync_lambda_main!].
///
/// # Example Usage
///
/// ```no_run
/// use lambda_appsync::{appsync_operation, AppsyncError};
/// # lambda_appsync::appsync_lambda_main!(
/// #    "schema.graphql",
/// #     exclude_lambda_handler = true,
/// # );
/// # async fn dynamodb_get_players() -> Result<Vec<Player>, AppsyncError> {
/// #    todo!()
/// # }
/// # async fn dynamodb_create_player(name: String) -> Result<Player, AppsyncError> {
/// #    todo!()
/// # }
///
/// // Execute when a 'players' query is received
/// #[appsync_operation(query(players))]
/// async fn get_players() -> Result<Vec<Player>, AppsyncError> {
///     // Implement resolver logic
///     Ok(dynamodb_get_players().await?)
/// }
///
/// // Handle a 'createPlayer' mutation
/// #[appsync_operation(mutation(createPlayer))]
/// async fn create_player(name: String) -> Result<Player, AppsyncError> {
///     Ok(dynamodb_create_player(name).await?)
/// }
///
/// // (Optional) Use an enhanced subscription filter for onCreatePlayer
/// use lambda_appsync::subscription_filters::{FilterGroup, Filter, FieldPath};
/// #[appsync_operation(subscription(onCreatePlayer))]
/// async fn on_create_player(name: String) -> Result<Option<FilterGroup>, AppsyncError> {
///     Ok(Some(FilterGroup::from([
///         Filter::from([
///             FieldPath::new("name")?.contains(name)
///         ])
///     ])))
/// }
/// # fn main() {}
/// ```
///
/// When using a single [FieldPath](lambda_appsync::subscription_filters::FieldPath) you can turn it directly into a [FilterGroup](lambda_appsync::subscription_filters::FilterGroup)
/// ```no_run
/// # use lambda_appsync::{appsync_operation, AppsyncError};
/// # lambda_appsync::appsync_lambda_main!(
/// #    "schema.graphql",
/// #     exclude_lambda_handler = true,
/// # );
/// use lambda_appsync::subscription_filters::{FilterGroup, Filter, FieldPath};
/// #[appsync_operation(subscription(onCreatePlayer))]
/// async fn on_create_player(name: String) -> Result<Option<FilterGroup>, AppsyncError> {
///     Ok(Some(FieldPath::new("name")?.contains(name).into()))
/// }
/// # fn main() {}
/// ```
///
/// By default the #[appsync_operation(...)] macro will discard your function's name but
/// you can also keep it available:
/// ```no_run
/// use lambda_appsync::{appsync_operation, AppsyncError};
/// # lambda_appsync::appsync_lambda_main!(
/// #    "schema.graphql",
/// #     exclude_lambda_handler = true,
/// # );
/// # async fn dynamodb_get_players() -> Result<Vec<Player>, AppsyncError> {
/// #    todo!()
/// # }
/// # async fn dynamodb_create_player(name: String) -> Result<Player, AppsyncError> {
/// #    todo!()
/// # }
/// // Keep the original function name available separately
/// #[appsync_operation(query(players), keep_original_function_name)]
/// async fn fetch_players() -> Result<Vec<Player>, AppsyncError> {
///     // Can still call fetch_players() directly
///     Ok(dynamodb_get_players().await?)
/// }
/// # fn main() {}
/// ```
///
/// The macro will ensure the function signature matches what is defined in the GraphQL schema,
/// and wire it up to be called when AWS AppSync invokes the Lambda resolver for that operation.
///
///
/// # Important Note
///
/// When using enhenced subscription filters (i.e. returning a [FilterGroup](lambda_appsync::subscription_filters::FilterGroup)
/// from Subscribe operation handlers), you need to modify your ***Response*** mapping in AWS AppSync.
/// It must contain exactly the following:
///
/// `$extensions.setSubscriptionFilter($context.result.data)null`
///
#[proc_macro_attribute]
pub fn appsync_operation(args: TokenStream, input: TokenStream) -> TokenStream {
    appsync_operation::appsync_operation_impl(args, input)
}
