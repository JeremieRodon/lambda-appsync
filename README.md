<!-- PROJECT SHIELDS -->
[![crates.io](https://img.shields.io/crates/v/lambda-appsync.svg)](https://crates.io/crates/lambda-appsync)
[![docs.rs](https://docs.rs/lambda-appsync/badge.svg)](https://docs.rs/lambda-appsync/0.1.0/lambda_appsync)
[![CI](https://github.com/JeremieRodon/lambda-appsync/workflows/CI/badge.svg)](https://github.com/JeremieRodon/lambda-appsync/actions)
[![License](https://img.shields.io/github/license/JeremieRodon/lambda-appsync.svg)](https://github.com/JeremieRodon/lambda-appsync/blob/master/LICENSE)

# lambda-appsync

A Rust framework that makes it easy to implement AWS AppSync Direct Lambda resolvers with complete type safety and validation.

lambda-appsync provides procedural macros and types to help convert GraphQL schemas into type-safe Rust code with full AWS Lambda runtime support. It allows you to focus on implementing the resolver logic while handling all the AWS AppSync integration details.

## Features

- ✨ Type-safe GraphQL schema conversion to Rust types
- 🚀 Complete AWS Lambda runtime integration
- 🔒 Built-in validation of resolver function signatures
- 🔌 Easy AWS SDK client initialization
- 📦 Batching support for improved performance
- 🛡️ Optional request validation hooks (e.g. for advanced authentication logic)

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
lambda-appsync = "0.1.0"
```

## Quick Start

1. Define your GraphQL schema in a separate file (e.g. `schema.graphql`):

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

2. Set up the Lambda runtime with AWS SDK clients in `main.rs`:

```rust
use lambda_appsync::appsync_lambda_main;

// Generate types and runtime setup from schema
appsync_lambda_main!(
    "schema.graphql",
    // Initialize DynamoDB client if needed
    dynamodb() -> aws_sdk_dynamodb::Client,
);
```

3. Implement resolver functions for GraphQL operations anywhere in the same crate:

```rust
use lambda_appsync::{appsync_operation, AppsyncError};
// The appsync_lambda_main! macro will have created the
// types declared in schema.graphql at the crate root
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
```

The macros ensure function signatures match the GraphQL schema and wire everything up to handle AWS AppSync requests automatically.

## Additional Examples

### Custom Type Overrides

You can override the generated Rust type for specific GraphQL fields:

```rust
appsync_lambda_main!(
    "schema.graphql",
    // Override Player.id type to be String instead of ID on the Rust struct
    field_type_override = Player.id: String
);
```

### Keeping Original Function Names

By default, the `appsync_operation` macro only use the body and signature of the function you provide to create an operation handler. You can ask to keep the original function name available separately:

```rust
#[appsync_operation(query(getUser), keep_original_function_name)]
async fn fetch_user(id: ID) -> Result<User, AppsyncError> {
    // Can still call fetch_user() directly
    todo!()
}
```

### Separate types and main implementation

For larger projects, you may want to share the GraphQL types across multiple Lambda functions while keeping the resolvers separate. The `appsync_lambda_main!` macro supports this pattern through flags:

```rust
// In a shared library crate:
appsync_lambda_main!(
    "schema.graphql",
    only_appsync_types = true,
);

// Then in each Lambda using this lib:
use shared_lib::*;

appsync_lambda_main!(
    "schema.graphql",
    exclude_appsync_types = true,
    dynamodb() -> aws_sdk_dynamodb::Client
);
```

This allows you to define custom traits and methods on the GraphQL types in one place and reuse them across multiple Lambda functions. The shared library contains just the type definitions, while each Lambda gets its own operation handlers and AWS SDK client initialization.

### Error Handling

Multiple errors can be combined using the pipe operator:

```rust
let err = AppsyncError::new("ValidationError", "Invalid email")
    | AppsyncError::new("DatabaseError", "User not found");
```
### AWS SDK Error Support

The `AppsyncError` type automatically handles AWS SDK errors for seamless integration:

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

The error type and message are extracted from the AWS SDK error metadata. This means you can use the `?` operator with AWS SDK calls and get properly formatted errors in your AppSync responses.

## Minimum Supported Rust Version (MSRV)

The minimum supported Rust version is 1.81.0.

## Contributing

We welcome contributions! Here are some ways you can help:

1. Report bugs by opening an issue
2. Suggest new features or improvements
3. Submit pull requests for bug fixes or features
4. Improve documentation
5. Share example code and use cases

Please review our contributing guidelines before submitting pull requests.

## Issues

If you find a bug or have a feature request, please check:

1. Existing issues to avoid duplicates
2. The documentation to ensure it's not a usage error
3. The FAQ for common problems

Then open a new issue with:

- A clear title and description
- Steps to reproduce bugs
- Expected vs actual behavior
- Code samples if relevant

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Authors

- Jérémie RODON ([@JeremieRodon](https://github.com/JeremieRodon))

If you find this crate useful, please star the repo and share your feedback!
