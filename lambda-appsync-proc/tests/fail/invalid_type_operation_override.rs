use lambda_appsync::appsync_lambda_main;

appsync_lambda_main!(
    "../../../../schema.graphql",
    exclude_lambda_handler = true,
    // Return value override with invalid type
    field_type_override = Query.gameStatus: InvalidStatus,
);

fn main() {}
