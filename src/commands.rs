use crate::cli::{DownloadArgs, ModificationArgs, UploadArgs};

use anyhow::{Context, Result};
use colored::Colorize;
use std::fmt::Write;
use tokio::io::AsyncWriteExt;
use waifuvault::{api::*, ApiCaller};

pub async fn upload_file(args: UploadArgs, caller: ApiCaller) -> Result<()> {
    let mut request = WaifuUploadRequest::new()
        .one_time_download(args.one_time_download)
        .hide_filename(args.hide_filename);
    if let Some(file) = args.file {
        request = request.file(file);
    } else {
        let url = args.url.expect("this should be present");
        request = request.url(url);
    }

    if let Some(password) = args.password {
        request = request.password(password);
    }

    if let Some(expires) = args.expires {
        request = request.expires(expires);
    }

    match caller.upload_file(request).await.context("uploading file") {
        Ok(response) => format_response(response)?,
        Err(err) => format_error(err).context("parsing error")?,
    }

    Ok(())
}

pub async fn download_file(args: DownloadArgs, caller: ApiCaller) -> Result<()> {
    match caller
        .download_file(&args.url, args.password)
        .await
        .context("downloading file content")
    {
        Ok(content) => {
            let output_location = determine_output_location(&args.url, args.output);
            let mut file = tokio::fs::File::create(&output_location)
                .await
                .with_context(|| format!("creating file {}", output_location))?;
            file.write_all(&content).await?;

            let mut writer = make_header();
            writeln!(
                writer,
                "File download successful!\nIt is stored at {}",
                output_location.bright_green().bold()
            )?;

            print!("{writer}");
        }
        Err(err) => format_error(err).context("parsing error")?,
    }

    Ok(())
}

pub async fn modify_file(args: ModificationArgs, caller: ApiCaller) -> Result<()> {
    let mut request = WaifuModificationRequest::new(args.token);
    if let Some(password) = args.password {
        request = request.password(password);
    }

    if let Some(prev) = args.previous_password {
        request = request.previous_password(prev);
    }

    if let Some(exp) = args.custom_expiry {
        request = request.custom_expiry(exp);
    }

    if let Some(hide) = args.hide_filename {
        request = request.hide_filename(hide);
    }

    match caller
        .update_file(request)
        .await
        .context("modifying file info")
    {
        Ok(response) => format_response(response)?,
        Err(err) => format_error(err).context("parsing error")?,
    }

    Ok(())
}

pub async fn file_info(token: String, caller: ApiCaller) -> Result<()> {
    let request = WaifuGetRequest::new(token).formatted(true);
    match caller
        .file_info(request)
        .await
        .context("getting file information")
    {
        Ok(response) => format_response(response)?,
        Err(err) => format_error(err).context("parsing error")?,
    }

    Ok(())
}

pub async fn delete_file(token: String, caller: ApiCaller) -> Result<()> {
    let response = caller.delete_file(token).await.context("deleting file");
    match response {
        Ok(deleted) => {
            let mut writer = make_header();
            if deleted {
                writeln!(
                    writer,
                    "{}",
                    "File was deleted successfully!".bright_green().bold()
                )?;
            } else {
                writeln!(
                    writer,
                    "{}",
                    "File was NOT deleted successfully!".bright_red().bold()
                )?;
            }

            print!("{writer}");
        }
        Err(err) => format_error(err).context("parsing error")?,
    }
    // TODO: Output string
    Ok(())
}

fn check_error(err: anyhow::Error) -> Result<WaifuError> {
    let Some(err) = err.downcast_ref::<WaifuError>() else {
        anyhow::bail!("unexpected error: {err:?}");
    };

    Ok(err.to_owned())
}

/// Helper function to determine the appropriate place to save the downloaded item.
///
/// This will create the appropriate path based on the user-specified location or
/// will use a default location instead (the current directory)
fn determine_output_location(url: impl AsRef<str>, output: Option<String>) -> String {
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

fn make_header() -> String {
    let mut writer = String::new();
    writeln!(
        writer,
        "{}",
        "--= Waifu Vault Client =--\n".bright_yellow().bold()
    )
    .unwrap();

    writer
}

fn format_response(response: WaifuResponse) -> Result<()> {
    let mut writer = make_header();
    let options = response.options.unwrap();
    writeln!(writer, "File upload complete!")?;
    writeln!(writer, "Token: {}", response.token.bright_yellow().bold())?;
    writeln!(
        writer,
        "It is stored at {}",
        response.url.bright_cyan().bold(),
    )?;

    if options.protected {
        writeln!(
            writer,
            "It is a {} file",
            "PROTECTED".bright_magenta().bold()
        )?;
    } else {
        writeln!(
            writer,
            "It is an {} file",
            "UNPROTECTED".bright_blue().bold()
        )?;
    }

    writeln!(
        writer,
        "It's available for {}",
        response.retention_period.to_string().bright_green().bold()
    )?;

    if options.one_time_download {
        writeln!(
            writer,
            "It will be {} after download",
            "DELETED".bright_magenta().bold()
        )?;
    }

    print!("{writer}");

    Ok(())
}

fn format_error(error: anyhow::Error) -> Result<()> {
    let error = check_error(error).context("converting to waifu error")?;
    let mut writer = make_header();
    writeln!(
        writer,
        "Received a bad response from the API: {} {}",
        error.name.red().bold(),
        format!("({})", error.status).red().bold()
    )?;

    writeln!(
        writer,
        "This is probably due to: {}",
        error.message.bright_yellow().bold()
    )?;

    print!("{writer}");

    Ok(())
}
