use lambda_appsync::{appsync_lambda_main, appsync_operation, AppsyncError};

appsync_lambda_main!("../../../../schema.graphql");

#[appsync_operation(query(nonexistent))]
async fn invalid_operation() -> Result<String, AppsyncError> {
    Ok("test".to_string())
}
