//! API types that can be received from the Waifu Vault API
use serde::Deserialize;

/// The main API responses that can be received
///
/// Serde will deserialize these into the appropriate type
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ApiResponse {
    // Everything is good, contains the info about the uploaded file
    OkResponse {
        token: String,
        url: String,
        protected: bool,

        #[serde(rename = "retentionPeriod")]
        retention_period: String,
    },
    // Something went wrong, shows the error type and reason
    BadResponse {
        name: String,
        message: String,
        status: u16,
    },
    /// Special case for the delete endpoint, just a boolean flag
    /// if it was successful or not
    Delete(bool),
}
