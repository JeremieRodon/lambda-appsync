# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.5.2] - 2025-04-23

### Added
- Extended type override capabilities to support operation return types and arguments
- Integration tests to validate macro behavior with different argument sets

### Changed
- Improved error messages for schema file not found scenarios to better explain path resolution

### Fixed
- Bug in `keep_original_function_name` attribute where it would fail when operation handlers had arguments

## [v0.5.1] - 2025-04-21

### Changed
- Improved internal documentation linking structure between crates

### Fixed
- Documentation linking issues between lambda-appsync and lambda-appsync-proc crates

## [v0.5.0] - 2025-04-21

### Added
- New option to get a reference to the AppsyncEvent structure in operation handlers through the `with_appsync_event` attribute parameter
- Comprehensive support for all AWS AppSync auth types (API Key, Cognito User Pools, IAM, OpenID Connect, Lambda)

### Changed
- **Breaking**: Complete rewrite of the `AppsyncIdentity` type to support all AWS AppSync auth modes (previously only supported Cognito)
- Better error handling for auth types serialization/deserialization
- Improved documentation links and clarity about workspace-relative paths

### Fixed
- Missing documentation links in the crate documentation

## [v0.4.2] - 2025-04-19

### Fixed
- Clippy warnings and code style improvements

## [v0.4.1] - 2025-04-19 [YANKED]

### Changed
- Enhanced test coverage for null handling in GraphQL schema type generation

## [v0.4.0] - 2025-04-19 [YANKED]

### Added
- Comprehensive integration tests for macro-generated structure serialization/deserialization

### Changed
- **Breaking**: Changed GraphQL Int scalar from i64 to i32 to comply with GraphQL specification
- Improved handling of Rust keywords in GraphQL field names (proper escaping with 'r#' prefix)

### Fixed
- Proper handling of Rust keywords when used as field names in GraphQL schema
- Documentation examples and subscription filter code blocks
- README referenced crate version

## [v0.3.0] - 2025-04-04

### Added
- Support for AWS AppSync Enhanced Subscription Filters
- New types for subscription filtering: `FilterGroup`, `Filter`, and `FieldPath`
- Convenience conversions between filter types
- Documentation for subscription filters usage and configuration

### Changed
- **Breaking**: `AppsyncIdentity` structure now has optional groups field for Cognito identity
- DefaultOperation generator now returns optional FilterGroup for subscriptions
- Improved documentation with compiled examples

### Fixed
- Case sensitivity issues with GraphQL schema field names
- Double logging of AppSync Operations
- Memory allocation optimization
- Documentation improvements

## [v0.2.0] - 2025-04-01

### Added
- New `unauthorized` constructor for AppsyncResponse struct
- Several convenience implementations for AWSTimestamp:
  - AddAssign<Duration>
  - SubAssign<Duration>
  - Display formatting
  - PartialEq
  - into_u64/from_u64 methods

### Changed
- Improved documentation with compiled examples
- Reorganized workspace dependencies

## [v0.1.0] - 2025-03-30

### Added
- Initial release with core functionality
- Type-safe GraphQL schema conversion to Rust types
- Complete AWS Lambda runtime integration
- Built-in validation of resolver function signatures
- AWS SDK client initialization support
- AWS SDK error built-in conversion
- Batching support for improved performance
- Optional request validation hook
- Support for custom type overrides
- Basic examples and documentation

[v0.5.1]: https://github.com/JeremieRodon/lambda-appsync/compare/v0.5.0...v0.5.1
[v0.5.0]: https://github.com/JeremieRodon/lambda-appsync/compare/v0.4.2...v0.5.0
[v0.4.2]: https://github.com/JeremieRodon/lambda-appsync/compare/v0.4.1...v0.4.2
[v0.4.1]: https://github.com/JeremieRodon/lambda-appsync/compare/v0.4.0...v0.4.1
[v0.4.0]: https://github.com/JeremieRodon/lambda-appsync/compare/v0.3.0...v0.4.0
[v0.3.0]: https://github.com/JeremieRodon/lambda-appsync/compare/v0.2.0...v0.3.0
[v0.2.0]: https://github.com/JeremieRodon/lambda-appsync/compare/v0.1.0...v0.2.0
[v0.1.0]: https://github.com/JeremieRodon/lambda-appsync/releases/tag/v0.1.0
