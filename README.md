<!-- PROJECT SHIELDS -->
[![crates.io](https://img.shields.io/crates/v/lambda-appsync.svg)](https://crates.io/crates/lambda-appsync)
[![docs.rs](https://docs.rs/lambda-appsync/badge.svg)](https://docs.rs/lambda-appsync/0.1.0/lambda_appsync)
[![CI](https://github.com/JeremieRodon/lambda-appsync/workflows/CI/badge.svg)](https://github.com/JeremieRodon/lambda-appsync/actions)
[![License](https://img.shields.io/github/license/JeremieRodon/lambda-appsync.svg)](https://github.com/JeremieRodon/lambda-appsync/blob/master/LICENSE)

# lambda-appsync

A Rust framework for implementing AWS AppSync Direct Lambda resolvers with complete type safety and validation.

The lambda-appsync crate provides procedural macros that convert GraphQL schemas into type-safe Rust code and types for AWS AppSync event Lambda integration. This allows you to focus on implementing resolver logic while the framework handles all AWS AppSync integration details.

## Features

- ✨ Type-safe GraphQL schema conversion to Rust types
- 🔔 AWS AppSync enhanced subscription filters
- 🚀 Full AWS Lambda runtime integration
- 🔒 Built-in validation of resolver function signatures
- 🔌 Easy AWS SDK client initialization
- 📦 Performance-optimized batching support
- 🛡️ Flexible request validation hooks (e.g. for advanced authentication flows)
- 🔐 Comprehensive support for all AWS AppSync auth types

## Known limitations

The framework currently has limited support for certain AWS AppSync and GraphQL schema features:

- GraphQL unions are not supported and will be ignored by the macro
- GraphQL interfaces are not directly supported, though concrete types that implement interfaces will work correctly

We actively track user needs around these features. If your project requires union or interface support, please open a GitHub issue detailing your use case and expected implementation. Your feedback helps us prioritize future development work and determine the best way to implement these features in a type-safe manner.

## Installation

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
lambda-appsync = "0.5.2"
```

## Quick Start

1. Create your GraphQL schema file (e.g. `graphql/schema.gql`).

Note: When in a workspace context, all relative paths are assumed to be relative to the workspace root directory:
```graphql
type Query {
  players: [Player!]!
  gameStatus: GameStatus!
}

type Player {
  id: ID!
  name: String!
  team: Team!
}

enum Team {
  RUST
  PYTHON
  JS
}

enum GameStatus {
  STARTED
  STOPPED
}
```

2. Configure the Lambda runtime with AWS SDK clients in `main.rs`:

```rust
use lambda_appsync::appsync_lambda_main;

// Generate types and runtime setup from schema
appsync_lambda_main!(
    "graphql/schema.gql",
    // Initialize DynamoDB client if needed
    dynamodb() -> aws_sdk_dynamodb::Client,
);
```

3. Implement resolver functions for GraphQL operations in your crate:

```rust
use lambda_appsync::{appsync_operation, AppsyncError};
use lambda_appsync::subscription_filters::{FieldPath, FilterGroup};
// The appsync_lambda_main! macro will have created the
// types declared in schema.gql at the crate root
use crate::{Player, GameStatus};

#[appsync_operation(query(players))]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    let client = dynamodb();
    // Implement resolver logic
    todo!()
}

#[appsync_operation(query(gameStatus))]
async fn get_game_status() -> Result<GameStatus, AppsyncError> {
    let client = dynamodb();
    // Implement resolver logic
    todo!()
}

#[appsync_operation(subscription(onCreatePlayer))]
async fn on_create_player(name: String) -> Result<Option<FilterGroup>, AppsyncError> {
    // Return a subscription filter to subscribe only
    // to events where the player name contains the string `name`
    Ok(Some(
        FieldPath::new("name")?.contains(name).into()
    ))
}
```

The framework's macros verify function signatures match the GraphQL schema and automatically wire everything up to handle AWS AppSync requests.

### Important Note

When using enhanced subscription filters (i.e., returning a [FilterGroup](lambda_appsync::subscription_filters::FilterGroup) from Subscribe operation handlers), you need to modify your ***Response*** mapping in AWS AppSync.

It must contain exactly the following:

```vtl
#if($context.result.data)
$extensions.setSubscriptionFilter($context.result.data)
#end
null
```

## Example project

Check out our [complete sample project](https://github.com/JeremieRodon/demo-rust-lambda-appsync) that demonstrates lambda-appsync in action! This full-featured demo implements a GraphQL API for a mini-game web application using AWS AppSync and Lambda, showcasing:

- 🎮 Real-world GraphQL schema
- 📊 DynamoDB integration
- 🏗️ Infrastructure as code with AWS CloudFormation
- 🚀 CI/CD pipeline configuration

Clone the repo to get started with a production-ready template that you can use as reference for your own projects. The demo includes detailed documentation and best practices for building serverless GraphQL APIs with Rust.

## Additional Examples

### Custom Type Overrides

Override generated Rust types for specific GraphQL fields:

```rust
appsync_lambda_main!(
    "graphql/schema.gql",
    // Override Player.id type to be String instead of ID on the Rust struct
    field_type_override = Player.id: String
);
```
### Subscription Filters

The framework provides subscription filtering capabilities:

```rust
use lambda_appsync::{appsync_operation, AppsyncError};
use lambda_appsync::subscription_filters::{FieldPath, FilterGroup};

#[appsync_operation(subscription(onCreatePlayer))]
async fn on_create_player(name: String) -> Result<Option<FilterGroup>, AppsyncError> {
    // Subscribe only to events where player name contains the given string
    Ok(Some(FieldPath::new("name")?.contains(name).into()))
}
```

Important: When using enhanced subscription filters, update your AppSync Response Mapping Template:

```vtl
#if($context.result.data)
$extensions.setSubscriptionFilter($context.result.data)
#end
null
```

### Accessing the AppSync Event

Access the full AppSync event context in operation handlers:

```rust
#[appsync_operation(mutation(createPlayer), with_appsync_event)]
async fn create_player(
    name: String,
    event: &AppsyncEvent<Operation>
) -> Result<Player, AppsyncError> {
    // Extract Cognito user ID from event
    let user_id = if let AppsyncIdentity::Cognito(cognito) = &event.identity {
        cognito.sub.clone()
    } else {
        return Err(AppsyncError::new("Unauthorized", "Must be Cognito authenticated"));
    };
    // Other use of the event...
    todo!()
}
```

### Preserving Original Function Names

Keep the original function name available while using it as an operation handler:

```rust
// Can still call fetch_user() directly
#[appsync_operation(query(getUser), keep_original_function_name)]
async fn fetch_user(id: ID) -> Result<User, AppsyncError> {
    todo!()
}
```

### Modular Type and Implementation Structure

For larger projects, share GraphQL types across multiple Lambda functions while keeping resolvers separate:

```rust
// In a shared library crate:
appsync_lambda_main!(
    "graphql/schema.gql",
    only_appsync_types = true,
);

// Then in each Lambda using this lib:
use shared_lib::*;

appsync_lambda_main!(
    "graphql/schema.gql",
    exclude_appsync_types = true,
    dynamodb() -> aws_sdk_dynamodb::Client
);
```

This enables defining custom traits and methods on GraphQL types in one place while reusing them across multiple Lambda functions. The shared library contains type definitions, while each Lambda maintains its operation handlers and AWS SDK client initialization.

### AWS SDK Error Support

Seamlessly handle AWS SDK errors with automatic conversion:

```rust
async fn store_item(item: Item, client: &aws_sdk_dynamodb::Client) -> Result<(), AppsyncError> {
    // AWS SDK errors are automatically converted to AppsyncError
    client.put_item()
        .table_name("my-table")
        .item("id", AttributeValue::S(item.id.to_string()))
        .item("data", AttributeValue::S(item.data))
        .send()
        .await?;
    Ok(())
}
```

Error types and messages are extracted from AWS SDK error metadata, allowing use of the `?` operator with AWS SDK calls for properly formatted AppSync response errors.

### Error Merging

Combine multiple errors using the pipe operator:

```rust
let err = AppsyncError::new("ValidationError", "Invalid email")
    | AppsyncError::new("DatabaseError", "User not found");
```

## Minimum Supported Rust Version (MSRV)

This crate requires Rust version 1.81.0 or later.

## Contributing

We welcome contributions! Here's how you can help:

1. Report bugs by opening an issue
2. Suggest new features or improvements
3. Submit pull requests for bug fixes or features
4. Improve documentation
5. Share example code and use cases

Please review our contributing guidelines before submitting pull requests.

### Git Hooks

This project uses git hooks to ensure code quality. The hooks are automatically installed when you enter a development shell using `nix flakes` and `direnv`.

The following checks are run before each commit:
- Code formatting (cargo fmt)
- Linting (cargo clippy)
- Doc generation (cargo doc)
- Tests (cargo test)

If any of these checks fail, the commit will be aborted. Fix the issues and try committing again.

To manually install the hooks:
```bash
./scripts/install-hooks.sh
```

Note: Any changes that have not passed local tests will result in CI failures, as GitHub Actions performs identical verification checks.

## Issues

Before reporting issues, please check:

1. Existing issues to avoid duplicates
2. The documentation to ensure it's not a usage error
3. The FAQ for common problems

When opening a new issue, include:

- A clear title and description
- Steps to reproduce bugs
- Expected vs actual behavior
- Code samples if relevant

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Authors

- Jérémie RODON ([@JeremieRodon](https://github.com/JeremieRodon))

If you find this crate useful, please star the repository and share your feedback!
