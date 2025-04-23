use lambda_appsync::appsync_lambda_main;

appsync_lambda_main!(
    "../../../../schema.graphql",
    exclude_lambda_handler = true,
    // Argument override with invalid type
    field_type_override = Query.player.id: BadId,
);

fn main() {}
