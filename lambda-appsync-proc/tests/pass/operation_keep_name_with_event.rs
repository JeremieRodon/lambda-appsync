use lambda_appsync::{
    appsync_lambda_main, appsync_operation,
    subscription_filters::{FieldPath, FilterGroup},
    AppsyncError, AppsyncEvent, ID,
};

appsync_lambda_main!("../../../../schema.graphql", exclude_lambda_handler = true);
fn main() {}

// Query Operations
#[appsync_operation(query(players), keep_original_function_name, with_appsync_event)]
async fn get_players(_event: &AppsyncEvent<Operation>) -> Result<Vec<Player>, AppsyncError> {
    Ok(vec![])
}

#[appsync_operation(query(gameStatus), keep_original_function_name, with_appsync_event)]
async fn get_game_status(_event: &AppsyncEvent<Operation>) -> Result<GameStatus, AppsyncError> {
    Ok(GameStatus::Started)
}

#[appsync_operation(query(player), keep_original_function_name, with_appsync_event)]
async fn get_player(
    id: ID,
    _event: &AppsyncEvent<Operation>,
) -> Result<Option<Player>, AppsyncError> {
    Ok(Some(Player {
        id,
        name: "test".into(),
        team: Team::Python,
    }))
}

// Mutation Operations
#[appsync_operation(
    mutation(createPlayer),
    keep_original_function_name,
    with_appsync_event
)]
async fn create_player(
    name: String,
    _event: &AppsyncEvent<Operation>,
) -> Result<Player, AppsyncError> {
    Ok(Player {
        id: ID::new(),
        name,
        team: Team::Rust,
    })
}

#[appsync_operation(
    mutation(deletePlayer),
    keep_original_function_name,
    with_appsync_event
)]
async fn delete_player(id: ID, _event: &AppsyncEvent<Operation>) -> Result<Player, AppsyncError> {
    Ok(Player {
        id,
        name: "deleted".into(),
        team: Team::Js,
    })
}

#[appsync_operation(
    mutation(setGameStatus),
    keep_original_function_name,
    with_appsync_event
)]
async fn set_game_status(_event: &AppsyncEvent<Operation>) -> Result<GameStatus, AppsyncError> {
    Ok(GameStatus::Started)
}

// Subscription Operations
#[appsync_operation(
    subscription(onCreatePlayer),
    keep_original_function_name,
    with_appsync_event
)]
async fn on_create_player(
    name: String,
    _event: &AppsyncEvent<Operation>,
) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(Some(FieldPath::new("name")?.contains(name).into()))
}

#[appsync_operation(
    subscription(onDeletePlayer),
    keep_original_function_name,
    with_appsync_event
)]
async fn on_delete_player(
    id: ID,
    _event: &AppsyncEvent<Operation>,
) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(Some(FieldPath::new("id")?.eq(id).into()))
}

#[appsync_operation(
    subscription(onGameStatusChange),
    keep_original_function_name,
    with_appsync_event
)]
async fn on_game_status_change(
    _event: &AppsyncEvent<Operation>,
) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(None)
}

// Test that we can still call the original functions with the event parameter
async fn test_original_fn(event: &AppsyncEvent<Operation>) {
    let _ = create_player("test".to_string(), event).await;
    let _ = get_players(event).await;
    let _ = get_game_status(event).await;
    let _ = get_player(ID::new(), event).await;
    let _ = delete_player(ID::new(), event).await;
    let _ = set_game_status(event).await;
    let _ = on_create_player("test".into(), event).await;
    let _ = on_delete_player(ID::new(), event).await;
    let _ = on_game_status_change(event).await;
}
