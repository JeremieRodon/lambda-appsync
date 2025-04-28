use std::{cell::RefCell, collections::HashMap, ops::Deref};

use lambda_appsync::{
    appsync_lambda_main, appsync_operation, AppsyncError, AppsyncEvent, AppsyncIdentity,
    AppsyncIdentityCognito, AppsyncResponse, ID,
};
use serde_json::json;

thread_local! {
    static TEST_DB: InnerDatabase = InnerDatabase::new();
}

struct InnerDatabase(RefCell<HashMap<ID, Player>>);
impl InnerDatabase {
    pub fn new() -> Self {
        Self(RefCell::new(HashMap::new()))
    }

    fn get(&self, id: &ID) -> Option<Player> {
        self.0.borrow().get(id).cloned()
    }

    fn insert(&self, id: ID, player: Player) -> Option<Player> {
        self.0.borrow_mut().insert(id, player)
    }

    fn remove(&self, id: &ID) -> Option<Player> {
        self.0.borrow_mut().remove(id)
    }

    fn values(&self) -> Vec<Player> {
        self.0.borrow().values().cloned().collect()
    }
}
struct Database;
impl Deref for Database {
    type Target = InnerDatabase;

    fn deref(&self) -> &Self::Target {
        // Safety: This is safe because we're only accessing the thread-local
        // storage, which is guaranteed to exist for the duration of the thread
        TEST_DB.with(|db| unsafe {
            // Convert the reference to 'static because we know the thread-local
            // storage will live for the remainder of the program
            std::mem::transmute::<&InnerDatabase, &'static InnerDatabase>(db)
        })
    }
}

// Generate AppSync types and runtime from schema
appsync_lambda_main!(
    "schema.graphql",
    // Add auth hook for Cognito group testing
    hook = verify_request
);

// Auth hook for testing group-based access
async fn verify_request(event: &AppsyncEvent<Operation>) -> Option<AppsyncResponse> {
    if let AppsyncIdentity::Cognito(AppsyncIdentityCognito { groups, .. }) = &event.identity {
        // Require "admin" group for deletePlayer operation
        if let Operation::Mutation(MutationField::DeletePlayer) = event.info.operation {
            if !groups.iter().flatten().any(|g| g == "admin") {
                return Some(AppsyncResponse::unauthorized());
            }
        }
    }
    None
}

// Operation Handlers
#[appsync_operation(query(players))]
async fn get_players() -> Result<Vec<Player>, AppsyncError> {
    Ok(Database.values())
}

#[appsync_operation(query(player))]
async fn get_player(id: ID) -> Result<Option<Player>, AppsyncError> {
    Ok(Database.get(&id))
}

#[appsync_operation(mutation(createPlayer))]
async fn create_player(name: String) -> Result<Player, AppsyncError> {
    let player = Player {
        id: ID::new(),
        name,
        team: Team::Rust,
    };

    Database.insert(player.id, player.clone());

    Ok(player)
}

#[appsync_operation(mutation(deletePlayer))]
async fn delete_player(id: ID) -> Result<Player, AppsyncError> {
    Database
        .remove(&id)
        .ok_or_else(|| AppsyncError::new("NotFound", "Player not found"))
}

// Tests
#[tokio::test]
async fn test_create_and_get_player() {
    // Create player
    let create_event = json!([{
        "info": {
            "fieldName": "createPlayer",
            "parentTypeName": "Mutation",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {
            "name": "Test Player"
        },
        "identity": null,
        "request": null,
        "source": null
    }]);

    let create_lambda_event = lambda_runtime::LambdaEvent::new(create_event, Default::default());
    let create_response = function_handler(create_lambda_event).await.unwrap();

    let response_value = serde_json::to_value(create_response).unwrap();
    let player_id = response_value[0]["data"]["id"].as_str().unwrap();

    // Get created player
    let get_event = json!([{
        "info": {
            "fieldName": "player",
            "parentTypeName": "Query",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {
            "id": player_id
        },
        "identity": null,
        "request": null,
        "source": null
    }]);

    let get_lambda_event = lambda_runtime::LambdaEvent::new(get_event, Default::default());
    let get_response = function_handler(get_lambda_event).await.unwrap();

    let response_value = serde_json::to_value(get_response).unwrap();
    assert_eq!(response_value[0]["data"]["name"], "Test Player");
}

#[tokio::test]
async fn test_get_nonexistent_player() {
    let event = json!([{
        "info": {
            "fieldName": "player",
            "parentTypeName": "Query",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {
            "id": ID::new().to_string()
        },
        "identity": null,
        "request": null,
        "source": null
    }]);

    let lambda_event = lambda_runtime::LambdaEvent::new(event, Default::default());
    let response = function_handler(lambda_event).await.unwrap();

    let response_value = serde_json::to_value(response).unwrap();
    assert!(response_value[0]["data"].is_null());
}

#[tokio::test]
async fn test_cognito_auth_no_groups() {
    let player = Player {
        id: ID::new(),
        name: "Test Player".to_string(),
        team: Team::Rust,
    };
    Database.insert(player.id, player.clone());

    let event = json!([{
        "info": {
            "fieldName": "deletePlayer",
            "parentTypeName": "Mutation",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {
            "id": player.id.to_string()
        },
        "identity": {
            "sub": "user123",
            "issuer": "cognito",
            "username": "testuser",
            "claims": {},
            "sourceIp": ["1.1.1.1"],
            "defaultAuthStrategy": "ALLOW"
        },
        "request": null,
        "source": null
    }]);

    let lambda_event = lambda_runtime::LambdaEvent::new(event, Default::default());
    let response = function_handler(lambda_event).await.unwrap();

    let response_value = serde_json::to_value(response).unwrap();
    assert_eq!(response_value[0]["errorType"], "Unauthorized");
}

#[tokio::test]
async fn test_cognito_auth_with_admin_group() {
    let player = Player {
        id: ID::new(),
        name: "Test Player".to_string(),
        team: Team::Rust,
    };
    Database.insert(player.id, player.clone());

    let event = json!([{
        "info": {
            "fieldName": "deletePlayer",
            "parentTypeName": "Mutation",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {
            "id": player.id.to_string()
        },
        "identity": {
            "sub": "user123",
            "issuer": "cognito",
            "username": "testuser",
            "claims": {},
            "groups": ["admin"],
            "sourceIp": ["1.1.1.1"],
            "defaultAuthStrategy": "ALLOW"
        },
        "request": null,
        "source": null
    }]);

    let lambda_event = lambda_runtime::LambdaEvent::new(event, Default::default());
    let response = function_handler(lambda_event).await.unwrap();

    let response_value = serde_json::to_value(response).unwrap();
    assert_eq!(response_value[0]["data"]["id"], player.id.to_string());
}

#[tokio::test]
async fn test_iam_auth() {
    let event = json!([{
        "info": {
            "fieldName": "players",
            "parentTypeName": "Query",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {},
        "identity": {
            "accountId": "123456789012",
            "sourceIp": ["1.1.1.1"],
            "username": "IAMUser",
            "userArn": "arn:aws:iam::123456789012:user/IAMUser"
        },
        "request": null,
        "source": null
    }]);

    let lambda_event = lambda_runtime::LambdaEvent::new(event, Default::default());
    let response = function_handler(lambda_event).await.unwrap();

    let response_value = serde_json::to_value(response).unwrap();
    assert!(response_value[0].get("data").is_some_and(|v| v.is_array()));
}

#[tokio::test]
async fn test_unimplemented_operation() {
    // Test setGameStatus mutation
    let set_status_event = json!([{
        "info": {
            "fieldName": "setGameStatus",
            "parentTypeName": "Mutation",
            "variables": {},
            "selectionSetList": [],
            "selectionSetGraphQL": ""
        },
        "arguments": {},
        "identity": null,
        "request": null,
        "source": null
    }]);

    let set_status_lambda_event =
        lambda_runtime::LambdaEvent::new(set_status_event, Default::default());
    let set_status_response = function_handler(set_status_lambda_event).await.unwrap();

    // Verify status was set
    let response_value = serde_json::to_value(set_status_response).unwrap();
    assert_eq!(response_value[0]["errorType"], "Unimplemented");
    assert_eq!(
        response_value[0]["errorMessage"],
        "Mutation `setGameStatus` is unimplemented"
    );
}

#[tokio::test]
async fn test_delete_nonexistent_player() {
    let event = json!([{
        "info": {
            "fieldName": "deletePlayer",
            "parentTypeName": "Mutation",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {
            "id": ID::new().to_string()
        },
        "identity": {
            "sub": "user123",
            "issuer": "cognito",
            "username": "testuser",
            "claims": {},
            "groups": ["admin"],
            "sourceIp": ["1.1.1.1"],
            "defaultAuthStrategy": "ALLOW"
        },
        "request": null,
        "source": null
    }]);

    let lambda_event = lambda_runtime::LambdaEvent::new(event, Default::default());
    let response = function_handler(lambda_event).await.unwrap();

    let response_value = serde_json::to_value(response).unwrap();
    assert_eq!(response_value[0]["errorType"], "NotFound");
    assert_eq!(response_value[0]["errorMessage"], "Player not found");
}

#[tokio::test]
async fn test_get_inexistent_player() {
    let event = json!([{
        "info": {
            "fieldName": "player",
            "parentTypeName": "Query",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {
            "id": ID::new().to_string()
        },
        "identity": null,
        "request": null,
        "source": null
    }]);

    let lambda_event = lambda_runtime::LambdaEvent::new(event, Default::default());
    let response = function_handler(lambda_event).await.unwrap();

    let response_value = serde_json::to_value(response).unwrap();
    assert!(response_value[0].get("data").is_none() || response_value["data"].is_null());
}

#[tokio::test]
async fn test_create_multiple_players_in_batch() {
    // Test creating multiple players and retrieving them all
    let names = vec!["Player 1", "Player 2", "Player 3"];

    // Create multiple players
    let create_event = json!(names
        .iter()
        .map(|name| json!({
            "info": {
                "fieldName": "createPlayer",
                "parentTypeName": "Mutation",
                "variables": {},
                "selectionSetList": ["id", "name", "team"],
                "selectionSetGraphQL": "{id name team}"
            },
            "arguments": {
                "name": name
            },
            "identity": null,
            "request": null,
            "source": null
        }))
        .collect::<Vec<_>>());

    let create_lambda_event = lambda_runtime::LambdaEvent::new(create_event, Default::default());
    let response = function_handler(create_lambda_event).await.unwrap();

    // Verify each creation was successful
    let response_values = serde_json::to_value(response).unwrap();
    for (response_value, name) in response_values.as_array().unwrap().iter().zip(names.iter()) {
        assert_eq!(response_value["data"]["name"], *name);
    }

    // Get all players
    let get_all_event = json!([{
        "info": {
            "fieldName": "players",
            "parentTypeName": "Query",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {},
        "identity": null,
        "request": null,
        "source": null
    }]);

    let get_all_lambda_event = lambda_runtime::LambdaEvent::new(get_all_event, Default::default());
    let response = function_handler(get_all_lambda_event).await.unwrap();

    // Verify all players were retrieved
    let response_value = serde_json::to_value(response).unwrap();
    let players = response_value[0]["data"].as_array().unwrap();
    assert_eq!(players.len(), names.len());

    // Verify all created names are present
    let retrieved_names: Vec<String> = players
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_string())
        .collect();

    for name in names {
        assert!(retrieved_names.contains(&name.to_string()));
    }
}

#[tokio::test]
async fn test_delete_player_twice() {
    // Create a player
    let player = Player {
        id: ID::new(),
        name: "Test Player".to_string(),
        team: Team::Rust,
    };
    Database.insert(player.id, player.clone());

    let delete_event = json!([{
        "info": {
            "fieldName": "deletePlayer",
            "parentTypeName": "Mutation",
            "variables": {},
            "selectionSetList": ["id", "name", "team"],
            "selectionSetGraphQL": "{id name team}"
        },
        "arguments": {
            "id": player.id.to_string()
        },
        "identity": {
            "sub": "user123",
            "issuer": "cognito",
            "username": "testuser",
            "claims": {},
            "groups": ["admin"],
            "sourceIp": ["1.1.1.1"],
            "defaultAuthStrategy": "ALLOW"
        },
        "request": null,
        "source": null
    }]);

    // First delete should succeed
    let lambda_event = lambda_runtime::LambdaEvent::new(delete_event.clone(), Default::default());
    let response = function_handler(lambda_event).await.unwrap();
    let response_value = serde_json::to_value(response).unwrap();
    assert_eq!(response_value[0]["data"]["id"], player.id.to_string());

    // Second delete should fail with NotFound
    let lambda_event = lambda_runtime::LambdaEvent::new(delete_event, Default::default());
    let response = function_handler(lambda_event).await.unwrap();
    let response_value = serde_json::to_value(response).unwrap();
    assert_eq!(response_value[0]["errorType"], "NotFound");
}
