// Test with batch processing disabled
lambda_appsync::appsync_lambda_main!(
    "../../../../schema.graphql",
    exclude_lambda_handler = true,
    batch = false
);

fn main() {}
