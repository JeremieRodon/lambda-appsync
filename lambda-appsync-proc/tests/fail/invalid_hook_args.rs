mod no_run {
    use lambda_appsync::{appsync_lambda_main, AppsyncResponse};
    async fn verify_request(_event: &Operation) -> Option<AppsyncResponse> {
        None // Allow all requests
    }
    appsync_lambda_main!("../../../../schema.graphql", hook = verify_request);
}

fn main() {}
