use lambda_appsync::appsync_lambda_main;

appsync_lambda_main!(
    "../../../../schema.graphql",
    exclude_lambda_handler = true,
    type_override = Mutation.invalidMutation: String,
);

fn main() {}
