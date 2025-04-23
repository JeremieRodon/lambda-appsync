use lambda_appsync::{
    appsync_lambda_main, appsync_operation,
    subscription_filters::{FieldPath, FilterGroup},
    AppsyncError, AppsyncEvent, ID,
};

appsync_lambda_main!("../../../../schema.graphql", exclude_lambda_handler = true);
fn main() {}

// Queries
#[appsync_operation(query(players), with_appsync_event)]
async fn get_players(event: &AppsyncEvent<Operation>) -> Result<Vec<Player>, AppsyncError> {
    Ok(vec![])
}

#[appsync_operation(query(gameStatus), with_appsync_event)]
async fn get_game_status(event: &AppsyncEvent<Operation>) -> Result<GameStatus, AppsyncError> {
    Ok(GameStatus::Started)
}

#[appsync_operation(query(player), with_appsync_event)]
async fn get_player(
    id: ID,
    event: &AppsyncEvent<Operation>,
) -> Result<Option<Player>, AppsyncError> {
    Ok(Some(Player {
        id,
        name: "test".into(),
        team: Team::Rust,
    }))
}

// Mutations
#[appsync_operation(mutation(createPlayer), with_appsync_event)]
async fn create_player(
    name: String,
    event: &AppsyncEvent<Operation>,
) -> Result<Player, AppsyncError> {
    Ok(Player {
        id: ID::new(),
        name,
        team: Team::Python,
    })
}

#[appsync_operation(mutation(deletePlayer), with_appsync_event)]
async fn delete_player(id: ID, event: &AppsyncEvent<Operation>) -> Result<Player, AppsyncError> {
    Ok(Player {
        id,
        name: "deleted".into(),
        team: Team::Js,
    })
}

#[appsync_operation(mutation(setGameStatus), with_appsync_event)]
async fn set_game_status(event: &AppsyncEvent<Operation>) -> Result<GameStatus, AppsyncError> {
    Ok(GameStatus::Stopped)
}

// Subscriptions
#[appsync_operation(subscription(onCreatePlayer), with_appsync_event)]
async fn on_create_player(
    name: String,
    event: &AppsyncEvent<Operation>,
) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(Some(FieldPath::new("name")?.contains(name).into()))
}

#[appsync_operation(subscription(onDeletePlayer), with_appsync_event)]
async fn on_delete_player(
    id: ID,
    event: &AppsyncEvent<Operation>,
) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(Some(FieldPath::new("id")?.eq(id).into()))
}

#[appsync_operation(subscription(onGameStatusChange), with_appsync_event)]
async fn on_game_status_change(
    event: &AppsyncEvent<Operation>,
) -> Result<Option<FilterGroup>, AppsyncError> {
    Ok(Some(
        FieldPath::new("status")?
            .eq(GameStatus::Started.to_string())
            .into(),
    ))
}
