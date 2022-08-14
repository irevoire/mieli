use miette::{IntoDiagnostic, Result};
use reqwest::blocking::Response;
use serde_json::Value;
use termion::color;

pub fn write_response_headers(response: &Response, verbose: usize) -> Result<()> {
    if verbose < 1 {
        return Ok(());
    }
    let colored = atty::is(atty::Stream::Stderr);

    if colored {
        eprint!("{}", color::Fg(color::Cyan));
    }

    for (key, value) in response.headers() {
        eprintln!("{}: {:?}", key, value);
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

pub fn write_response_full(response: Response, verbose: usize) -> Result<Value> {
    write_response_headers(&response, verbose)?;
    write_json(response.json().into_diagnostic()?)
}
