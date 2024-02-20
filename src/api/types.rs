use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum ApiResponse {
    OkResponse {
        token: String,
        url: String,
        protected: bool,

        #[serde(rename = "retentionPeriod")]
        retention_period: String,
    },
    BadResponse {
        name: String,
        message: String,
        status: u16,
    },
    Delete(bool),
}
