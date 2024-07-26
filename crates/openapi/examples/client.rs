use restate_sdk_openapi_client::apis::*;
use restate_sdk_openapi_client::models::*;

fn main() {
    let mut openapi_config = configuration::Configuration::default();
    openapi_config.bearer_access_token = Some("my token".to_string());
    openapi_config.base_path = "http://localhost:3000/".to_string();


}
