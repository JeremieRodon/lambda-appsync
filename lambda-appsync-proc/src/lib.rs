//! Crate not intended for direct use.

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
/// # Schema Path Argument
///
/// The first argument to this macro must be a string literal containing the path to your GraphQL schema file.
/// The schema path can be:
///
/// - An absolute filesystem path (e.g. "/home/user/project/schema.graphql")
/// - A relative path, that will be relative to your crate's root directory (e.g. "schema.graphql", "graphql/schema.gql")
/// - When in a workspace context, the relative path will be relative to the workspace root directory
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
/// - `type_override` - see section below for details
/// - `name_override` - see section below for details
/// - `field_type_override` (Deprecated): Same as `type_override`
///
/// ## Type Overrides
///
/// The `type_override` option allows overriding Rust types affected to various schema elements:
///
/// - GraphQL `type` and `input` Field types: `type_override = Type.field: CustomType`
/// - Operation return types (Query/Mutation): `type_override = OpType.operation: CustomType`
/// - Operation arguments (Query/Mutation/Subscription): `type_override = OpType.operation.arg: CustomType`
///
/// These overrides are only for the Rust code and must be compatible for serialization/deserialization purposes,
/// i.e. you can use `String` for a GraphQL `ID` but you cannot use a `u32` for a GraphQL `Float`.
///
/// ## Name Overrides
///
/// The `name_override` option supports renaming various schema elements:
///
/// - Type/input/enum names: `name_override = TypeName: NewTypeName`
/// - Field names: `name_override = Type.field: new_field_name`
/// - Enum variants: `name_override = Enum.VARIANT: NewVariant`
///
/// These overrides are only for the Rust code and will not change serialization/deserialization,
/// i.e. `serde` will rename to the original GraphQL schema name.
///
/// # AWS SDK Clients
///
/// AWS SDK clients can be initialized by providing function definitions that return a cached SDK client type.
/// Each client is initialized only once and stored in a static [OnceLock](std::sync::OnceLock), making subsequent function calls
/// essentially free:
///
/// - Function name: Any valid Rust identifier that will be used to access the client
/// - Return type: Must be a valid AWS SDK client like `aws_sdk_dynamodb::Client`
///
/// ```no_run
/// # mod sub {
/// use lambda_appsync::appsync_lambda_main;
///
/// // Single client
/// appsync_lambda_main!(
///     "schema.graphql",
///     dynamodb() -> aws_sdk_dynamodb::Client,
/// );
/// # }
/// # fn main() {}
/// ```
/// ```no_run
/// # mod sub {
/// # use lambda_appsync::appsync_lambda_main;
/// // Multiple clients
/// appsync_lambda_main!(
///     "schema.graphql",
///     dynamodb() -> aws_sdk_dynamodb::Client,
///     s3() -> aws_sdk_s3::Client,
/// );
/// # }
/// # fn main() {}
/// ```
///
/// These client functions can then be called from anywhere in the Lambda crate:
/// ```no_run
/// # fn dynamodb() -> aws_sdk_dynamodb::Client {
/// #   todo!()
/// # }
/// # fn s3() -> aws_sdk_s3::Client {
/// #   todo!()
/// # }
/// # mod sub {
/// use crate::{dynamodb, s3};
/// async fn do_something() {
///     let dynamodb_client = dynamodb();
///     let s3_client = s3();
///     // Use clients...
/// }
/// # }
/// # fn main() {}
/// ```
///
/// # Examples
///
/// ## Basic usage with authentication hook:
/// ```no_run
/// # mod sub {
/// use lambda_appsync::{appsync_lambda_main, AppsyncEvent, AppsyncResponse, AppsyncIdentity};
///
/// fn is_authorized(identity: &AppsyncIdentity) -> bool {
///     todo!()
/// }
///
/// // If the function returns Some(AppsyncResponse), the Lambda function will immediately return it.
/// // Otherwise, the normal flow of the AppSync operation processing will continue.
/// // This is primarily intended for advanced authentication checks that AppSync cannot perform, such as verifying that a user is requesting their own ID.
/// async fn auth_hook(
///     event: &lambda_appsync::AppsyncEvent<Operation>
/// ) -> Option<lambda_appsync::AppsyncResponse> {
///     // Verify JWT token, check permissions etc
///     if !is_authorized(&event.identity) {
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
/// # }
/// # fn main() {}
/// ```
///
/// ## Generate only types for lib code generation:
/// ```no_run
/// # mod sub {
/// use lambda_appsync::appsync_lambda_main;
/// appsync_lambda_main!(
///     "schema.graphql",
///     only_appsync_types = true
/// );
/// # }
/// # fn main() {}
/// ```
///
/// ## Override field types, operation return type or argument types:
/// ```no_run
/// # mod sub {
/// use lambda_appsync::appsync_lambda_main;
/// appsync_lambda_main!(
///     "schema.graphql",
///     // Use String instead of the default lambda_appsync::ID
///     // Override Player.id to use String instead of ID
///     type_override = Player.id: String,
///     // Multiple overrides, here changing another `Player` field type
///     type_override = Player.team: String,
///     // Return value override
///     type_override = Query.gameStatus: String,
///     type_override = Mutation.setGameStatus: String,
///     // Argument override
///     type_override = Query.player.id: String,
///     type_override = Mutation.deletePlayer.id: String,
///     type_override = Subscription.onDeletePlayer.id: String,
/// );
/// # }
/// # fn main() {}
/// ```
///
/// ## Override type, input, enum, fields or variants names:
/// ```no_run
/// # mod sub {
/// use lambda_appsync::appsync_lambda_main;
/// appsync_lambda_main!(
///     "schema.graphql",
///     // Override Player struct name
///     name_override = Player: NewPlayer,
///     // Override Player struct field name
///     name_override = Player.name: email,
///     // Override team `PYTHON` to be `Snake` (instead of `Python`)
///     name_override = Team.PYTHON: Snake,
///     // MUST also override ALL the operations return type !!!
///     type_override = Query.players: NewPlayer,
///     type_override = Query.player: NewPlayer,
///     type_override = Mutation.createPlayer: NewPlayer,
///     type_override = Mutation.deletePlayer: NewPlayer,
/// );
/// # }
/// # fn main() {}
/// ```
/// Note that when using `name_override`, the macro does not automatically change the case:
/// you are responsible to provide the appropriate casing or Clippy will complain.
///
/// ## Disable batch processing:
/// ```no_run
/// # mod sub {
/// lambda_appsync::appsync_lambda_main!(
///     "schema.graphql",
///     batch = false
/// );
/// # }
/// # fn main() {}
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
/// # lambda_appsync::appsync_lambda_main!(
/// #    "schema.graphql",
/// #     exclude_lambda_handler = true,
/// # );
/// # mod sub {
/// # async fn dynamodb_get_players() -> Result<Vec<Player>, AppsyncError> {
/// #    todo!()
/// # }
/// # async fn dynamodb_create_player(name: String) -> Result<Player, AppsyncError> {
/// #    todo!()
/// # }
/// use lambda_appsync::{appsync_operation, AppsyncError};
///
/// // Your types are declared at the crate level by the appsync_lambda_main! macro
/// use crate::Player;
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
/// # }
/// # fn main() {}
/// ```
///
/// ## Using the AppSync event
///
/// You may need to explore the [AppsyncEvent](struct.AppsyncEvent.html) received by the lambda
/// in your operation handler. You can make it available by adding the `with_appsync_event` flag and
/// adding a reference to it in your operation handler signature (must be the last argument), like so:
/// ```no_run
/// # lambda_appsync::appsync_lambda_main!(
/// #    "schema.graphql",
/// #     exclude_lambda_handler = true,
/// # );
/// # mod sub {
/// # async fn dynamodb_create_player(name: String) -> Result<Player, AppsyncError> {
/// #    todo!()
/// # }
/// use lambda_appsync::{appsync_operation, AppsyncError, AppsyncEvent, AppsyncIdentity};
///
/// // Your types are declared at the crate level by the appsync_lambda_main! macro
/// use crate::{Operation, Player};
///
/// // Use the AppsyncEvent
/// #[appsync_operation(mutation(createPlayer), with_appsync_event)]
/// async fn create_player(name: String, event: &AppsyncEvent<Operation>) -> Result<Player, AppsyncError> {
///     // Example: extract the cognito user ID
///     let user_id = if let AppsyncIdentity::Cognito(cognito_id) = &event.identity {
///         cognito_id.sub.clone()
///     } else {
///         return Err(AppsyncError::new("Unauthorized", "Must be Cognito authenticated"))
///     };
///     Ok(dynamodb_create_player(name).await?)
/// }
/// # }
/// # fn main() {}
/// ```
///
/// Note that the `args` field of the [AppsyncEvent](struct.AppsyncEvent.html) will always contain
/// [Null](https://docs.rs/serde_json/latest/serde_json/enum.Value.html#variant.Null) at this stage because its initial content is taken to extract
/// the argument values for the operation.
///
/// ## Preserve original function name
///
/// By default the [macro@appsync_operation] macro will discard your function's name but
/// you can also keep it available by adding the `keep_original_function_name` flag:
/// ```no_run
/// # lambda_appsync::appsync_lambda_main!(
/// #    "schema.graphql",
/// #     exclude_lambda_handler = true,
/// # );
/// # mod sub {
/// use lambda_appsync::{appsync_operation, AppsyncError};
///
/// // Your types are declared at the crate level by the appsync_lambda_main! macro
/// use crate::Player;
///
/// # async fn dynamodb_get_players() -> Result<Vec<Player>, AppsyncError> {
/// #    todo!()
/// # }
/// // Keep the original function name available separately
/// #[appsync_operation(query(players), keep_original_function_name)]
/// async fn fetch_players() -> Result<Vec<Player>, AppsyncError> {
///     Ok(dynamodb_get_players().await?)
/// }
/// async fn other_stuff() {
///     // Can still call fetch_players() directly
///     fetch_players().await;
/// }
/// # }
/// # fn main() {}
/// ```
///
/// ## Using enhanced subscription filters
///
/// ```no_run
/// # lambda_appsync::appsync_lambda_main!(
/// #    "schema.graphql",
/// #     exclude_lambda_handler = true,
/// # );
/// // (Optional) Use an enhanced subscription filter for onCreatePlayer
/// use lambda_appsync::{appsync_operation, AppsyncError};
/// use lambda_appsync::subscription_filters::{FilterGroup, Filter, FieldPath};
///
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
/// When using a single [FieldPath](subscription_filters/struct.FieldPath.html) you can turn it directly into a [FilterGroup](subscription_filters/struct.FilterGroup.html).
/// The following code is equivalent to the one above:
/// ```no_run
/// # lambda_appsync::appsync_lambda_main!(
/// #    "schema.graphql",
/// #     exclude_lambda_handler = true,
/// # );
/// # use lambda_appsync::{appsync_operation, AppsyncError};
/// # use lambda_appsync::subscription_filters::{FilterGroup, Filter, FieldPath};
/// #[appsync_operation(subscription(onCreatePlayer))]
/// async fn on_create_player(name: String) -> Result<Option<FilterGroup>, AppsyncError> {
///     Ok(Some(FieldPath::new("name")?.contains(name).into()))
/// }
/// # fn main() {}
/// ```
///
/// ### Important Note
///
/// When using enhanced subscription filters (i.e., returning a [FilterGroup](subscription_filters/struct.FilterGroup.html)
/// from Subscribe operation handlers), you need to modify your ***Response*** mapping in AWS AppSync.
/// It must contain the following:
///
/// ```vtl
/// #if($context.result.data)
/// $extensions.setSubscriptionFilter($context.result.data)
/// #end
/// null
/// ```
#[proc_macro_attribute]
pub fn appsync_operation(args: TokenStream, input: TokenStream) -> TokenStream {
    appsync_operation::appsync_operation_impl(args, input)
}
