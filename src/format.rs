use anyhow::Result;
use reqwest::blocking::Response;
use serde_json::Value;
use std::io::Write;
use termion::color;

pub fn write_response_headers(output: &mut dyn Write, response: &Response) -> Result<()> {
    write!(output, "{}", color::Fg(color::Cyan))?;
    for (key, value) in response.headers() {
        writeln!(output, "{}: {:?}", key, value)?;
    }
    write!(output, "{}", color::Fg(color::Reset))?;
    Ok(())
}

pub fn write_json(output: &mut dyn Write, response: Value) -> Result<Value> {
    writeln!(output, "{}", color::Fg(color::Reset))?;
    writeln!(output, "{}", colored_json::to_colored_json_auto(&response)?)?;
    Ok(response)
}

pub fn write_response_full(output: &mut dyn Write, response: Response) -> Result<Value> {
    write_response_headers(output, &response)?;
    write_json(output, response.json()?)
}
