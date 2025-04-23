use lambda_appsync::appsync_lambda_main;

appsync_lambda_main!(
    "../../../../schema.graphql",
    exclude_lambda_handler = true,
    // Non-existent operation arg overrides
    field_type_override = Query.player.invalidArg: String,
);

fn main() {}
