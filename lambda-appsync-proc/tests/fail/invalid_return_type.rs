use lambda_appsync::{appsync_lambda_main, appsync_operation, AppsyncError};

appsync_lambda_main!("../../../../schema.graphql");

// Wrong return type (String instead of Player)
#[appsync_operation(query(players))]
async fn wrong_return_type() -> Result<String, AppsyncError> {
    Ok("not a Player vector".to_string())
}
