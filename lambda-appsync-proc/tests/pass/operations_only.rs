mod types {
    lambda_appsync::appsync_lambda_main!("../../../../schema.graphql", only_appsync_types = true);
}

use types::*;

// Test generating only operation enums
lambda_appsync::appsync_lambda_main!("../../../../schema.graphql", only_appsync_operations = true);

fn main() {
    // Verify we can use the Operation enum
    let op = Operation::Query(QueryField::Players);
    match op {
        Operation::Query(QueryField::Players) => {}
        _ => panic!("Unexpected operation"),
    }
}
