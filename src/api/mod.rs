pub(crate) mod types;

use crate::cli::{DeleteArgs, DownloadArgs, InfoArgs, UploadArgs};
use types::ApiResponse;

use anyhow::Result;
use reqwest::{
    blocking::{multipart::Form, Client},
    StatusCode,
};

const API: &'static str = "https://waifuvault.moe/rest";

pub(crate) fn upload(args: &UploadArgs) -> Result<()> {
    let client = Client::new();

    let request = {
        let mut request = client
            .put(API)
            .query(&[("hide_filename", &args.hide_filename)]);

        if let Some(file) = &args.file {
            let form = Form::new().file("file", file);
            request = request.multipart(form.unwrap());
        } else {
            let url = &args.url.clone().unwrap();
            request = request.form(&[("url", url)]);
        }

        if let Some(pwd) = &args.password {
            request = request.query(&[("password", pwd)]);
        }

        if let Some(expires) = &args.expires {
            request = request.query(&[("expires", expires)]);
        }

        request
    };

    let response = request.send()?;
    let status = response.status();
    let response: ApiResponse = response.json()?;

    parse_response(response, status);
    Ok(())
}

pub(crate) async fn download(args: DownloadArgs) {}
pub(crate) async fn info(args: InfoArgs) {}
pub(crate) async fn delete(args: DeleteArgs) {}

fn parse_response(response: ApiResponse, status_code: StatusCode) {
    println!("--= Waifu Vault Client =--\n");
    match response {
        ApiResponse::OkResponse {
            token,
            url,
            protected,
            retention_period,
        } => {
            if status_code == StatusCode::OK {
                println!("File status: File already exists!");
            } else if status_code == StatusCode::CREATED {
                println!("File status: File stored successfully!");
            } else {
                unreachable!("it's either 200 or 201");
            };

            println!("It is stored at {url}");
            println!("It has the unique token {token}");
            if protected {
                println!("It is a PROTECTED file");
            } else {
                println!("It is an UNPROTECTED file");
            }
            println!("It is available for {retention_period}");
        }
        ApiResponse::BadResponse {
            name,
            message,
            status: _,
            errors,
        } => {
            println!("Received a bad response from API: {name}");
            println!("This is probably due to {message}");
            println!("More error info:");
            for error in errors {
                println!("Error: {} - Message: {}", error.name, error.message);
            }
        }
        ApiResponse::Delete(result) => {
            if result {
                println!("File deleted successfully!");
            } else {
                println!("File was NOT deleted successfully...");
            }
        }
    }
    println!();
}
