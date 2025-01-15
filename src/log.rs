use std::io::{Read, Write};

use clap::Parser;
use log::{info, warn};
use miette::{IntoDiagnostic, Result};
use serde_json::json;

use crate::{format::write_response_headers, Meilisearch};

#[derive(Debug, Parser)]
pub enum Log {
    /// Stream the logs
    #[clap(aliases = &["get", "retrieve"])]
    Stream {
        /// Specifies either human-readabale or JSON output
        #[clap(default_value = "human")]
        mode: String,
        /// A string specifying one or more log type and its log level
        #[clap(default_value = "info")]
        target: String,
    },
    /// Stop streaming the logs
    #[clap(aliases = &["stop", "interrupt"])]
    Remove,
    /// Update the log target of the logs outputted on stderr
    #[clap(aliases = &["update"])]
    Stderr { target: String },
}

impl Log {
    pub fn execute(self, meili: Meilisearch) -> Result<()> {
        match self {
            Log::Stream { mode, target } => meili.stream_logs(mode, target),
            Log::Remove => meili.remove_log(),
            Log::Stderr { target } => meili.update_logs(target),
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

        info!("Removing the log listener");
        self.remove_log()
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
}
