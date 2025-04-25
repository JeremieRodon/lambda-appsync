use lambda_appsync::appsync_lambda_main;

appsync_lambda_main!(
    "../../../../schema.graphql",
    exclude_lambda_handler = true,
    // Non-existent overrides
    name_override = Inexistant.field: NewName,
);

fn main() {}
