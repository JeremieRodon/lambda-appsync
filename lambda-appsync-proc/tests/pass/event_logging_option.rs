mod no_run {
    // Test with event_logging disabled
    lambda_appsync::appsync_lambda_main!("../../../../schema.graphql", event_logging = false);
}

fn main() {}
