use crate::stdlib::alloc::{borrow::ToOwned, collections::HashMap, string::String, vec::Vec};

use core::ops::BitOr;

use aws_smithy_types::error::metadata::ProvideErrorMetadata;
use serde_json::Value;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

/// Authorization strategy for AppSync operations.
///
/// It determines whether operations are allowed or denied based on the
/// authentication context provided by AWS AppSync. It is typically used by AppSync
/// itself in conjunction with AWS Cognito user pools and usually do not concern
/// the application code.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AppsyncAuthStrategy {
    /// Allows the operation by default if no explicit authorizer is associated to the field
    Allow,
    /// Denies the operation by default if no explicit authorizer is associated to the field
    Deny,
}

/// Identity information for Cognito User Pools authenticated requests.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppsyncIdentityCognito {
    /// Unique identifier of the authenticated user/client
    pub sub: String,
    /// Username of the authenticated user (from Cognito user pools)
    pub username: String,
    /// Identity provider that authenticated the request (e.g. Cognito user pool URL)
    pub issuer: String,
    /// Default authorization strategy for the authenticated identity
    pub default_auth_strategy: AppsyncAuthStrategy,
    /// Source IP addresses associated with the request
    pub source_ip: Vec<String>,
    /// Groups the authenticated user belongs to
    pub groups: Option<Vec<String>>,
    /// Additional claims/attributes associated with the identity
    pub claims: Value,
}

/// Authentication type in a Cognito Identity Pool
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CognitoIdentityAuthType {
    /// User is authenticated with an identity provider
    Authenticated,
    /// User is an unauthenticated guest
    Unauthenticated,
}

/// Cognito Identity Pool information for federated IAM authentication
#[derive(Debug, Deserialize)]
pub struct CognitoFederatedIdentity {
    /// Unique identifier assigned to the authenticated/unauthenticated identity
    /// within the Cognito Identity Pool
    #[serde(rename = "cognitoIdentityId")]
    pub identity_id: String,
    /// Identifier of the Cognito Identity Pool that is being used for federation.
    /// In the format of region:pool-id
    #[serde(rename = "cognitoIdentityPoolId")]
    pub identity_pool_id: String,
    /// Indicates whether the identity is authenticated with an identity provider
    /// or is an unauthenticated guest access
    #[serde(rename = "cognitoIdentityAuthType")]
    pub auth_type: CognitoIdentityAuthType,
    /// For authenticated identities, contains information about the identity provider
    /// used for authentication. Format varies by provider type
    #[serde(rename = "cognitoIdentityAuthProvider")]
    pub auth_provider: String,
}

/// Identity information for IAM-authenticated requests.
///
/// Contains AWS IAM-specific authentication details, including optional Cognito
/// identity pool information when using federated identities.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppsyncIdentityIam {
    /// AWS account ID of the caller
    pub account_id: String,
    /// Source IP address(es) of the caller
    pub source_ip: Vec<String>,
    /// IAM username of the caller
    pub username: String,
    /// Full IAM ARN of the caller
    pub user_arn: String,
    /// Federated identity information when using Cognito Identity Pools
    #[serde(flatten)]
    pub federated_identity: Option<CognitoFederatedIdentity>,
}

/// Identity information for OIDC-authenticated requests.
#[derive(Debug, Deserialize)]
pub struct AppsyncIdentityOidc {
    /// The issuer of the token
    pub iss: String,
    /// The subject (usually the user identifier)
    pub sub: String,
    /// Token audience
    pub aud: String,
    /// Expiration time
    pub exp: i64,
    /// Issued at time
    pub iat: i64,
    /// Additional custom claims from the OIDC provider
    #[serde(flatten)]
    pub additional_claims: HashMap<String, serde_json::Value>,
}

/// Identity information for Lambda-authorized requests.
#[derive(Debug, Deserialize)]
pub struct AppsyncIdentityLambda {
    /// Custom resolver context returned by the Lambda authorizer
    #[serde(rename = "resolverContext")]
    pub resolver_context: serde_json::Value,
}

/// Identity information for an AppSync request.
///
/// Represents the identity context of the authenticated user/client making the request to
/// AWS AppSync. This enum corresponds directly to AppSync's authorization types as defined
/// in the AWS documentation.
///
/// Each variant maps to one of the five supported AWS AppSync authorization modes:
///
/// - [Cognito](AppsyncIdentity::Cognito): Uses Amazon Cognito User Pools, providing group-based
///   access control with JWT tokens containing encoded user information like groups and custom claims.
///
/// - [Iam](AppsyncIdentity::Iam): Uses AWS IAM roles and policies through AWS Signature Version 4
///   signing. Can be used either directly with IAM users/roles or through Cognito Identity Pools
///   for federated access. Enables fine-grained access control through IAM policies.
///
/// - [Oidc](AppsyncIdentity::Oidc): OpenID Connect authentication integrating with any
///   OIDC-compliant provider.
///
/// - [Lambda](AppsyncIdentity::Lambda): Custom authorization through an AWS Lambda function
///   that evaluates each request.
///
/// - [ApiKey](AppsyncIdentity::ApiKey): Simple API key-based authentication using keys
///   generated and managed by AppSync.
///
/// The variant is determined by the authorization configuration of your AppSync API and
/// the authentication credentials provided in the request. Each variant contains structured
/// information specific to that authentication mode, which can be used in resolvers for
/// custom authorization logic.
///
/// More information can be found in the [AWS documentation](https://docs.aws.amazon.com/appsync/latest/devguide/security-authz.html).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AppsyncIdentity {
    /// Amazon Cognito User Pools authentication
    Cognito(AppsyncIdentityCognito),
    /// AWS IAM authentication
    Iam(AppsyncIdentityIam),
    /// OpenID Connect authentication
    Oidc(AppsyncIdentityOidc),
    /// Lambda authorizer authentication
    Lambda(AppsyncIdentityLambda),
    /// API Key authentication (represents null identity in JSON)
    ApiKey,
}

/// Metadata about an AppSync GraphQL operation execution.
///
/// Contains detailed information about the GraphQL operation being executed,
/// including the operation type, selected fields, and variables. The type parameter
/// `O` represents the enum generated by [appsync_lambda_main](crate::appsync_lambda_main) that defines all valid
/// operations for this Lambda resolver.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AppsyncEventInfo<O> {
    /// The specific GraphQL operation being executed (Query/Mutation)
    #[serde(flatten)]
    pub operation: O,
    /// Raw GraphQL selection set as a string
    #[serde(rename = "selectionSetGraphQL")]
    pub selection_set_graphql: String,
    /// List of selected field paths in the GraphQL query
    #[serde(rename = "selectionSetList")]
    pub selection_set_list: Vec<String>,
    /// Variables passed to the GraphQL operation
    pub variables: HashMap<String, Value>,
}

/// Represents a complete AWS AppSync event sent to a Lambda resolver.
///
/// Contains all context and data needed to resolve a GraphQL operation, including
/// authentication details, operation info, and arguments. The generics `O`
/// must be the Operation enum generated by the [appsync_lambda_main](crate::appsync_lambda_main) macro.
///
/// # Limitations
/// - Omits the `stash` field used for pipeline resolvers
/// - Omits the `prev` field as it's not relevant for direct Lambda resolvers
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AppsyncEvent<O> {
    /// Authentication context
    pub identity: AppsyncIdentity,
    /// Raw request context from AppSync
    pub request: Value,
    /// Parent field's resolved value in nested resolvers
    pub source: Value,
    /// Metadata about the GraphQL operation
    pub info: AppsyncEventInfo<O>,
    /// Arguments passed to the GraphQL field
    #[serde(rename = "arguments")]
    pub args: Value,
    // Should never be usefull in a Direct Lambda Invocation context
    // pub stash: Value,
    // pub prev: Value,
}

/// Response structure returned to AWS AppSync from a Lambda resolver.
///
/// Can contain either successful data or error information, but not both.
/// Should be constructed using From implementations for either [Value] (success)
/// or [AppsyncError] (failure).
///
/// # Examples
/// ```
/// # use serde_json::json;
/// # use lambda_appsync::{AppsyncError, AppsyncResponse};
/// // Success response
/// let response: AppsyncResponse = json!({ "id": 123 }).into();
///
/// // Error response
/// let error = AppsyncError::new("NotFound", "Resource not found");
/// let response: AppsyncResponse = error.into();
/// ```
#[derive(Debug, Serialize)]
pub struct AppsyncResponse {
    data: Option<Value>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    error: Option<AppsyncError>,
}

impl AppsyncResponse {
    /// Returns an unauthorized error response
    ///
    /// This creates a standard unauthorized error response for when a request
    /// lacks proper authentication.
    ///
    /// # Examples
    /// ```
    /// # use lambda_appsync::AppsyncResponse;
    /// let response = AppsyncResponse::unauthorized();
    /// ```
    pub fn unauthorized() -> Self {
        AppsyncError::new("Unauthorized", "This operation cannot be authorized").into()
    }
}

impl From<Value> for AppsyncResponse {
    fn from(value: Value) -> Self {
        Self {
            data: Some(value),
            error: None,
        }
    }
}
impl From<AppsyncError> for AppsyncResponse {
    fn from(value: AppsyncError) -> Self {
        Self {
            data: None,
            error: Some(value),
        }
    }
}

/// Error type for AWS AppSync operations
///
/// Multiple errors can be combined in one using the pipe operator
///
/// # Example
/// ```
/// # use lambda_appsync::AppsyncError;
/// let combined_error = AppsyncError::new("ValidationError", "Email address is invalid") | AppsyncError::new("DatabaseError", "User not found in database");
/// // error_type: "ValidationError|DatabaseError"
/// // error_message: "Email address is invalid\nUser not found in database"
/// ```
///
/// Can be created from any AWS SDK error or directly by the user.
///
/// # Example
/// ```
/// # use lambda_appsync::AppsyncError;
/// # use aws_sdk_dynamodb::types::AttributeValue;
/// struct Item {
///   id: u64,
///   data: String
/// }
/// async fn store_item(item: Item, client: &aws_sdk_dynamodb::Client) -> Result<(), AppsyncError> {
///     client.put_item()
///         .table_name("my-table")
///         .item("id", AttributeValue::N(item.id.to_string()))
///         .item("data", AttributeValue::S(item.data))
///         .send()
///         .await?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Error, Serialize)]
#[serde(rename_all = "camelCase")]
#[error("{error_type}: {error_message}")]
pub struct AppsyncError {
    /// The type/category of error that occurred (e.g. "ValidationError", "NotFound", "DatabaseError")
    pub error_type: String,
    /// A detailed message describing the specific error condition
    pub error_message: String,
}
impl AppsyncError {
    /// Creates a new AppSync error with the specified error type and message
    ///
    /// # Arguments
    /// * `error_type` - The type/category of the error (e.g. "ValidationError", "NotFound")
    /// * `error_message` - A detailed message describing the error
    ///
    /// # Example
    /// ```
    /// # use lambda_appsync::AppsyncError;
    /// let error = AppsyncError::new("NotFound", "User with ID 123 not found");
    /// ```
    pub fn new(error_type: impl Into<String>, error_message: impl Into<String>) -> Self {
        AppsyncError {
            error_type: error_type.into(),
            error_message: error_message.into(),
        }
    }
}
impl<T: ProvideErrorMetadata> From<T> for AppsyncError {
    fn from(value: T) -> Self {
        let meta = ProvideErrorMetadata::meta(&value);
        AppsyncError {
            error_type: meta.code().unwrap_or("Unknown").to_owned(),
            error_message: meta.message().unwrap_or_default().to_owned(),
        }
    }
}

impl BitOr for AppsyncError {
    type Output = AppsyncError;
    fn bitor(self, rhs: Self) -> Self::Output {
        AppsyncError {
            error_type: format!("{}|{}", self.error_type, rhs.error_type),
            error_message: format!("{}\n{}", self.error_message, rhs.error_message),
        }
    }
}

/// Extracts and deserializes a named argument from a JSON Value into the specified type
///
/// # Arguments
/// * `args` - Mutable reference to a JSON Value containing arguments
/// * `arg_name` - Name of the argument to extract
///
/// # Returns
/// * `Ok(T)` - Successfully deserialized value of type T
/// * `Err(AppsyncError)` - Error if argument is missing or invalid format
///
/// # Examples
/// ```
/// # use serde_json::json;
/// # use lambda_appsync::arg_from_json;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut args = json!({
///     "userId": "123",
///     "count": 5
/// });
///
/// // Extract userId as String
/// let user_id: String = arg_from_json(&mut args, "userId")?;
/// assert_eq!(user_id, "123");
///
/// // Extract count as i32
/// let count: i32 = arg_from_json(&mut args, "count")?;
/// assert_eq!(count, 5);
///
/// // Error case: invalid type
/// let result: Result<String, _> = arg_from_json(&mut args, "count");
/// assert!(result.is_err());
///
/// // Error case: missing argument
/// let result: Result<String, _> = arg_from_json(&mut args, "missing");
/// assert!(result.is_err());
/// # Ok(())
/// # }
/// ```
pub fn arg_from_json<T: DeserializeOwned>(
    args: &mut serde_json::Value,
    arg_name: &'static str,
) -> Result<T, AppsyncError> {
    serde_json::from_value(
        args.get_mut(arg_name)
            .unwrap_or(&mut serde_json::Value::Null)
            .take(),
    )
    .map_err(|e| {
        AppsyncError::new(
            "InvalidArgs",
            format!("Argument \"{arg_name}\" is not the expected format ({e})"),
        )
    })
}

/// Serializes a value into a JSON Value for AppSync responses
///
/// # Arguments
/// * `res` - Value to serialize that implements Serialize
///
/// # Returns
/// JSON Value representation of the input
///
/// # Panics
/// Panics if the value cannot be serialized to JSON. This should never happen
/// for valid AppSync schema objects as generated by the `appsync_lambda_main` proc macro.
///
/// # Examples
/// ```
/// # use serde::Serialize;
/// # use serde_json::json;
/// # use lambda_appsync::res_to_json;
/// #[derive(Serialize)]
/// struct User {
///     id: String,
///     name: String
/// }
///
/// let user = User {
///     id: "123".to_string(),
///     name: "John".to_string()
/// };
///
/// let json = res_to_json(user);
/// assert_eq!(json, json!({
///     "id": "123",
///     "name": "John"
/// }));
///
/// // Simple types also work
/// let num = res_to_json(42);
/// assert_eq!(num, json!(42));
/// ```
pub fn res_to_json<T: Serialize>(res: T) -> serde_json::Value {
    serde_json::to_value(res).expect("Appsync schema objects are JSON compatible")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stdlib::alloc::{
        string::{String, ToString},
        vec,
    };
    use serde_json::json;

    #[test]
    fn test_appsync_auth_strategy() {
        let allow: AppsyncAuthStrategy = serde_json::from_str("\"ALLOW\"").unwrap();
        let deny: AppsyncAuthStrategy = serde_json::from_str("\"DENY\"").unwrap();

        match allow {
            AppsyncAuthStrategy::Allow => (),
            _ => panic!("Expected Allow"),
        }

        match deny {
            AppsyncAuthStrategy::Deny => (),
            _ => panic!("Expected Deny"),
        }
    }

    #[test]
    fn test_appsync_identity_cognito() {
        let json = json!({
            "sub": "user123",
            "username": "testuser",
            "issuer": "https://cognito-idp.region.amazonaws.com/pool_id",
            "defaultAuthStrategy": "ALLOW",
            "sourceIp": ["1.2.3.4"],
            "groups": ["admin", "users"],
            "claims": {
                "email": "user@example.com",
                "custom:role": "developer"
            }
        });

        if let AppsyncIdentity::Cognito(cognito) = serde_json::from_value(json).unwrap() {
            assert_eq!(cognito.sub, "user123");
            assert_eq!(cognito.username, "testuser");
            assert_eq!(
                cognito.issuer,
                "https://cognito-idp.region.amazonaws.com/pool_id"
            );
            assert_eq!(cognito.default_auth_strategy, AppsyncAuthStrategy::Allow);
            assert_eq!(cognito.source_ip, vec!["1.2.3.4"]);
            assert_eq!(
                cognito.groups,
                Some(vec!["admin".to_string(), "users".to_string()])
            );
            assert_eq!(
                cognito.claims,
                json!({
                    "email": "user@example.com",
                    "custom:role": "developer"
                })
            );
        } else {
            panic!("Expected Cognito variant");
        }
    }

    #[test]
    fn test_appsync_identity_iam() {
        let json = json!({
            "accountId": "123456789012",
            "sourceIp": ["1.2.3.4"],
            "username": "IAMUser",
            "userArn": "arn:aws:iam::123456789012:user/IAMUser"
        });

        if let AppsyncIdentity::Iam(iam) = serde_json::from_value(json).unwrap() {
            assert_eq!(iam.account_id, "123456789012");
            assert_eq!(iam.source_ip, vec!["1.2.3.4"]);
            assert_eq!(iam.username, "IAMUser");
            assert_eq!(iam.user_arn, "arn:aws:iam::123456789012:user/IAMUser");
            assert!(iam.federated_identity.is_none());
        } else {
            panic!("Expected IAM variant");
        }
    }

    #[test]
    fn test_appsync_identity_iam_with_cognito() {
        let json = json!({
            "accountId": "123456789012",
            "sourceIp": ["1.2.3.4"],
            "username": "IAMUser",
            "userArn": "arn:aws:iam::123456789012:user/IAMUser",
            "cognitoIdentityId": "region:id",
            "cognitoIdentityPoolId": "region:pool_id",
            "cognitoIdentityAuthType": "authenticated",
            "cognitoIdentityAuthProvider": "cognito-idp.region.amazonaws.com/pool_id"
        });

        if let AppsyncIdentity::Iam(iam) = serde_json::from_value(json).unwrap() {
            assert_eq!(iam.account_id, "123456789012");
            assert_eq!(iam.source_ip, vec!["1.2.3.4"]);
            assert_eq!(iam.username, "IAMUser");
            assert_eq!(iam.user_arn, "arn:aws:iam::123456789012:user/IAMUser");

            let federated = iam.federated_identity.unwrap();
            assert_eq!(federated.identity_id, "region:id");
            assert_eq!(federated.identity_pool_id, "region:pool_id");
            assert!(matches!(
                federated.auth_type,
                CognitoIdentityAuthType::Authenticated
            ));
            assert_eq!(
                federated.auth_provider,
                "cognito-idp.region.amazonaws.com/pool_id"
            );
        } else {
            panic!("Expected IAM variant");
        }
    }

    #[test]
    fn test_appsync_identity_oidc() {
        let json = json!({
            "iss": "https://auth.example.com",
            "sub": "user123",
            "aud": "client123",
            "exp": 1714521210,
            "iat": 1714517610,
            "name": "John Doe",
            "email": "john@example.com",
            "roles": ["admin"],
            "org_id": "org123",
            "custom_claim": "value"
        });

        if let AppsyncIdentity::Oidc(oidc) = serde_json::from_value(json).unwrap() {
            assert_eq!(oidc.iss, "https://auth.example.com");
            assert_eq!(oidc.sub, "user123");
            assert_eq!(oidc.aud, "client123");
            assert_eq!(oidc.exp, 1714521210);
            assert_eq!(oidc.iat, 1714517610);
            assert_eq!(oidc.additional_claims.get("name").unwrap(), "John Doe");
            assert_eq!(
                oidc.additional_claims.get("email").unwrap(),
                "john@example.com"
            );
            assert_eq!(
                oidc.additional_claims.get("roles").unwrap(),
                &json!(["admin"])
            );
            assert_eq!(oidc.additional_claims.get("org_id").unwrap(), "org123");
            assert_eq!(oidc.additional_claims.get("custom_claim").unwrap(), "value");
        } else {
            panic!("Expected OIDC variant");
        }
    }

    #[test]
    fn test_appsync_identity_lambda() {
        let json = json!({
            "resolverContext": {
                "userId": "user123",
                "permissions": ["read", "write"],
                "metadata": {
                    "region": "us-west-2",
                    "environment": "prod"
                }
            }
        });

        if let AppsyncIdentity::Lambda(lambda) = serde_json::from_value(json).unwrap() {
            assert_eq!(
                lambda.resolver_context,
                json!({
                    "userId": "user123",
                    "permissions": ["read", "write"],
                    "metadata": {
                        "region": "us-west-2",
                        "environment": "prod"
                    }
                })
            );
        } else {
            panic!("Expected Lambda variant");
        }
    }

    #[test]
    fn test_appsync_identity_api_key() {
        let json = serde_json::Value::Null;

        if let AppsyncIdentity::ApiKey = serde_json::from_value(json).unwrap() {
            // Test passes if we get the ApiKey variant
        } else {
            panic!("Expected ApiKey variant");
        }
    }

    #[test]
    fn test_appsync_response() {
        let success = AppsyncResponse::from(json!({"field": "value"}));
        assert!(success.data.is_some());
        assert!(success.error.is_none());

        let error = AppsyncResponse::from(AppsyncError::new("TestError", "message"));
        assert!(error.data.is_none());
        assert!(error.error.is_some());
    }

    #[test]
    fn test_appsync_error() {
        let error = AppsyncError::new("TestError", "message");
        assert_eq!(error.error_type, "TestError");
        assert_eq!(error.error_message, "message");

        let error1 = AppsyncError::new("Error1", "msg1");
        let error2 = AppsyncError::new("Error2", "msg2");
        let combined = error1 | error2;

        assert_eq!(combined.error_type, "Error1|Error2");
        assert_eq!(combined.error_message, "msg1\nmsg2");
    }

    #[test]
    fn test_arg_from_json() {
        let mut args = json!({
            "string": "test",
            "number": 42,
            "bool": true
        });

        let s: String = arg_from_json(&mut args, "string").unwrap();
        assert_eq!(s, "test");

        let n: i32 = arg_from_json(&mut args, "number").unwrap();
        assert_eq!(n, 42);

        let b: bool = arg_from_json(&mut args, "bool").unwrap();
        assert!(b);

        let err: Result<String, _> = arg_from_json(&mut args, "missing");
        assert!(err.is_err());
    }

    #[test]
    fn test_res_to_json() {
        #[derive(Serialize)]
        struct Test {
            field: String,
        }

        let test = Test {
            field: "value".to_string(),
        };

        let json = res_to_json(test);
        assert_eq!(json, json!({"field": "value"}));

        assert_eq!(res_to_json(42), json!(42));
        assert_eq!(res_to_json("test"), json!("test"));
    }
}
