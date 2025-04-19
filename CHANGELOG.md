# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.4.2 - 2025-04-19

- 7a72565 Making clippy happy


## v0.4.1 - 2025-04-19 (unpublished)

- a0b8136 Fix/Improv: Making the test_null_handling integration test actually usefull by introcuding a type with a nullable field in the GraphQL schema

## v0.4.0 - 2025-04-19 (unpublished)

- 4e316a7 Feat: Adding integration tests testing ser/de of the macro-generated strcutures
- c3424e5 Fix: Making sure Rust keywords are properly escaped when used as field names in the GraphQL schema, e.g. a field named 'type' in the GraphQL schema will become 'r#type' in the Rust structure. Note that crate, self and super are special Rust keywords that cannot be escaped using the 'r#' syntax, instead they will be prefixed with 'r_', e.g. a field named 'crate' will become 'r_crate'
- 13fff75 Improving span handling of the Name type struct
- 5616d48 Making it so all the code generated from the GraphQL schema is 'spanned' on the schema file path argument of the macro
- 53d7c25 Introducing Rust keywords as field names in the example schema, so that the test fails
- 61d6dfa Fix: Update README referenced crate version
- 6974e43 Fix (Breaking): Using an i32 for GraphQL scalar  instead of a i64, per GraphQL specification
- 6ea63ca Fix: Marking the Response Mapping template for enhenced subscription filter codeblock as VTL to prevent Rust to trying to compile it
- f52b927 Feat: Making the Response Mapping template for enhenced subscription filters more generic

## v0.3.0 - 2025-04-04

- 47f17cc Doc/Feat: Again improving documentation examples. They are now all compiled as part of testing
- 4fd2e7e Doc: Adding precisions on what Response Mapping Template to use in AWS AppSync in order for enhenced subscription filters to work
- 993e9b1 Feat/BugFix: Adding IFSBValueMarker/IFSValueMarker traits on every lambda_appsync AWS types
- bfc571c Feat: Improving the documentation, notably adding something concerning Subscription Filters
- 598db75 Feat: Changing the DefaultOperation generator so that subscriptions now return an optionnal FilterGroup
- 1a6b1b0 Feat: Adding convenience conversion for FieldFilter -> Filter, Filter -> FilterGroup and FieldFilter -> FilterGroup
- 3dcef68 Feat: Adding support for AppSync subscription filters in a type-safe maner
- 8cfd4ba BugFix: Making clippy happy by removing a useless allocation
- e08daba BugFix: Removed the double log::info! of the Appsync Operation
- 7cd00bf BugFix: Stopped assuming the case of the GraphQl schema type fields. Feat: Improving the Struct fields code generation so that serde option  is only added for GraphQL  not for  (which is not serializable)
- b45b6e1 BugFix: Changing AppsyncIdentity structure so users wheel networkmanager docker tss is now an Option<Vec<String>> instead of a Vec<String> as Appsync can pass  values in this field

## v0.2.0 - 2025-04-01

- b147f57 Feat: Improving appsync_lambda_main documentation so that the code is actualy at least compiled. Also reorganizing the workspace dependencies
- 573717d Feat: Improving appsync_lambda_main documentation
- 86f1e5b Feat: Adding a unauthorized constructor to the AppsyncResponse struct
- 06d75a2 Feat: Added several implementation to AWSTimestanp to make it easier to use
- 1c5a56d Slight docstring example cleanup

## [0.1.0] - 2025-03-30

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

### Changed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A

[0.1.0]: https://github.com/JeremieRodon/lambda-appsync/releases/tag/v0.1.0
