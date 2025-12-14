mod no_run {
    use lambda_appsync::{appsync_lambda_main, AppsyncEvent, AppsyncResponse};
    fn verify_request(_event: &AppsyncEvent<Operation>) -> Option<AppsyncResponse> {
        None // Allow all requests
    }
    appsync_lambda_main!("../../../../schema.graphql", hook = verify_request);
}

fn main() {}
