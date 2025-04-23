use lambda_appsync::{appsync_lambda_main, appsync_operation, AppsyncError};

// Test field type overrides
appsync_lambda_main!(
    "../../../../schema.graphql",
    exclude_lambda_handler = true,
    // Override Player.id to use String instead of ID
    field_type_override = Player.id: String,
    // Multiple overrides
    field_type_override = Player.team: String,
);

fn main() {}

#[appsync_operation(mutation(createPlayer))]
async fn create_player(name: String) -> Result<Player, AppsyncError> {
    Ok(Player {
        id: "custom-id".to_string(),
        name,
        team: "RUST".to_string(), // Now accepts String directly
    })
}
