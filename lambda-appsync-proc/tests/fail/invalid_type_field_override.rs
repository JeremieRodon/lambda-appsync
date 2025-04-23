use lambda_appsync::appsync_lambda_main;

appsync_lambda_main!(
    "../../../../schema.graphql",
    exclude_lambda_handler = true,
    // Override Player.id with invalid type
    field_type_override = Player.id: NonExistentType,
);

fn main() {}
