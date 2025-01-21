use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

use clap::Parser;
use log::warn;
use miette::{IntoDiagnostic, Result};
use serde_json::json;

use crate::{format::write_response_headers, Meilisearch};

#[derive(Debug, Parser)]
pub enum Log {
    /// Stream the logs
    #[clap(aliases = &["get", "retrieve"])]
    Stream {
        /// Specifies either human-readabale or JSON output
        #[clap(long, default_value = "human")]
        mode: String,
        /// A string specifying one or more log type and its log level
        #[clap(long, default_value = "info")]
        target: String,
    },
    /// Stop streaming the logs
    #[clap(aliases = &["stop", "interrupt"])]
    Remove,
    /// Update the log target of the logs outputted on stderr
    #[clap(aliases = &["update"])]
    Stderr { target: String },
    /// Shortcut to profile meilisearch
    Profile {
        /// A string specifying one or more log type and its log level
        target: String,
    },
}

impl Log {
    pub fn execute(self, meili: Meilisearch) -> Result<()> {
        match self {
            Log::Stream { mode, target } => meili.stream_logs(mode, target),
            Log::Remove => meili.remove_log(),
            Log::Stderr { target } => meili.update_logs(target),
            Log::Profile { target } => meili.profile(target),
        }
    }
}

impl Meilisearch {
    fn stream_logs(&self, mode: String, target: String) -> Result<()> {
        let mut response = self
            .post(format!("{}/logs/stream", self.addr))
            .json(&json!({ "mode": mode, "target": target}))
            .send()
            .into_diagnostic()?;
        if !response.status().is_success() {
            self.handle_response(response)?;
            return Ok(());
        }
        write_response_headers(&response, self.verbose)?;

        let this = self.clone();
        let ret = ctrlc::set_handler(move || {
            let ret = this.remove_log();
            if ret.is_err() {
                warn!("Could not disable the log listener, you may need to remove the listener with `mieli stream remove`");
            }
        });
        if ret.is_err() {
            warn!("Could not set up the ctrlc handler. You may need to call remove the stream after exiting with `mieli stream remove`");
        }

        let mut stdout = std::io::stdout();

        let mut buf = [0; 8192];
        loop {
            let read = response.read(&mut buf[..]).into_diagnostic()?;
            if read == 0 {
                break;
            }
            stdout.write_all(&buf[..read]).into_diagnostic()?;
            stdout.flush().into_diagnostic()?;
        }
        Ok(())
    }

    fn remove_log(&self) -> Result<()> {
        let response = self
            .delete(format!("{}/logs/stream", self.addr))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn update_logs(&self, target: String) -> Result<()> {
        let response = self
            .post(format!("{}/logs/stderr", self.addr))
            .json(&json!({ "target": target}))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn profile(&self, target: String) -> Result<()> {
        let response = self
            .post(format!("{}/logs/stream", self.addr))
            .json(&json!({ "mode": "profile", "target": target}))
            .send()
            .into_diagnostic()?;
        if !response.status().is_success() {
            self.handle_response(response)?;
            return Ok(());
        }
        write_response_headers(&response, self.verbose)?;

        let this = self.clone();
        let ret = ctrlc::set_handler(move || {
            let ret = this.remove_log();
            if ret.is_err() {
                warn!("Could not disable the log listener, you may need to remove the listener with `mieli stream remove`");
            }
        });
        if ret.is_err() {
            warn!("Could not set up the ctrlc handler. You may need to call remove the stream after exiting with `mieli stream remove`");
        }

        let trace = tracing_trace::TraceReader::new(BufReader::new(response));
        let profile =
            tracing_trace::processor::firefox_profiler::to_firefox_profile(trace, "Meilisearch")
                .unwrap();
        let now = time::OffsetDateTime::now_utc();
        let filename = format!(
            "firefox-{}_{}_{}-{}:{}:{}.profile",
            now.year(),
            now.month() as u8,
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );
        let mut output_file = BufWriter::new(File::create(filename).unwrap());
        serde_json::to_writer(&mut output_file, &profile).unwrap();
        output_file.flush().unwrap();
        Ok(())
    }
}
