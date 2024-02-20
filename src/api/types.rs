use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub(crate) struct OkResponse {
    pub token: String,
    pub url: String,
    pub protected: bool,
    pub retention: u64,
}

#[derive(Deserialize, Debug)]
pub(crate) struct BadResponse {
    pub name: String,
    pub message: String,
    pub status: u16,
    pub errors: Vec<ApiError>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ApiError {
    pub name: String,
    pub message: String,
}
