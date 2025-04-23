use lambda_appsync::appsync_lambda_main;

appsync_lambda_main!(
    "../../../../schema.graphql",
    exclude_lambda_handler = true,
    // Non-existent overrides
    field_type_override = Player.inexistant: String,
);

fn main() {}
