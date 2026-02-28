use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterClientRequest {
    pub device_name: String,
    pub device_type: String,
    pub client_version: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterClientResponse {
    pub api_key: String,
}
