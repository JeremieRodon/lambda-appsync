use lambda_appsync::{appsync_lambda_main, appsync_operation, AppsyncError, AppsyncEvent, ID};

appsync_lambda_main!("../../../../schema.graphql");

// Missing argument
#[appsync_operation(mutation(createPlayer), with_appsync_event)]
async fn create_player() -> Result<Player, AppsyncError> {
    Ok(Player {
        id: ID::new(),
        name: "Test".to_string(),
        team: Team::Rust,
    })
}

// Missing op argument
#[appsync_operation(mutation(deletePlayer), with_appsync_event)]
async fn delete_player(_event: &AppsyncEvent<Operation>) -> Result<Player, AppsyncError> {
    Ok(Player {
        id: ID::new(),
        name: "deleted".into(),
        team: Team::MultiWordsTeam,
    })
}

// Missing event argument
#[appsync_operation(query(player), with_appsync_event)]
async fn get_player(_id: ID) -> Result<Option<Player>, AppsyncError> {
    Ok(None)
}
