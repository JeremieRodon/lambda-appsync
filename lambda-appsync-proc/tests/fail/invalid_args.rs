use lambda_appsync::{appsync_lambda_main, appsync_operation, AppsyncError, ID};

appsync_lambda_main!("../../../../schema.graphql");

// Wrong argument type (i32 instead of String)
#[appsync_operation(mutation(createPlayer))]
async fn create_player(name: i32) -> Result<Player, AppsyncError> {
    Ok(Player {
        id: ID::new(),
        name: name.to_string(),
        team: Team::Rust,
    })
}
