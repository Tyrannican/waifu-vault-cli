pub(crate) mod types;

use crate::cli::{DeleteArgs, DownloadArgs, InfoArgs, UploadArgs};
use types::{BadResponse, OkResponse};

use reqwest::{multipart::Form, Client};

pub(crate) async fn upload(args: UploadArgs) {}
pub(crate) async fn download(args: DownloadArgs) {}
pub(crate) async fn info(args: InfoArgs) {}
pub(crate) async fn delete(args: DeleteArgs) {}
