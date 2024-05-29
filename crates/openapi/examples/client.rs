use restate_openapi::apis::*;
use restate_openapi::models::*;

fn main() {
    let mut openapi_config = configuration::Configuration::default();
    openapi_config.bearer_access_token = Some("my token".to_string());
    openapi_config.base_path = "http://192.168.86.123:8080/".to_string();
}
