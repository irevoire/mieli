use miette::{Context, IntoDiagnostic, Result};
use reqwest::blocking::Response;
use serde_json::Value;
use termion::color;

pub fn write_response_headers(response: &Response, verbose: u8) -> Result<()> {
    let status = response.status();
    if verbose < 1 && status.is_success() {
        return Ok(());
    }
    let colored = atty::is(atty::Stream::Stderr);

    // HTTP/1.1 404 Not Found
    if colored {
        eprint!("{}", color::Fg(color::Blue));
    }
    eprint!("{:?} {:?}", response.version(), status);
    if colored {
        eprint!("{}", color::Fg(color::Cyan));
    }
    if let Some(reason) = status.canonical_reason() {
        eprint!(" {reason}");
    }
    eprintln!();

    let mut headers: Vec<_> = response.headers().iter().collect();
    headers.sort_by_key(|(k, _)| k.as_str());

    for (key, value) in response.headers() {
        if colored {
            eprint!("{}", color::Fg(color::Cyan));
        }

        eprint!("{key}:");

        if colored {
            eprint!("{}", color::Fg(color::White));
        }
        if let Ok(value) = value.to_str() {
            eprintln!(" {value}");
        } else {
            eprintln!(" {value:?}");
        }
    }

    if colored {
        eprintln!("{}", color::Fg(color::Reset));
    }
    Ok(())
}

pub fn write_json(response: Value) -> Result<Value> {
    if atty::is(atty::Stream::Stdout) {
        println!(
            "{}",
            colored_json::to_colored_json_auto(&response).into_diagnostic()?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&response).into_diagnostic()?
        );
    }
    Ok(response)
}

pub fn write_response_full(response: Response, verbose: u8) -> Result<Value> {
    write_response_headers(&response, verbose)?;
    let body = response
        .bytes()
        .into_diagnostic()
        .context("While retrieving the body as bytes")?;
    if body.is_empty() {
        return Ok(serde_json::Value::Null);
    }
    let json: serde_json::Value = serde_json::from_slice(&body)
        .into_diagnostic()
        .context(format!("While converting the body as json: {body:?}"))?;
    write_json(json)
}
