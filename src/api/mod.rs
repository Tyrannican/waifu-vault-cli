pub mod types;

use std::io::Write;

use crate::cli::{DeleteArgs, DownloadArgs, InfoArgs, UploadArgs};
use types::ApiResponse;

use anyhow::Result;
use colored::Colorize;
use reqwest::{
    blocking::{multipart::Form, Client},
    StatusCode,
};

const API: &str = "https://waifuvault.moe/rest";

pub struct ApiCaller {
    client: Client,
    info_str: String,
}

impl ApiCaller {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            info_str: String::new(),
        }
    }

    pub fn upload(&mut self, args: &UploadArgs) -> Result<()> {
        let request = {
            let mut r = self
                .client
                .put(API)
                .query(&[("hide_filename", &args.hide_filename)]);

            if let Some(file) = &args.file {
                let form = Form::new().file("file", file);
                r = r.multipart(form.unwrap());
            } else {
                let url = &args.url.clone().unwrap();
                r = r.form(&[("url", url)]);
            }

            if let Some(pwd) = &args.password {
                r = r.query(&[("password", pwd)]);
            }

            if let Some(expires) = &args.expires {
                r = r.query(&[("expires", expires)]);
            }

            r
        };

        let response = request.send()?;
        let status = response.status();
        let response: ApiResponse = response.json()?;

        self.parse_response(response, status);
        self.display();

        Ok(())
    }

    pub fn download(&mut self, args: &DownloadArgs) -> Result<()> {
        let api_url = format!("{API}/{}", args.token.clone());
        let response: ApiResponse = self
            .client
            .get(api_url)
            .query(&[("formatted", true)])
            .send()?
            .json()?;

        let parsed_response = self.parse_download_response(response);

        if parsed_response.is_none() {
            self.display();

            return Ok(());
        }

        let (url, protected) = parsed_response.unwrap();
        let output_location = match &args.output {
            Some(output) => output.to_owned(),
            None => format!("./{}", &args.token),
        };

        let request = {
            let mut r = self.client.get(url);
            if protected && args.password.is_none() {
                self.add_info(
                    "This file is password protected and requires a password!"
                        .red()
                        .to_string(),
                );
                self.display();

                return Ok(());
            }

            match &args.password {
                Some(pwd) => {
                    r = r.header("x-password", pwd);
                }
                None => {}
            }

            r
        };

        let response = request.send()?;
        let status_code = response.status();

        if status_code != StatusCode::OK {
            let api_response: ApiResponse = response.json()?;
            self.parse_response(api_response, status_code);
            self.display();
            return Ok(());
        }

        // Filepath sanity
        let output_location = if std::path::PathBuf::from(&output_location).is_dir() {
            format!("{output_location}/wvc_file")
        } else {
            output_location
        };

        let mut fh = std::fs::File::create(&output_location)?;
        let contents = response.bytes()?;
        fh.write_all(&contents)?;

        self.add_info(format!(
            "File downloaded successfully and stored at {}!",
            output_location.bright_green().bold()
        ));
        self.display();

        Ok(())
    }

    pub fn info(&mut self, args: &InfoArgs) -> Result<()> {
        let api_url = format!("{API}/{}", args.token.clone());
        let request = {
            let mut r = self.client.get(api_url);
            r = r.query(&[("formatted", true)]);

            r
        };

        let response = request.send()?;
        let status = response.status();
        let response: ApiResponse = response.json()?;
        self.parse_response(response, status);
        self.display();

        Ok(())
    }

    pub fn delete(&mut self, args: &DeleteArgs) -> Result<()> {
        let api_url = format!("{API}/{}", args.token.clone());
        let request = self.client.delete(api_url);
        let response = request.send()?;
        let status_code = response.status();
        let response = response.json()?;

        self.parse_response(response, status_code);
        self.display();

        Ok(())
    }

    fn display(&self) {
        println!("{}", "--= Waifu Vault Client =--\n".bold().yellow());
        println!("{}", self.info_str);
    }

    fn add_info(&mut self, msg: impl AsRef<str>) {
        self.info_str.push_str(msg.as_ref());
        self.info_str.push('\n');
    }

    fn parse_response(&mut self, response: ApiResponse, status_code: StatusCode) {
        match response {
            ApiResponse::OkResponse {
                token,
                url,
                protected,
                retention_period,
            } => {
                match status_code {
                    StatusCode::OK => self.add_info("File exists!"),
                    StatusCode::CREATED => self.add_info("File stored successfully!"),
                    _ => unreachable!("it's either a 200 or 201"),
                }

                self.add_info(format!("It is stored at {}", url.bright_cyan().bold()));
                self.add_info(format!(
                    "It has the unique token: {}",
                    token.bright_white().bold()
                ));
                if protected {
                    self.add_info(format!(
                        "It is a {} file",
                        "PROTECTED".bright_magenta().bold()
                    ));
                } else {
                    self.add_info(format!(
                        "It is an {} file",
                        "UNPROTECTED".bright_blue().bold()
                    ));
                }
                self.add_info(format!(
                    "It is available for {}",
                    retention_period.bright_green().bold()
                ));
            }
            ApiResponse::BadResponse {
                name,
                message,
                status: _,
            } => {
                self.add_info(format!(
                    "Received a bad response from API: {}",
                    name.red().bold()
                ));
                self.add_info(format!(
                    "This is probably due to: {}",
                    message.bright_yellow().bold()
                ));
            }
            ApiResponse::Delete(result) => {
                if result {
                    self.add_info("File deleted successfully!".bright_green().to_string());
                } else {
                    self.add_info(
                        "File was NOT deleted successfully..."
                            .bright_red()
                            .to_string(),
                    );
                }
            }
        }
    }

    fn parse_download_response(&mut self, response: ApiResponse) -> Option<(String, bool)> {
        match response {
            ApiResponse::OkResponse {
                token: _,
                url,
                protected,
                retention_period: _,
            } => Some((url, protected)),
            ApiResponse::BadResponse {
                name,
                message,
                status: _,
            } => {
                self.add_info(format!(
                    "Received a bad response from API: {}",
                    name.bright_red().bold()
                ));
                self.add_info(format!(
                    "This is probably due to: {}",
                    message.bright_yellow().bold()
                ));

                None
            }
            _ => None,
        }
    }
}
