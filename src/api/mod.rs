//! API calling methods to interact with the Waifu Vault API
//!
//! This performs all the logic for upload / download / info / deleting files
//! from the API
//!
//! Also handles the display logic to the terminal
pub mod types;

use std::io::Write;

use crate::cli::{DownloadArgs, TokenArgs, UploadArgs};
use types::ApiResponse;

use anyhow::Result;
use colored::Colorize;
use reqwest::{
    blocking::{multipart::Form, Client},
    StatusCode,
};

/// Main API endpoint to hit
const API: &str = "https://waifuvault.moe/rest";

/// API caller which interacts with the API
///
/// Holds the client to build requests from and the display string
/// to output any info received back
pub struct ApiCaller {
    client: Client,
    info_str: String,
}

impl ApiCaller {
    /// Create a new API Caller instance
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            info_str: String::new(),
        }
    }

    /// Upload a file to the API based on supplied CLI arguments
    pub fn upload(&mut self, args: &UploadArgs) -> Result<()> {
        // Build the request to send
        // Adds any required query parameters if they're passed in the arguments
        let request = {
            let mut r = self
                .client
                .put(API)
                .query(&[("hide_filename", &args.hide_filename)]);

            // Adds the appropriate info to the request based on if it's a file
            // upload or an upload from a URL resource
            if let Some(file) = &args.file {
                let form = Form::new().file("file", file);
                r = r.multipart(form.unwrap());
            } else {
                let url = &args.url.clone().unwrap();
                r = r.form(&[("url", url)]);
            }

            // Add a password query if set
            if let Some(pwd) = &args.password {
                r = r.query(&[("password", pwd)]);
            }

            // Add an expiry for the file if set
            if let Some(expires) = &args.expires {
                r = r.query(&[("expires", expires)]);
            }

            r
        };

        // Send, parse response, display output / errors if any
        let response = request.send()?;
        let status = response.status();
        let response: ApiResponse = response.json()?;

        self.parse_response(response, status);
        self.display();

        Ok(())
    }

    /// Downloads a file from the API based on the CLI arguments supplied
    pub fn download(&mut self, args: &DownloadArgs) -> Result<()> {
        let url = &args.url;

        // Build the request
        // Set the `x-password` header if a password was supplied
        let request = {
            let mut r = self.client.get(url);

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

        // Handle errors when attempting to reach the resource
        match status_code {
            // This means that the password was either wrong or missing when required
            // Just displays the error and returns
            StatusCode::FORBIDDEN => {
                match &args.password {
                    Some(_) => {
                        self.add_info(
                            "The password given for this file is incorrect!"
                                .red()
                                .to_string(),
                        );
                    }
                    None => {
                        self.add_info(
                            "This file is password protected and needs a password to download!"
                                .red()
                                .to_string(),
                        );
                    }
                }
                self.display();

                return Ok(());
            }
            // Everything is good, move onto saving the file
            StatusCode::OK => {}

            // Some other error happened, parse it, display, and return
            _ => {
                let api_response: ApiResponse = response.json()?;
                self.parse_response(api_response, status_code);
                self.display();
                return Ok(());
            }
        }

        // Performs some path manipulation to get the appropriate location to save
        // the file
        let output_location = self.determine_output_location(url, &args.output);

        // Write out the file to disk
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

    /// Displays information about the file from the API
    pub fn info(&mut self, args: &TokenArgs) -> Result<()> {
        // Add the token to the endpoint location
        let api_url = format!("{API}/{}", args.token.clone());

        // Build the request from the given token
        // As this is a CLI app, the datetime is formatted to human-readable by default
        // This could change later but will do for now
        let request = {
            let mut r = self.client.get(api_url);
            r = r.query(&[("formatted", true)]);

            r
        };

        // Send, parse, display
        let response = request.send()?;
        let status = response.status();
        let response: ApiResponse = response.json()?;
        self.parse_response(response, status);
        self.display();

        Ok(())
    }

    /// Delete a file from the API
    ///
    /// Does what it says on the tin tbh
    pub fn delete(&mut self, args: &TokenArgs) -> Result<()> {
        let api_url = format!("{API}/{}", args.token.clone());
        let request = self.client.delete(api_url);
        let response = request.send()?;
        let status_code = response.status();
        let response = response.json()?;

        self.parse_response(response, status_code);
        self.display();

        Ok(())
    }

    /// Helper function to display the actions of the app with a formatted header
    fn display(&self) {
        println!("{}", "--= Waifu Vault Client =--\n".bold().yellow());
        println!("{}", self.info_str);
    }

    /// Adds any information for the user to a single string for display purposes
    fn add_info(&mut self, msg: impl AsRef<str>) {
        self.info_str.push_str(msg.as_ref());
        self.info_str.push('\n');
    }

    /// Parses the responses that can be received from the API
    ///
    /// There are three possible response types:
    ///     * `OkResponse`: Everything is file, contains all information required
    ///     * `BadResponse`: Something went wrong, contains all the errors
    ///     * `DeleteResponse`: A boolean when the delete endpoint is hit
    fn parse_response(&mut self, response: ApiResponse, status_code: StatusCode) {
        match response {
            // Good response from the API, the operation was successful
            ApiResponse::OkResponse {
                token,
                url,
                protected,
                retention_period,
            } => {
                // Since this is a success, the responses allowed are always either
                // 200 OK or a 201 Created
                match status_code {
                    StatusCode::OK => self.add_info("File exists!"),
                    StatusCode::CREATED => self.add_info("File stored successfully!"),
                    _ => unreachable!("it's either a 200 or 201"),
                }

                // The rest here is just formatted info about the file
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
            // Something went wrong, displays the error type and cause
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
            // Special case for the delete endpoint (makes deserializing the responses easier)
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

    /// Helper function to determine the appropriate place to save the downloaded item.
    ///
    /// This will create the appropriate path based on the user-specified location or
    /// will use a default location instead (the current directory)
    fn determine_output_location(&self, url: impl AsRef<str>, output: &Option<String>) -> String {
        // Obtains the filename from the API
        // As the API url always has the filename last, this should always pass
        let filename = match url.as_ref().split('/').last() {
            Some(fname) => fname.to_owned(),
            None => unreachable!("there has to be a filename from the URL"),
        };

        // Get the user-defined location or set a default to the current directory
        let output_location = match output {
            Some(loc) => loc.to_owned(),
            None => format!("./{filename}"),
        };

        // If the user-supplied location is a directory, add the filename
        // else just use what was given
        let path = std::path::PathBuf::from(&output_location);
        let output_location = if path.is_dir() {
            path.join(filename).to_str().unwrap().to_owned()
        } else {
            output_location
        };

        output_location
    }
}
