use lambda_appsync::{appsync_lambda_main, AppsyncEvent, AppsyncResponse};

async fn verify_request(_event: &AppsyncEvent<Operation>) -> Option<AppsyncResponse> {
    None // Allow all requests
}

appsync_lambda_main!(
    "../../../../schema.graphql",
    exclude_lambda_handler = true,
    hook = verify_request
);

fn main() {}
