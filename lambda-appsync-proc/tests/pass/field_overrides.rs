use lambda_appsync::{
    appsync_lambda_main, appsync_operation, subscription_filters::FilterGroup, AppsyncError,
};

// Test field type overrides
appsync_lambda_main!(
    "../../../../schema.graphql",
    exclude_lambda_handler = true,
    // Override Player.id to use String instead of ID
    type_override = Player.id: String,
    // Multiple overrides
    type_override = Player.team: String,
    // Return value override
    type_override = Query.gameStatus: String,
    type_override = Mutation.setGameStatus: String,
    // Argument override
    type_override = Query.player.id: String,
    type_override = Mutation.deletePlayer.id: String,
    type_override = Subscription.onDeletePlayer.id: String,
);

fn main() {}

// Id is now a string
#[appsync_operation(query(player))]
async fn get_player(id: String) -> Result<Option<Player>, AppsyncError> {
    Ok(Some(Player {
        id,
        name: "JohnDoe".to_string(),
        team: "RUST".to_string(), // Now accepts String directly
    }))
}

// Id is now a string
#[appsync_operation(mutation(deletePlayer))]
async fn delete_player(id: String) -> Result<Player, AppsyncError> {
    Ok(Player {
        id,
        name: "deleted".into(),
        team: "RUST".to_string(), // Now accepts String directly
    })
}

// Id is now a string
#[appsync_operation(subscription(onDeletePlayer))]
async fn on_delete_player(id: String) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(None)
}

// setGameStatus now expects a String
#[appsync_operation(mutation(setGameStatus))]
async fn set_game_status() -> Result<String, AppsyncError> {
    Ok("Started".to_owned())
}
