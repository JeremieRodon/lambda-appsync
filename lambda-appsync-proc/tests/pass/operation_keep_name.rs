use lambda_appsync::{
    appsync_lambda_main, appsync_operation,
    subscription_filters::{FieldPath, FilterGroup},
    AppsyncError, ID,
};

appsync_lambda_main!("../../../../schema.graphql", exclude_lambda_handler = true);
fn main() {}

// Query operations
#[appsync_operation(query(players), keep_original_function_name)]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    Ok(vec![])
}

#[appsync_operation(query(gameStatus), keep_original_function_name)]
async fn get_game_status() -> Result<GameStatus, AppsyncError> {
    Ok(GameStatus::Started)
}

#[appsync_operation(query(player), keep_original_function_name)]
async fn get_player(id: ID) -> Result<Option<Player>, AppsyncError> {
    Ok(Some(Player {
        id,
        name: "Test".into(),
        team: Team::Rust,
    }))
}

// Mutation operations
#[appsync_operation(mutation(createPlayer), keep_original_function_name)]
async fn create_player(name: String) -> Result<Player, AppsyncError> {
    Ok(Player {
        id: ID::new(),
        name,
        team: Team::Rust,
    })
}

#[appsync_operation(mutation(deletePlayer), keep_original_function_name)]
async fn delete_player(id: ID) -> Result<Player, AppsyncError> {
    Ok(Player {
        id,
        name: "Deleted".into(),
        team: Team::MultiWordsTeam,
    })
}

#[appsync_operation(mutation(setGameStatus), keep_original_function_name)]
async fn set_game_status() -> Result<GameStatus, AppsyncError> {
    Ok(GameStatus::Started)
}

// Subscription operations
#[appsync_operation(subscription(onCreatePlayer), keep_original_function_name)]
async fn on_create_player(name: String) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(Some(FieldPath::new("name")?.contains(name).into()))
}

#[appsync_operation(subscription(onDeletePlayer), keep_original_function_name)]
async fn on_delete_player(id: ID) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(Some(FieldPath::new("id")?.eq(id).into()))
}

#[appsync_operation(subscription(onGameStatusChange), keep_original_function_name)]
async fn on_game_status_change() -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(None)
}

// Test that we can still call the original functions
async fn test_original_fn() {
    let _ = create_player("test".to_string()).await;
    let _ = get_players().await;
    let _ = get_game_status().await;
    let _ = get_player(ID::new()).await;
    let _ = delete_player(ID::new()).await;
    let _ = set_game_status().await;
    let _ = on_create_player("test".to_string()).await;
    let _ = on_delete_player(ID::new()).await;
    let _ = on_game_status_change().await;
}
