use lambda_appsync::{appsync_lambda_main, appsync_operation, AppsyncError, ID};

appsync_lambda_main!("../../../../schema.graphql");

// Missing argument
#[appsync_operation(mutation(createPlayer))]
async fn create_player() -> Result<Player, AppsyncError> {
    Ok(Player {
        id: ID::new(),
        name: "Test".to_string(),
        team: Team::Rust,
    })
}
